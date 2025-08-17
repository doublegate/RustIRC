//! Message routing and command handling
//!
//! This module provides comprehensive message routing capabilities for IRC clients:
//! - Incoming message processing and routing to appropriate handlers
//! - Command validation and execution
//! - Message filtering and transformation
//! - Plugin/script integration points
//! - Rate limiting and flood protection

use crate::events::{Event, EventBus};
use crate::error::{Error, Result};
use crate::state::{StateManager, User};
use rustirc_protocol::{Command, Message, Numeric, Prefix};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn, error, info};

/// Message handler trait for processing different types of IRC messages
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an IRC message
    async fn handle_message(&self, context: &MessageContext, message: &Message) -> Result<()>;
    
    /// Get the priority of this handler (higher = processed first)
    fn priority(&self) -> i32 { 0 }
    
    /// Get the message types this handler can process
    fn message_types(&self) -> Vec<String>;
    
    /// Check if this handler should process the message
    fn should_handle(&self, message: &Message) -> bool {
        self.message_types().contains(&message.command.to_uppercase())
    }
}

/// Context information for message processing
#[derive(Debug, Clone)]
pub struct MessageContext {
    pub connection_id: String,
    pub timestamp: Instant,
    pub is_own_message: bool,
    pub target_channel: Option<String>,
    pub source_user: Option<User>,
}

impl MessageContext {
    pub fn new(connection_id: String) -> Self {
        Self {
            connection_id,
            timestamp: Instant::now(),
            is_own_message: false,
            target_channel: None,
            source_user: None,
        }
    }

    pub fn with_channel(mut self, channel: String) -> Self {
        self.target_channel = Some(channel);
        self
    }

    pub fn with_user(mut self, user: User) -> Self {
        self.source_user = Some(user);
        self
    }

    pub fn mark_own_message(mut self) -> Self {
        self.is_own_message = true;
        self
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_messages_per_second: u32,
    pub burst_capacity: u32,
    pub penalty_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_messages_per_second: 10,
            burst_capacity: 20,
            penalty_duration: Duration::from_secs(30),
        }
    }
}

/// Rate limiting state for connections/users
#[derive(Debug)]
struct RateLimitState {
    last_message: Instant,
    message_count: u32,
    penalty_until: Option<Instant>,
}

impl RateLimitState {
    fn new() -> Self {
        Self {
            last_message: Instant::now(),
            message_count: 0,
            penalty_until: None,
        }
    }

    fn check_rate_limit(&mut self, config: &RateLimitConfig) -> bool {
        let now = Instant::now();
        
        // Check if still in penalty period
        if let Some(penalty_until) = self.penalty_until {
            if now < penalty_until {
                return false;
            } else {
                self.penalty_until = None;
                self.message_count = 0;
            }
        }

        // Reset counter if enough time has passed
        if now.duration_since(self.last_message) >= Duration::from_secs(1) {
            self.message_count = 0;
        }

        // Check rate limit
        if self.message_count >= config.max_messages_per_second {
            // Apply penalty
            self.penalty_until = Some(now + config.penalty_duration);
            warn!("Rate limit exceeded, applying penalty");
            return false;
        }

        self.message_count += 1;
        self.last_message = now;
        true
    }
}

/// Message router that handles incoming IRC messages
pub struct MessageRouter {
    handlers: Arc<RwLock<Vec<Box<dyn MessageHandler>>>>,
    state_manager: Arc<StateManager>,
    event_bus: Arc<EventBus>,
    rate_limits: Arc<RwLock<HashMap<String, RateLimitState>>>,
    rate_limit_config: RateLimitConfig,
    command_queue: mpsc::UnboundedSender<(String, Command)>, // connection_id, command
}

impl MessageRouter {
    pub fn new(
        state_manager: Arc<StateManager>,
        event_bus: Arc<EventBus>,
        command_queue: mpsc::UnboundedSender<(String, Command)>,
    ) -> Self {
        let mut router = Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
            state_manager,
            event_bus,
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
            rate_limit_config: RateLimitConfig::default(),
            command_queue,
        };

