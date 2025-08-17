//! State management with event sourcing
//!
//! This module provides comprehensive state management for IRC clients using
//! event sourcing patterns for reliable state reconstruction and persistence.

use crate::events::Event;
use crate::error::{Error, Result};
use rustirc_protocol::{Message, Prefix};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Complete client state containing all servers and global settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientState {
    pub servers: HashMap<String, ServerState>,
    pub current_server: Option<String>,
    pub global_settings: GlobalSettings,
    pub version: u64, // State version for event sourcing
}

/// Global client settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub auto_reconnect: bool,
    pub reconnect_delay: u64, // seconds
    pub default_nickname: String,
    pub default_username: String,
    pub default_realname: String,
    pub highlight_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            reconnect_delay: 5,
            default_nickname: "RustIRC".to_string(),
            default_username: "rustirc".to_string(),
            default_realname: "RustIRC Client".to_string(),
            highlight_patterns: vec![],
            ignore_patterns: vec![],
        }
    }
}

/// State for a single IRC server connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub connection_id: String,
    pub address: String,
    pub port: u16,
    pub use_tls: bool,
    pub connected: bool,
    pub registered: bool,
    pub nickname: String,
    pub username: String,
    pub realname: String,
    pub channels: HashMap<String, ChannelState>,
    pub users: HashMap<String, User>, // Global user cache
    pub capabilities: Vec<String>,
    pub isupport: HashMap<String, String>,
    pub server_info: ServerInfo,
    pub last_ping: Option<u64>, // Unix timestamp
    pub message_history: VecDeque<HistoryEntry>,
}

impl ServerState {
    pub fn new(connection_id: String, address: String, port: u16, use_tls: bool) -> Self {
        Self {
            connection_id,
            address,
            port,
            use_tls,
            connected: false,
            registered: false,
            nickname: "RustIRC".to_string(),
            username: "rustirc".to_string(),
            realname: "RustIRC Client".to_string(),
            channels: HashMap::new(),
            users: HashMap::new(),
            capabilities: Vec::new(),
            isupport: HashMap::new(),
            server_info: ServerInfo::default(),
            last_ping: None,
            message_history: VecDeque::with_capacity(1000),
        }
    }
}

/// Information about the IRC server
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub created: Option<String>,
    pub motd: Vec<String>,
    pub admin_info: Vec<String>,
}

/// State for a single IRC channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    pub name: String,
    pub topic: Option<TopicInfo>,
    pub users: HashMap<String, ChannelUser>, // Nick -> User info
    pub modes: ChannelModes,
    pub joined: bool,
    pub message_history: VecDeque<HistoryEntry>,
    pub user_limit: Option<u32>,
    pub creation_time: Option<u64>,
}

impl ChannelState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            topic: None,
            users: HashMap::new(),
            modes: ChannelModes::default(),
            joined: false,
            message_history: VecDeque::with_capacity(1000),
            user_limit: None,
            creation_time: None,
        }
    }

    /// Add user to channel
    pub fn add_user(&mut self, nick: String, user: User) {
        let channel_user = ChannelUser {
            nick: nick.clone(),
            modes: Vec::new(),
            join_time: current_timestamp(),
        };
        self.users.insert(nick, channel_user);
    }

    /// Remove user from channel
    pub fn remove_user(&mut self, nick: &str) -> Option<ChannelUser> {
        self.users.remove(nick)
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        self.users.len()
    }
}

/// Channel-specific user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUser {
    pub nick: String,
    pub modes: Vec<char>, // Channel-specific modes (o, v, etc.)
    pub join_time: u64,
}

/// Channel modes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelModes {
    pub modes: String, // Raw mode string
    pub secret: bool,
    pub private: bool,
    pub invite_only: bool,
    pub topic_protected: bool,
    pub no_external_messages: bool,
    pub moderated: bool,
    pub key: Option<String>,
    pub limit: Option<u32>,
}

/// Topic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub text: String,
    pub set_by: String,
    pub set_time: u64,
}

/// Global user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub nickname: String,
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub realname: Option<String>,
    pub server: Option<String>,
    pub away: bool,
    pub away_message: Option<String>,
    pub idle_time: Option<u64>,
    pub signon_time: Option<u64>,
    pub oper: bool,
    pub account: Option<String>, // SASL account
}

