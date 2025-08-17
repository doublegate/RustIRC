//! TUI application
//!
//! Main terminal user interface for RustIRC using ratatui.
//! Features:
//! - Full-screen terminal interface
//! - Vi-like keybindings for navigation
//! - Multiple panes for server list, messages, and user list
//! - Cross-platform terminal support
//! - Efficient text rendering and scrolling

use crate::ui::TuiRenderer;
use crate::input::{InputHandler, KeyEvent, InputMode};
use crate::state::TuiState;
use crate::event_handler::TuiEventHandler;
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
        execute!(
            stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        Ok(())
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        self.terminal = Some(Self::setup_terminal()?);
        
        // Initialize TUI state
        self.tui_state.add_server("freenode.net".to_string());
        self.tui_state.add_channel("freenode.net".to_string(), "#rust".to_string());
        self.tui_state.add_channel("freenode.net".to_string(), "#programming".to_string());
        
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
            (KeyCode::Char('c'), KeyModifiers::CONTROL) |
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(());
            }
            _ => {}
        }
        
        // Handle input based on current mode
        if let Some(command) = self.input_handler.handle_key(key_event, &mut self.tui_state)? {
            self.handle_command(command)?;
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
            CoreEvent::Disconnected { connection_id, reason } => {
                info!("Disconnected from {}: {}", connection_id, reason);
                self.tui_state.remove_server(&connection_id);
            }
            CoreEvent::MessageReceived { connection_id, message } => {
                debug!("Received message from {}: {:?}", connection_id, message);
                // Handle IRC message parsing and display
            }
            CoreEvent::ChannelJoined { connection_id, channel } => {
                info!("Joined channel {} on {}", channel, connection_id);
                self.tui_state.add_channel(connection_id, channel);
            }
            CoreEvent::ChannelLeft { connection_id, channel } => {
                info!("Left channel {} on {}", channel, connection_id);
                self.tui_state.remove_channel(&connection_id, &channel);
            }
            _ => {
                debug!("Unhandled core event: {:?}", event);
            }
        }
    }

    /// Handle user commands
    fn handle_command(&mut self, command: String) -> Result<()> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
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
                            info!("Switched to next theme: {:?}", self.renderer.current_theme());
                        }
                        "prev" | "previous" => {
                            self.renderer.previous_theme();
                            info!("Switched to previous theme: {:?}", self.renderer.current_theme());
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
                            self.renderer.set_theme(crate::themes::ThemeName::HighContrast);
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
                } else {
                    // Regular message
                    if let (Some(server), Some(channel)) = (
                        self.tui_state.current_server(),
                        self.tui_state.current_channel()
                    ) {
                        self.tui_state.add_message(
                            server.clone(),
                            channel.clone(),
                            "you".to_string(),
                            command,
                        );
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

    /// Check if application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Connect to an IRC server
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

        let connection_id = format!("{}:{}", server, port);
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
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new().expect("Failed to create TUI app")
    }
}