        // Register built-in handlers
        router.register_builtin_handlers();
        router
    }

    /// Register a message handler
    pub async fn register_handler<H: MessageHandler + 'static>(&self, handler: H) {
        let mut handlers = self.handlers.write().await;
        handlers.push(Box::new(handler));
        // Sort by priority (highest first)
        handlers.sort_by_key(|h| -h.priority());
    }

    /// Process an incoming IRC message
    pub async fn route_message(&self, connection_id: String, message: Message) -> Result<()> {
        // Check rate limits
        if !self.check_rate_limit(&connection_id).await {
            debug!("Message dropped due to rate limiting: {}", connection_id);
            return Ok(());
        }

        // Create message context
        let mut context = MessageContext::new(connection_id.clone());
        
        // Extract user information from prefix
        if let Some(prefix) = &message.prefix {
            context.source_user = Some(User::from_prefix(prefix));
        }

        // Determine target channel for channel messages
        match message.command.as_str() {
            "PRIVMSG" | "NOTICE" | "JOIN" | "PART" | "TOPIC" | "MODE" => {
                if let Some(target) = message.params.first() {
                    if target.starts_with('#') || target.starts_with('&') {
                        context.target_channel = Some(target.clone());
                    }
                }
            }
            _ => {}
        }

        debug!("Routing message: {} from {}", message.command, connection_id);

        // Route to handlers
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            if handler.should_handle(&message) {
                if let Err(e) = handler.handle_message(&context, &message).await {
                    error!("Handler error for {}: {}", message.command, e);
                }
            }
        }

        // Update state through event system
        let event = Event::MessageReceived {
            connection_id: connection_id.clone(),
            message: message.clone(),
        };
        self.event_bus.emit(event).await;

        Ok(())
    }

    /// Send a command to a specific connection
    pub async fn send_command(&self, connection_id: String, command: Command) -> Result<()> {
        self.command_queue.send((connection_id.clone(), command.clone()))
            .map_err(|_| Error::ConnectionClosed)?;

        // Emit event
        let message = command.to_message();
        let event = Event::MessageSent {
            connection_id,
            message,
        };
        self.event_bus.emit(event).await;

        Ok(())
    }

    /// Check rate limits for a connection
    async fn check_rate_limit(&self, connection_id: &str) -> bool {
        let mut rate_limits = self.rate_limits.write().await;
        let state = rate_limits.entry(connection_id.to_string())
            .or_insert_with(RateLimitState::new);
        
        state.check_rate_limit(&self.rate_limit_config)
    }

    /// Register built-in message handlers
    fn register_builtin_handlers(&mut self) {
        // Will be implemented with specific handlers
    }
}

/// Built-in handler for PING/PONG messages
pub struct PingHandler {
    router: Arc<MessageRouter>,
}

impl PingHandler {
    pub fn new(router: Arc<MessageRouter>) -> Self {
        Self { router }
    }
}

#[async_trait::async_trait]
impl MessageHandler for PingHandler {
    async fn handle_message(&self, context: &MessageContext, message: &Message) -> Result<()> {
        if message.command == "PING" {
            if let Some(server) = message.params.first() {
                let pong_cmd = Command::Pong {
                    server1: server.clone(),
                    server2: None,
                };
                self.router.send_command(context.connection_id.clone(), pong_cmd).await?;
                debug!("Responded to PING from {}", server);
            }
        }
        Ok(())
    }

    fn priority(&self) -> i32 {
        1000 // High priority for ping responses
    }

    fn message_types(&self) -> Vec<String> {
        vec!["PING".to_string()]
    }
}

/// Handler for numeric replies (001, 002, etc.)
pub struct NumericHandler;

#[async_trait::async_trait]
impl MessageHandler for NumericHandler {
    async fn handle_message(&self, context: &MessageContext, message: &Message) -> Result<()> {
        if let Ok(numeric) = message.command.parse::<u16>() {
            match numeric {
                001 => {
                    // RPL_WELCOME - Registration successful
                    info!("Registration successful for {}", context.connection_id);
                    // Emit registration event
                }
                353 => {
                    // RPL_NAMREPLY - Names list
                    if message.params.len() >= 3 {
                        let channel = &message.params[2];
                        let default_names = String::new();
                        let names = message.params.last().unwrap_or(&default_names);
                        debug!("Names for {}: {}", channel, names);
                        // Process user list
                    }
                }
                366 => {
                    // RPL_ENDOFNAMES - End of names list
                    if message.params.len() >= 2 {
                        let channel = &message.params[1];
                        debug!("End of names for {}", channel);
                    }
                }
                432 | 433 => {
                    // ERR_ERRONEUSNICKNAME or ERR_NICKNAMEINUSE
                    warn!("Nickname error for {}: {}", context.connection_id, numeric);
                    // Could trigger automatic nick change
                }
                _ => {
                    debug!("Unhandled numeric {}: {:?}", numeric, message.params);
                }
            }
        }
        Ok(())
    }

    fn priority(&self) -> i32 {
        500 // Medium-high priority
    }

    fn message_types(&self) -> Vec<String> {
        // Return all numeric codes as strings
        (1..=999).map(|n| format!("{:03}", n)).collect()
    }
    
    fn should_handle(&self, message: &Message) -> bool {
        message.command.parse::<u16>().is_ok()
    }
}

/// Handler for channel events (JOIN, PART, QUIT, etc.)
pub struct ChannelHandler;

