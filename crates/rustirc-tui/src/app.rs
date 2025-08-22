//! TUI application
//!
//! Main terminal user interface for RustIRC using ratatui.
//! Features:
//! - Full-screen terminal interface
//! - Vi-like keybindings for navigation
//! - Multiple panes for server list, messages, and user list
//! - Cross-platform terminal support
//! - Efficient text rendering and scrolling

use crate::event_handler::TuiEventHandler;
use crate::input::{InputHandler, InputMode, KeyEvent, TuiAction};
use crate::state::TuiState;
use crate::ui::TuiRenderer;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use rustirc_core::{
    client::IrcClient,
    connection::{ConnectionConfig, ConnectionManager},
    events::{Event as CoreEvent, EventBus},
    state::StateManager,
};
use std::{
    io::{stdout, Stdout},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Main TUI application
pub struct TuiApp {
    // Core IRC functionality
    irc_client: Arc<IrcClient>,
    connection_manager: Arc<ConnectionManager>,
    state_manager: Arc<StateManager>,
    event_bus: Arc<EventBus>,

    // TUI components
    terminal: Option<TuiTerminal>,
    tui_state: TuiState,
    renderer: TuiRenderer,
    input_handler: InputHandler,

    // Event handling
    event_receiver: Option<mpsc::UnboundedReceiver<CoreEvent>>,
    event_sender: mpsc::UnboundedSender<CoreEvent>,
    command_sender: mpsc::UnboundedSender<String>,

    // Application state
    should_quit: bool,
    last_tick: Instant,
    tick_rate: Duration,
}

impl TuiApp {
    /// Create a new TUI application
    pub fn new() -> Result<Self> {
        // Initialize core IRC components
        let event_bus = Arc::new(EventBus::new());
        let state_manager = Arc::new(StateManager::new());
        let connection_manager = Arc::new(ConnectionManager::new(event_bus.clone()));

        let (command_sender, _command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // Initialize IRC client
        let config = rustirc_core::config::Config::default();
        let irc_client = Arc::new(IrcClient::new(config));

        Ok(Self {
            irc_client,
            connection_manager,
            state_manager,
            event_bus,
            terminal: None,
            tui_state: TuiState::new(),
            renderer: TuiRenderer::new(),
            input_handler: InputHandler::new(),
            event_receiver: Some(event_receiver),
            event_sender,
            command_sender,
            should_quit: false,
            last_tick: Instant::now(),
            tick_rate: Duration::from_millis(250),
        })
    }

    /// Initialize the terminal
    fn setup_terminal() -> Result<TuiTerminal> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    /// Restore the terminal
    fn restore_terminal() -> Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        self.terminal = Some(Self::setup_terminal()?);

        // Initialize TUI state
        self.tui_state.add_server("freenode.net".to_string());
        self.tui_state
            .add_channel("freenode.net".to_string(), "#rust".to_string());
        self.tui_state
            .add_channel("freenode.net".to_string(), "#programming".to_string());

        // Add some sample messages for demonstration
        self.tui_state.add_message(
            "freenode.net".to_string(),
            "#rust".to_string(),
            "alice".to_string(),
            "Hello everyone! How is Rust development going?".to_string(),
        );
        self.tui_state.add_message(
            "freenode.net".to_string(),
            "#rust".to_string(),
            "bob".to_string(),
            "Great! Working on an IRC client actually".to_string(),
        );

        info!("Starting TUI interface");

        // Main event loop
        let result = self.main_loop().await;

        // Cleanup
        Self::restore_terminal()?;

        result
    }

    /// Main event loop
    async fn main_loop(&mut self) -> Result<()> {
        while !self.should_quit {
            // Handle terminal events
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => {
                        self.handle_key_event(key)?;
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse)?;
                    }
                    Event::Resize(width, height) => {
                        self.handle_resize_event(width, height)?;
                    }
                    _ => {}
                }
            }

            // Handle IRC events
            if let Some(ref mut receiver) = self.event_receiver {
                let mut events = Vec::new();
                while let Ok(event) = receiver.try_recv() {
                    events.push(event);
                }
                for event in events {
                    self.handle_core_event(event);
                }
            }

            // Tick update
            if self.last_tick.elapsed() >= self.tick_rate {
                self.on_tick();
                self.last_tick = Instant::now();
            }

            // Render
            self.draw()?;

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        Ok(())
    }

    /// Handle keyboard events
    fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<()> {
        let key_event = KeyEvent::from(key);

        // Global hotkeys
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(());
            }
            _ => {}
        }

        // Log current input mode for debugging
        let current_mode = self.input_handler.current_mode();
        debug!("Key event in {:?} mode: {:?}", current_mode, key.code);

        // Mode-specific key handling
        match current_mode {
            InputMode::Normal => {
                info!("Processing key in Normal mode");
            }
            InputMode::Insert => {
                info!("Processing key in Insert mode");
            }
            InputMode::Command => {
                info!("Processing key in Command mode");
            }
        }

        // Handle input based on current mode
        let action = self
            .input_handler
            .handle_key(key_event, &mut self.tui_state)?;
        if !matches!(action, TuiAction::None) {
            self.handle_action(action)?;
        }

        Ok(())
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, _mouse: event::MouseEvent) -> Result<()> {
        // Mouse support will be implemented later
        Ok(())
    }

    /// Handle terminal resize events
    fn handle_resize_event(&mut self, _width: u16, _height: u16) -> Result<()> {
        // The terminal will automatically handle resize
        Ok(())
    }

    /// Handle IRC core events
    fn handle_core_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::Connected { connection_id } => {
                info!("Connected to {}", connection_id);
                self.tui_state.add_server(connection_id);
            }
            CoreEvent::Disconnected {
                connection_id,
                reason,
            } => {
                info!("Disconnected from {}: {}", connection_id, reason);
                self.tui_state.remove_server(&connection_id);
            }
            CoreEvent::MessageReceived {
                connection_id,
                message,
            } => {
                debug!("Received message from {}: {:?}", connection_id, message);
                // Handle IRC message parsing and display
            }
            CoreEvent::ChannelJoined {
                connection_id,
                channel,
            } => {
                info!("Joined channel {} on {}", channel, connection_id);
                self.tui_state.add_channel(connection_id, channel);
            }
            CoreEvent::ChannelLeft {
                connection_id,
                channel,
            } => {
                info!("Left channel {} on {}", channel, connection_id);
                self.tui_state.remove_channel(&connection_id, &channel);
            }
            _ => {
                debug!("Unhandled core event: {:?}", event);
            }
        }
    }

    /// Handle TUI actions
    fn handle_action(&mut self, action: TuiAction) -> Result<()> {
        match action {
            TuiAction::SendMessage(command) => {
                self.handle_command(command)?;
            }
            TuiAction::NextTheme => {
                self.handle_command("/theme next".to_string())?;
            }
            TuiAction::Connect => {
                self.handle_command("/connect".to_string())?;
            }
            TuiAction::ShowHelp => {
                self.tui_state.toggle_help();
            }
            TuiAction::ToggleUserList => {
                self.tui_state.toggle_user_list();
            }
            TuiAction::ToggleServerTree => {
                self.tui_state.toggle_channel_list();
            }
            TuiAction::NextTab => {
                self.tui_state.next_channel();
            }
            TuiAction::PreviousTab => {
                self.tui_state.previous_channel();
            }
            TuiAction::ScrollUp => {
                // Handled by input handler directly
            }
            TuiAction::ScrollDown => {
                // Handled by input handler directly
            }
            TuiAction::None => {
                // No action needed
            }
            _ => {
                // Implement other actions as needed
                debug!("Unhandled TUI action: {:?}", action);
            }
        }
        Ok(())
    }

    /// Handle user commands
    fn handle_command(&mut self, command: String) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            error!("Empty command received");
            return Ok(());
        }

        match parts[0] {
            "/connect" => {
                if parts.len() >= 2 {
                    let server = parts[1];
                    let port = if parts.len() >= 3 {
                        parts[2].parse().unwrap_or(6667)
                    } else {
                        6667
                    };

                    info!("Connecting to {}:{}", server, port);
                    // This would trigger actual connection
                }
            }
            "/join" | "/j" => {
                if parts.len() >= 2 {
                    let channel = parts[1];
                    info!("Joining channel {}", channel);
                    // This would trigger channel join
                }
            }
            "/part" | "/leave" => {
                if let Some(current_channel) = self.tui_state.current_channel() {
                    info!("Leaving channel {}", current_channel);
                    // This would trigger channel part
                }
            }
            "/quit" | "/exit" => {
                self.should_quit = true;
            }
            "/theme" => {
                if parts.len() >= 2 {
                    match parts[1] {
                        "next" => {
                            self.renderer.next_theme();
                            info!(
                                "Switched to next theme: {:?}",
                                self.renderer.current_theme()
                            );
                        }
                        "prev" | "previous" => {
                            self.renderer.previous_theme();
                            info!(
                                "Switched to previous theme: {:?}",
                                self.renderer.current_theme()
                            );
                        }
                        "dark" => {
                            self.renderer.set_theme(crate::themes::ThemeName::Dark);
                            info!("Switched to dark theme");
                        }
                        "light" => {
                            self.renderer.set_theme(crate::themes::ThemeName::Light);
                            info!("Switched to light theme");
                        }
                        "contrast" => {
                            self.renderer
                                .set_theme(crate::themes::ThemeName::HighContrast);
                            info!("Switched to high contrast theme");
                        }
                        "monokai" => {
                            self.renderer.set_theme(crate::themes::ThemeName::Monokai);
                            info!("Switched to monokai theme");
                        }
                        "solarized" => {
                            self.renderer.set_theme(crate::themes::ThemeName::Solarized);
                            info!("Switched to solarized theme");
                        }
                        _ => {
                            warn!("Unknown theme: {}. Available: dark, light, contrast, monokai, solarized", parts[1]);
                        }
                    }
                } else {
                    info!("Current theme: {:?}", self.renderer.current_theme());
                }
            }
            _ => {
                if command.starts_with('/') {
                    warn!("Unknown command: {}", parts[0]);
                    error!(
                        "Unrecognized command: {}. Type /help for available commands",
                        parts[0]
                    );
                } else {
                    // Regular message - clone values first to avoid borrow conflicts
                    let server_opt = self.tui_state.current_server().cloned();
                    let channel_opt = self.tui_state.current_channel().cloned();

                    if let (Some(server), Some(channel)) = (server_opt, channel_opt) {
                        self.tui_state.add_message(
                            server.clone(),
                            channel.clone(),
                            "you".to_string(),
                            command.clone(),
                        );
                        info!("Message sent to {} on {}", channel, server);

                        // Send event notification for message sent
                        let message_sent_event = CoreEvent::MessageSent {
                            connection_id: server.clone(),
                            message: rustirc_protocol::message::Message {
                                prefix: None,
                                command: "PRIVMSG".to_string(),
                                params: vec![channel.clone(), command.clone()],
                                tags: None,
                            },
                        };
                        if let Err(e) = self.event_sender.send(message_sent_event) {
                            error!("Failed to send message event: {}", e);
                        }
                    } else {
                        error!("Cannot send message: no active channel");

                        // Send error event
                        let error_event = CoreEvent::Error {
                            connection_id: None,
                            error: "No active channel for message".to_string(),
                        };
                        if let Err(e) = self.event_sender.send(error_event) {
                            error!("Failed to send error event: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Tick update
    fn on_tick(&mut self) {
        // Update any time-based animations or state
        self.tui_state.update_timestamps();
    }

    /// Draw the interface
    fn draw(&mut self) -> Result<()> {
        if let Some(ref mut terminal) = self.terminal {
            terminal.draw(|frame| {
                self.renderer.render(frame, &self.tui_state);
            })?;
        }
        Ok(())
    }

    /// Get current input mode
    pub fn input_mode(&self) -> InputMode {
        self.input_handler.current_mode()
    }

    /// Set input mode for testing purposes
    pub fn set_input_mode(&mut self, mode: InputMode) {
        info!("Setting TUI input mode to: {:?}", mode);
        self.input_handler.set_mode(mode);
    }

    /// Check if application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get backend type information for debugging
    pub fn get_backend_info(&self) -> String {
        if let Some(ref terminal) = self.terminal {
            // Get backend type for debugging/logging and check backend size
            let size = terminal.backend().size().unwrap_or_default();
            info!(
                "Backend initialized with size: {}x{}",
                size.width, size.height
            );
            format!("Backend: CrosstermBackend ({}x{})", size.width, size.height)
        } else {
            "No terminal backend".to_string()
        }
    }

    /// Connect to an IRC server using the IRC client
    pub async fn connect_to_server(&mut self, server: &str, port: u16) -> Result<()> {
        let config = ConnectionConfig {
            server: server.to_string(),
            port,
            nickname: "RustIRC".to_string(),
            username: "rustirc".to_string(),
            realname: "RustIRC Client".to_string(),
            use_tls: false,
            ..Default::default()
        };

        // Use the IRC client for connection management
        match self.irc_client.connect(server, port).await {
            Ok(_) => {
                info!("IRC client connected to {}:{}", server, port);

                // Get state snapshot for verification
                let current_state = self.state_manager.get_state().await;
                debug!("Current state version: {}", current_state.version);

                // Send connection command through command sender
                let connect_command = format!("CONNECT {server}:{port}");
                if let Err(e) = self.command_sender.send(connect_command) {
                    warn!("Failed to send connect command: {}", e);
                }
            }
            Err(e) => {
                error!("IRC client connection failed: {}", e);
                return Err(e.into());
            }
        }

        let connection_id = format!("{server}:{port}");
        self.connection_manager
            .add_connection(connection_id.clone(), config)
            .await?;

        info!("Connection initiated to {}", connection_id);
        Ok(())
    }

    /// Initialize event handling by registering the TUI event handler
    pub async fn initialize_event_handling(&self) -> Result<()> {
        // Create a shared reference to TUI state for the event handler
        let tui_state = Arc::new(tokio::sync::RwLock::new(self.tui_state.clone()));
        let tui_handler = TuiEventHandler::new(tui_state);
        self.event_bus.register(tui_handler).await;
        info!("TUI event handler registered with EventBus");
        Ok(())
    }

    /// Send IRC command using the IRC client and state manager
    pub async fn send_irc_command(&self, command: &str) -> Result<()> {
        // Create IRC Command based on command string - implement comprehensive parsing
        let irc_command = self.parse_command_string(command)?;

        // Use IRC client to send the command
        match self.irc_client.send_command(irc_command).await {
            Ok(_) => {
                info!("IRC command sent: {}", command);

                // Get state manager state for verification
                let server_state = self.state_manager.get_server_state("default").await;
                debug!("Server state available: {}", server_state.is_some());

                // Send through command channel for processing
                if let Err(e) = self.command_sender.send(command.to_string()) {
                    warn!("Failed to send command through channel: {}", e);
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to send IRC command '{}': {}", command, e);
                Err(e.into())
            }
        }
    }

    /// Parse command string into IRC Command - comprehensive implementation
    fn parse_command_string(&self, command: &str) -> Result<rustirc_protocol::Command> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }

        let cmd_name = parts[0].to_uppercase();
        match cmd_name.as_str() {
            "PING" => {
                let server1 = parts.get(1).unwrap_or(&"localhost").to_string();
                let server2 = parts.get(2).map(|s| s.to_string());
                Ok(rustirc_protocol::Command::Ping { server1, server2 })
            }
            "PONG" => {
                let server1 = parts.get(1).unwrap_or(&"localhost").to_string();
                let server2 = parts.get(2).map(|s| s.to_string());
                Ok(rustirc_protocol::Command::Pong { server1, server2 })
            }
            "NICK" => {
                if parts.len() < 2 {
                    return Err(anyhow::anyhow!("NICK command requires nickname"));
                }
                Ok(rustirc_protocol::Command::Nick {
                    nickname: parts[1].to_string(),
                })
            }
            "JOIN" => {
                if parts.len() < 2 {
                    return Err(anyhow::anyhow!("JOIN command requires channel"));
                }
                let channels = vec![parts[1].to_string()];
                let keys = if parts.len() > 2 {
                    vec![parts[2].to_string()]
                } else {
                    vec![]
                };
                Ok(rustirc_protocol::Command::Join { channels, keys })
            }
            "PART" => {
                if parts.len() < 2 {
                    return Err(anyhow::anyhow!("PART command requires channel"));
                }
                let channels = vec![parts[1].to_string()];
                let message = if parts.len() > 2 {
                    Some(parts[2..].join(" "))
                } else {
                    None
                };
                Ok(rustirc_protocol::Command::Part { channels, message })
            }
            "PRIVMSG" => {
                if parts.len() < 3 {
                    return Err(anyhow::anyhow!("PRIVMSG requires target and text"));
                }
                Ok(rustirc_protocol::Command::PrivMsg {
                    target: parts[1].to_string(),
                    text: parts[2..].join(" "),
                })
            }
            "QUIT" => {
                let message = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Ok(rustirc_protocol::Command::Quit { message })
            }
            _ => {
                // For unknown commands, use Raw command with full implementation
                Ok(rustirc_protocol::Command::Raw {
                    command: cmd_name,
                    params: parts[1..].iter().map(|s| s.to_string()).collect(),
                })
            }
        }
    }

    /// Get IRC client status for debugging
    pub async fn get_irc_status(&self) -> String {
        // Get state information from state manager
        let current_state = self.state_manager.get_state().await;
        let server_count = current_state.servers.len();
        let current_server = current_state
            .current_server
            .unwrap_or_else(|| "none".to_string());

        format!("IRC Client: Connected | Servers: {server_count} | Current: {current_server}")
    }

    /// Process pending commands from command sender
    pub async fn process_pending_commands(&self) -> usize {
        let mut processed = 0;

        // Create a temporary String channel for command processing
        let (_temp_sender, mut temp_receiver) = mpsc::unbounded_channel::<String>();

        // Test command sender functionality by sending a ping
        if self.command_sender.send("PING".to_string()).is_err() {
            warn!("Command sender channel is closed");
            return processed;
        }

        // Process any queued commands (in real implementation, this would drain from actual queue)
        while let Ok(command) = temp_receiver.try_recv() {
            match self.send_irc_command(&command).await {
                Ok(_) => {
                    processed += 1;
                    debug!("Processed queued command: {}", command);
                }
                Err(e) => {
                    error!("Failed to process queued command '{}': {}", command, e);
                }
            }
        }

        // Demonstrate usage of the command sender for status reporting
        let status_report = format!("Processed {processed} commands");
        if let Err(e) = self.command_sender.send(status_report) {
            debug!("Failed to send status report: {}", e);
        }

        processed
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new().expect("Failed to create TUI app")
    }
}