impl User {
    pub fn from_prefix(prefix: &Prefix) -> Self {
        match prefix {
            Prefix::User { nick, user, host } => Self {
                nickname: nick.clone(),
                username: user.clone(),
                hostname: host.clone(),
                realname: None,
                server: None,
                away: false,
                away_message: None,
                idle_time: None,
                signon_time: None,
                oper: false,
                account: None,
            },
            Prefix::Server(server) => Self {
                nickname: server.clone(),
                username: None,
                hostname: None,
                realname: None,
                server: Some(server.clone()),
                away: false,
                away_message: None,
                idle_time: None,
                signon_time: None,
                oper: false,
                account: None,
            },
        }
    }
}

/// Message history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: u64,
    pub message: Message,
    pub processed: bool,
}

impl HistoryEntry {
    pub fn new(message: Message) -> Self {
        Self {
            timestamp: current_timestamp(),
            message,
            processed: false,
        }
    }
}

/// State event for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEvent {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: StateEventType,
    pub connection_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateEventType {
    // Connection events
    ServerConnected,
    ServerDisconnected { reason: String },
    ServerRegistered,
    
    // User events
    NickChanged { old_nick: String, new_nick: String },
    UserJoined { channel: String, user: User },
    UserLeft { channel: String, nick: String, reason: Option<String> },
    UserQuit { nick: String, reason: Option<String> },
    UserModeChanged { nick: String, modes: String },
    
    // Channel events
    ChannelJoined { channel: String },
    ChannelLeft { channel: String, reason: Option<String> },
    TopicChanged { channel: String, topic: TopicInfo },
    ChannelModeChanged { channel: String, modes: String },
    
    // Message events
    MessageReceived { target: String, message: Message },
    MessageSent { target: String, message: Message },
    
    // Server events
    CapabilitiesReceived { capabilities: Vec<String> },
    IsupportReceived { params: HashMap<String, String> },
    MotdReceived { lines: Vec<String> },
}