#[async_trait::async_trait]
impl MessageHandler for ChannelHandler {
    async fn handle_message(&self, context: &MessageContext, message: &Message) -> Result<()> {
        match message.command.as_str() {
            "JOIN" => {
                if let Some(channel) = message.params.first() {
                    info!("User joined {}: {:?}", channel, context.source_user);
                    // Emit channel join event
                    if let Some(event_bus) = self.get_event_bus(context).await {
                        let event = Event::ChannelJoined {
                            connection_id: context.connection_id.clone(),
                            channel: channel.clone(),
                        };
                        event_bus.emit(event).await;
                    }
                }
            }
            "PART" => {
                if let Some(channel) = message.params.first() {
                    let reason = message.params.get(1).cloned();
                    info!("User left {}: {:?} ({})", channel, context.source_user, 
                          reason.as_deref().unwrap_or("no reason"));
                    
                    // Emit channel part event
                    if let Some(event_bus) = self.get_event_bus(context).await {
                        let event = Event::ChannelLeft {
                            connection_id: context.connection_id.clone(),
                            channel: channel.clone(),
                        };
                        event_bus.emit(event).await;
                    }
                }
            }
            "QUIT" => {
                if let Some(user) = &context.source_user {
                    let reason = message.params.first().cloned();
                    info!("User quit: {} ({})", user.nickname, 
                          reason.as_deref().unwrap_or("no reason"));
                }
            }
            "NICK" => {
                if let Some(new_nick) = message.params.first() {
                    if let Some(user) = &context.source_user {
                        info!("Nick change: {} -> {}", user.nickname, new_nick);
                        
                        // Emit nick change event
                        if let Some(event_bus) = self.get_event_bus(context).await {
                            let event = Event::NickChanged {
                                connection_id: context.connection_id.clone(),
                                old: user.nickname.clone(),
                                new: new_nick.clone(),
                            };
                            event_bus.emit(event).await;
                        }
                    }
                }
            }
            "TOPIC" => {
                if message.params.len() >= 2 {
                    let channel = &message.params[0];
                    let topic = &message.params[1];
                    info!("Topic changed in {}: {}", channel, topic);
                    
                    // Emit topic change event
                    if let Some(event_bus) = self.get_event_bus(context).await {
                        let event = Event::TopicChanged {
                            connection_id: context.connection_id.clone(),
                            channel: channel.clone(),
                            topic: topic.clone(),
                        };
                        event_bus.emit(event).await;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn priority(&self) -> i32 {
        400 // Medium priority
    }

    fn message_types(&self) -> Vec<String> {
        vec![
            "JOIN".to_string(),
            "PART".to_string(),
            "QUIT".to_string(),
            "NICK".to_string(),
            "TOPIC".to_string(),
            "MODE".to_string(),
        ]
    }
}

impl ChannelHandler {
    async fn get_event_bus(&self, _context: &MessageContext) -> Option<Arc<EventBus>> {
        // In a real implementation, this would get the event bus from the context
        // For now, return None as this is a placeholder
        None
    }
}

/// Handler for private messages and notices
pub struct MessageHandler_ {
    highlight_patterns: Vec<String>,
}

impl MessageHandler_ {
    pub fn new(highlight_patterns: Vec<String>) -> Self {
        Self { highlight_patterns }
    }

    fn is_highlighted(&self, message: &str) -> bool {
        for pattern in &self.highlight_patterns {
            if message.to_lowercase().contains(&pattern.to_lowercase()) {
                return true;
            }
        }
        false
    }
}

#[async_trait::async_trait]
impl MessageHandler for MessageHandler_ {
    async fn handle_message(&self, context: &MessageContext, message: &Message) -> Result<()> {
        if message.params.len() >= 2 {
            let target = &message.params[0];
            let text = &message.params[1];
            
            let is_private = !target.starts_with('#') && !target.starts_with('&');
            let is_highlighted = self.is_highlighted(text);
            
            match message.command.as_str() {
                "PRIVMSG" => {
                    if is_private {
                        info!("Private message from {:?}: {}", context.source_user, text);
                    } else {
                        if is_highlighted {
                            info!("Highlighted message in {}: {}", target, text);
                        } else {
                            debug!("Channel message in {}: {}", target, text);
                        }
                    }
                }
                "NOTICE" => {
                    if is_private {
                        info!("Private notice from {:?}: {}", context.source_user, text);
                    } else {
                        debug!("Channel notice in {}: {}", target, text);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn priority(&self) -> i32 {
        300 // Lower priority than channel events
    }

    fn message_types(&self) -> Vec<String> {
        vec!["PRIVMSG".to_string(), "NOTICE".to_string()]
    }
}

/// Command processor for outgoing commands
pub struct CommandProcessor {
    router: Arc<MessageRouter>,
    aliases: HashMap<String, String>,
}

impl CommandProcessor {
    pub fn new(router: Arc<MessageRouter>) -> Self {
        let mut processor = Self {
            router,
            aliases: HashMap::new(),
        };
        processor.setup_default_aliases();
        processor
    }

    /// Process a user command (from UI input)
    pub async fn process_command(&self, connection_id: String, input: &str) -> Result<()> {
        let input = input.trim();
        
        if input.is_empty() {
            return Ok(());
        }

        if input.starts_with('/') {
            // Command
            self.process_slash_command(connection_id, &input[1..]).await
        } else {
            // Regular message - needs target context
            Err(Error::Protocol("No target specified for message".to_string()))
        }
    }

    /// Process a slash command
    async fn process_slash_command(&self, connection_id: String, input: &str) -> Result<()> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        let cmd_name = parts[0].to_lowercase();
        let args = &parts[1..];

        // Check for aliases
        let cmd_name = self.aliases.get(&cmd_name).unwrap_or(&cmd_name).clone();

        let command = match cmd_name.as_str() {
            "join" | "j" => {
                if !args.is_empty() {
                    Some(Command::Join {
                        channels: vec![args[0].to_string()],
                        keys: vec![],
                    })
                } else {
                    return Err(Error::Protocol("JOIN requires a channel name".to_string()));
                }
            }
            "part" | "leave" => {
                if !args.is_empty() {
                    let reason = if args.len() > 1 {
                        Some(args[1..].join(" "))
                    } else {
                        None
                    };
                    Some(Command::Part {
                        channels: vec![args[0].to_string()],
                        message: reason,
                    })
                } else {
                    return Err(Error::Protocol("PART requires a channel name".to_string()));
                }
            }
            "nick" => {
                if !args.is_empty() {
                    Some(Command::Nick {
                        nickname: args[0].to_string(),
                    })
                } else {
                    return Err(Error::Protocol("NICK requires a nickname".to_string()));
                }
            }
            "quit" => {
                let reason = if !args.is_empty() {
                    Some(args.join(" "))
                } else {
                    Some("RustIRC".to_string())
                };
                Some(Command::Quit { message: reason })
            }
            "msg" | "privmsg" => {
                if args.len() >= 2 {
                    Some(Command::PrivMsg {
                        target: args[0].to_string(),
                        text: args[1..].join(" "),
                    })
                } else {
                    return Err(Error::Protocol("MSG requires target and message".to_string()));
                }
            }
            "notice" => {
                if args.len() >= 2 {
                    Some(Command::Notice {
                        target: args[0].to_string(),
                        text: args[1..].join(" "),
                    })
                } else {
                    return Err(Error::Protocol("NOTICE requires target and message".to_string()));
                }
            }
            "topic" => {
                if !args.is_empty() {
                    let topic = if args.len() > 1 {
                        Some(args[1..].join(" "))
                    } else {
                        None
                    };
                    Some(Command::Topic {
                        channel: args[0].to_string(),
                        topic,
                    })
                } else {
                    return Err(Error::Protocol("TOPIC requires a channel name".to_string()));
                }
            }
            "mode" => {
                if !args.is_empty() {
                    let modes = if args.len() > 1 {
                        Some(args[1..].join(" "))
                    } else {
                        None
                    };
                    Some(Command::Mode {
                        target: args[0].to_string(),
                        modes,
                        params: vec![],
                    })
                } else {
                    return Err(Error::Protocol("MODE requires a target".to_string()));
                }
            }
            "whois" => {
                if !args.is_empty() {
                    Some(Command::Whois {
                        targets: vec![args[0].to_string()],
                    })
                } else {
                    return Err(Error::Protocol("WHOIS requires a nickname".to_string()));
                }
            }
            _ => {
                // Unknown command, send as raw
                Some(Command::Raw {
                    command: cmd_name.to_uppercase(),
                    params: args.iter().map(|s| s.to_string()).collect(),
                })
            }
        };

        if let Some(command) = command {
            self.router.send_command(connection_id, command).await?;
        }

        Ok(())
    }

    /// Set up default command aliases
    fn setup_default_aliases(&mut self) {
        self.aliases.insert("j".to_string(), "join".to_string());
        self.aliases.insert("leave".to_string(), "part".to_string());
        self.aliases.insert("msg".to_string(), "privmsg".to_string());
        self.aliases.insert("q".to_string(), "quit".to_string());
        self.aliases.insert("wii".to_string(), "whois".to_string());
    }

    /// Add a custom alias
    pub fn add_alias(&mut self, alias: String, command: String) {
        self.aliases.insert(alias, command);
    }
}