/// Centralized state manager with event sourcing
pub struct StateManager {
    state: Arc<RwLock<ClientState>>,
    events: Arc<RwLock<Vec<StateEvent>>>,
    event_id_counter: Arc<RwLock<u64>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ClientState::default())),
            events: Arc::new(RwLock::new(Vec::new())),
            event_id_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Get current state snapshot
    pub async fn get_state(&self) -> ClientState {
        self.state.read().await.clone()
    }

    /// Get server state by connection ID
    pub async fn get_server_state(&self, connection_id: &str) -> Option<ServerState> {
        let state = self.state.read().await;
        state.servers.get(connection_id).cloned()
    }

    /// Apply an IRC event to the state
    pub async fn apply_event(&self, event: &Event) -> Result<()> {
        let mut state = self.state.write().await;
        let mut events = self.events.write().await;
        let mut counter = self.event_id_counter.write().await;
        
        let state_event = self.create_state_event(&event, *counter).await?;
        *counter += 1;
        
        // Apply the event to the state
        self.apply_state_event(&mut state, &state_event).await?;
        
        // Store the event for persistence/replay
        events.push(state_event);
        
        // Update state version
        state.version += 1;
        
        debug!("Applied event, new state version: {}", state.version);
        
        Ok(())
    }

    /// Create state event from IRC event
    async fn create_state_event(&self, event: &Event, id: u64) -> Result<StateEvent> {
        let event_type = match event {
            Event::Connected { connection_id } => {
                StateEventType::ServerConnected
            }
            Event::Disconnected { connection_id, reason } => {
                StateEventType::ServerDisconnected { reason: reason.clone() }
            }
            Event::MessageReceived { connection_id, message } => {
                let target = self.determine_message_target(message).await;
                StateEventType::MessageReceived { 
                    target, 
                    message: message.clone() 
                }
            }
            Event::ChannelJoined { connection_id, channel } => {
                StateEventType::ChannelJoined { channel: channel.clone() }
            }
            Event::ChannelLeft { connection_id, channel } => {
                StateEventType::ChannelLeft { 
                    channel: channel.clone(), 
                    reason: None 
                }
            }
            Event::NickChanged { connection_id, old, new } => {
                StateEventType::NickChanged { 
                    old_nick: old.clone(), 
                    new_nick: new.clone() 
                }
            }
            _ => {
                return Err(Error::State("Unsupported event type".to_string()));
            }
        };

        let connection_id = match event {
            Event::Connected { connection_id } |
            Event::Disconnected { connection_id, .. } |
            Event::MessageReceived { connection_id, .. } |
            Event::ChannelJoined { connection_id, .. } |
            Event::ChannelLeft { connection_id, .. } |
            Event::NickChanged { connection_id, .. } => connection_id.clone(),
            _ => String::new(),
        };

        Ok(StateEvent {
            id,
            timestamp: current_timestamp(),
            event_type,
            connection_id,
        })
    }

    /// Apply state event to the current state
    async fn apply_state_event(&self, state: &mut ClientState, event: &StateEvent) -> Result<()> {
        let server_state = state.servers.entry(event.connection_id.clone())
            .or_insert_with(|| ServerState::new(
                event.connection_id.clone(),
                "unknown".to_string(),
                6667,
                false,
            ));

        match &event.event_type {
            StateEventType::ServerConnected => {
                server_state.connected = true;
            }
            StateEventType::ServerDisconnected { reason } => {
                server_state.connected = false;
                server_state.registered = false;
                // Clear channel join status
                for channel in server_state.channels.values_mut() {
                    channel.joined = false;
                }
            }
            StateEventType::ServerRegistered => {
                server_state.registered = true;
            }
            StateEventType::ChannelJoined { channel } => {
                let channel_state = server_state.channels.entry(channel.clone())
                    .or_insert_with(|| ChannelState::new(channel.clone()));
                channel_state.joined = true;
            }
            StateEventType::ChannelLeft { channel, reason } => {
                if let Some(channel_state) = server_state.channels.get_mut(channel) {
                    channel_state.joined = false;
                }
            }
            StateEventType::NickChanged { old_nick, new_nick } => {
                server_state.nickname = new_nick.clone();
                // Update nick in all channels
                for channel_state in server_state.channels.values_mut() {
                    if let Some(user) = channel_state.users.remove(old_nick) {
                        let mut new_user = user;
                        new_user.nick = new_nick.clone();
                        channel_state.users.insert(new_nick.clone(), new_user);
                    }
                }
            }
            StateEventType::MessageReceived { target, message } => {
                // Add to appropriate message history
                let history_entry = HistoryEntry::new(message.clone());
                
                if target.starts_with('#') || target.starts_with('&') {
                    // Channel message
                    if let Some(channel_state) = server_state.channels.get_mut(target) {
                        channel_state.message_history.push_back(history_entry);
                        // Keep only last 1000 messages
                        if channel_state.message_history.len() > 1000 {
                            channel_state.message_history.pop_front();
                        }
                    }
                } else {
                    // Private message or server message
                    server_state.message_history.push_back(history_entry);
                    if server_state.message_history.len() > 1000 {
                        server_state.message_history.pop_front();
                    }
                }
            }
            StateEventType::TopicChanged { channel, topic } => {
                if let Some(channel_state) = server_state.channels.get_mut(channel) {
                    channel_state.topic = Some(topic.clone());
                }
            }
            _ => {
                // Handle other event types
                debug!("Unhandled state event type: {:?}", event.event_type);
            }
        }

        Ok(())
    }

    /// Determine message target for state storage
    async fn determine_message_target(&self, message: &Message) -> String {
        match message.command.as_str() {
            "PRIVMSG" | "NOTICE" => {
                message.params.first().unwrap_or(&"unknown".to_string()).clone()
            }
            _ => "server".to_string(),
        }
    }

    /// Replay events to reconstruct state (useful for persistence)
    pub async fn replay_events(&self, events: Vec<StateEvent>) -> Result<()> {
        let mut state = self.state.write().await;
        *state = ClientState::default();
        
        for event in events {
            self.apply_state_event(&mut state, &event).await?;
        }
        
        Ok(())
    }

    /// Get all events (for persistence)
    pub async fn get_events(&self) -> Vec<StateEvent> {
        self.events.read().await.clone()
    }

    /// Clear old events (for memory management)
    pub async fn compact_events(&self, keep_last_n: usize) -> Result<()> {
        let mut events = self.events.write().await;
        let len = events.len();
        if len > keep_last_n {
            events.drain(0..len - keep_last_n);
        }
        Ok(())
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}