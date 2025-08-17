//! Main GUI application
//!
//! This module implements the complete Iced GUI application for RustIRC.
//! Features:
//! - Multi-server IRC client interface
//! - Resizable panels with server tree, message view, and user lists
//! - Comprehensive tab system for channels and private messages
//! - Full IRC message formatting with colors and styles
//! - Theming support with multiple built-in themes
//! - Context menus, dialogs, and platform integration

use crate::theme::{Theme, ThemeType};
use crate::widgets::{
    server_tree::{ServerTree, ServerTreeMessage},
    message_view::{MessageView, MessageViewMessage},
    input_area::{InputArea, InputAreaMessage},
    user_list::{UserList, UserListMessage},
    status_bar::{StatusBar, StatusBarMessage},
    tab_bar::{TabBar, TabBarMessage},
};
use crate::state::{AppState};
use iced::{
    executor, widget::{pane_grid, container, row, column, text},
    Element, Length, Settings, Size, Subscription, Task,
};
use rustirc_core::{
    client::IrcClient,
    connection::{ConnectionConfig, ConnectionManager, ConnectionState},
    events::{Event as CoreEvent, EventBus},
    state::StateManager,
    router::MessageRouter,
};
use rustirc_protocol::{Command as IrcCommand, Message as IrcMessage};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use crate::event_handler::{GuiEventHandler, CoreEventMessage};

/// Main application message types
#[derive(Debug, Clone)]
pub enum Message {
    // Layout and UI messages
    PaneResized(pane_grid::ResizeEvent),
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    
    // Widget messages
    ServerTree(ServerTreeMessage),
    MessageView(MessageViewMessage),
    InputArea(InputAreaMessage),
    UserList(UserListMessage),
    StatusBar(StatusBarMessage),
    TabBar(TabBarMessage),
    
    // Connection management
    ConnectToServer(ConnectionConfig),
    DisconnectFromServer(String),
    ConnectionStateChanged(String, ConnectionState),
    
    // IRC messages
    IrcMessageReceived(String, IrcMessage),
    SendIrcCommand(String, IrcCommand),
    
    // Channel and user management
    JoinChannel(String, String), // server_id, channel
    LeaveChannel(String, String), // server_id, channel
    SwitchToTab(String), // tab_id
    CloseTab(String), // tab_id
    
    // Theme and settings
    ChangeTheme(ThemeType),
    ToggleUserList,
    ToggleServerTree,
    
    // Menu and dialog actions
    ShowConnectDialog,
    ShowPreferencesDialog,
    ShowAboutDialog,
    
    // System events
    CoreEvent(CoreEventMessage),
    Error(String),
}

/// Pane types for the layout system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneType {
    ServerTree,
    MessageArea,
    UserList,
}

/// Layout configuration
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub show_server_tree: bool,
    pub show_user_list: bool,
    pub server_tree_width: f32,
    pub user_list_width: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            show_server_tree: true,
            show_user_list: true,
            server_tree_width: 200.0,
            user_list_width: 150.0,
        }
    }
}

/// Main RustIRC GUI application
pub struct RustIrcGui {
    // Core IRC functionality
    irc_client: Arc<IrcClient>,
    connection_manager: Arc<ConnectionManager>,
    state_manager: Arc<StateManager>,
    event_bus: Arc<EventBus>,
    
    // Application state
    app_state: AppState,
    layout_config: LayoutConfig,
    current_theme: Theme,
    
    // UI components
    panes: pane_grid::State<PaneType>,
    server_tree: ServerTree,
    message_view: MessageView,
    input_area: InputArea,
    user_list: UserList,
    status_bar: StatusBar,
    tab_bar: TabBar,
    
    // Event handling
    command_sender: mpsc::UnboundedSender<(String, IrcCommand)>,
    
    // Dialog states
    show_connect_dialog: bool,
    show_preferences_dialog: bool,
    show_about_dialog: bool,
}

impl RustIrcGui {
    pub fn new() -> (Self, Task<Message>) {
        // Initialize core IRC components
        let event_bus = Arc::new(EventBus::new());
        let state_manager = Arc::new(StateManager::new());
        let connection_manager = Arc::new(ConnectionManager::new(event_bus.clone()));
        
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        // Create message router
        let message_router = Arc::new(MessageRouter::new(
            state_manager.clone(),
            event_bus.clone(),
            command_sender.clone(),
        ));
        
        // Initialize IRC client
        let irc_client = Arc::new(IrcClient::new(
            connection_manager.clone(),
            state_manager.clone(),
            event_bus.clone(),
        ));
        
        // Set up pane layout with the main message area
        let (mut panes, main_pane) = pane_grid::State::new(PaneType::MessageArea);
        
        // For now we'll start with just the message area
        // Later we can add functionality to split panes as needed
        
        let app = Self {
            irc_client,
            connection_manager,
            state_manager,
            event_bus,
            app_state: AppState::new(),
            layout_config: LayoutConfig::default(),
            current_theme: Theme::default(),
            panes,
            server_tree: ServerTree::new(),
            message_view: MessageView::new(),
            input_area: InputArea::new(),
            user_list: UserList::new(),
            status_bar: StatusBar::new(),
            tab_bar: TabBar::new(),
            command_sender,
            show_connect_dialog: false,
            show_preferences_dialog: false,
            show_about_dialog: false,
        };
        
        (app, Task::none())
    }

    fn title(&self) -> String {
        if let Some(current_tab) = self.app_state.current_tab() {
            format!("RustIRC - {}", current_tab.title())
        } else {
            "RustIRC".to_string()
        }
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::PaneResized(event) => {
                self.panes.resize(&event.split, event.ratio);
                Task::none()
            }
            
            Message::PaneClicked(pane) => {
                self.panes.set_focused(pane);
                Task::none()
            }
            
            Message::PaneDragged(event) => {
                self.panes.split(&event.split.axis, &event.split.pane, event.split.state);
                Task::none()
            }
            
            Message::ServerTree(msg) => {
                let command = self.server_tree.update(msg, &mut self.app_state);
                command.map(Message::ServerTree)
            }
            
            Message::MessageView(msg) => {
                let command = self.message_view.update(msg, &mut self.app_state);
                command.map(Message::MessageView)
            }
            
            Message::InputArea(msg) => {
                match msg {
                    InputAreaMessage::SendMessage(text) => {
                        if let Some(current_tab) = self.app_state.current_tab() {
                            if text.starts_with('/') {
                                // Handle command
                                self.handle_user_command(&text[1..]);
                            } else {
                                // Send message
                                let target = current_tab.target().to_string();
                                let command = IrcCommand::PrivMsg {
                                    target,
                                    text,
                                };
                                if let Some(server_id) = current_tab.server_id() {
                                    let _ = self.command_sender.send((server_id.clone(), command));
                                }
                            }
                        }
                        Task::none()
                    }
                    _ => {
                        let command = self.input_area.update(msg, &mut self.app_state);
                        command.map(Message::InputArea)
                    }
                }
            }
            
            Message::UserList(msg) => {
                let command = self.user_list.update(msg, &mut self.app_state);
                command.map(Message::UserList)
            }
            
            Message::StatusBar(msg) => {
                let command = self.status_bar.update(msg, &mut self.app_state);
                command.map(Message::StatusBar)
            }
            
            Message::TabBar(msg) => {
                match msg {
                    TabBarMessage::SwitchTab(tab_id) => {
                        self.app_state.switch_to_tab(&tab_id);
                        Task::none()
                    }
                    TabBarMessage::CloseTab(tab_id) => {
                        self.app_state.close_tab(&tab_id);
                        Task::none()
                    }
                    _ => {
                        let command = self.tab_bar.update(msg, &mut self.app_state);
                        command.map(Message::TabBar)
                    }
                }
            }
            
            Message::ConnectToServer(config) => {
                let server_id = format!("{}:{}", config.server, config.port);
                // Trigger connection through IRC client
                Task::perform(
                    self.connect_to_server_async(server_id, config),
                    |result| match result {
                        Ok(()) => Message::Error("Connected successfully".to_string()),
                        Err(e) => Message::Error(format!("Connection failed: {}", e)),
                    }
                )
            }
            
            Message::DisconnectFromServer(server_id) => {
                Task::perform(
                    self.disconnect_from_server_async(server_id),
                    |result| match result {
                        Ok(()) => Message::Error("Disconnected".to_string()),
                        Err(e) => Message::Error(format!("Disconnect failed: {}", e)),
                    }
                )
            }
            
            Message::ConnectionStateChanged(server_id, state) => {
                self.app_state.update_connection_state(&server_id, state.into());
                Task::none()
            }
            
            Message::IrcMessageReceived(server_id, message) => {
                self.handle_irc_message(&server_id, &message);
                Task::none()
            }
            
            Message::SendIrcCommand(server_id, command) => {
                let _ = self.command_sender.send((server_id, command));
                Task::none()
            }
            
            Message::JoinChannel(server_id, channel) => {
                let command = IrcCommand::Join {
                    channels: vec![channel.clone()],
                    keys: vec![],
                };
                let _ = self.command_sender.send((server_id.clone(), command));
                
                // Create tab for channel
                self.app_state.add_channel_tab(&server_id, &channel);
                Task::none()
            }
            
            Message::LeaveChannel(server_id, channel) => {
                let command = IrcCommand::Part {
                    channels: vec![channel.clone()],
                    message: Some("Leaving".to_string()),
                };
                let _ = self.command_sender.send((server_id.clone(), command));
                
                // Remove tab
                let tab_id = format!("{}:{}", server_id, channel);
                self.app_state.close_tab(&tab_id);
                Task::none()
            }
            
            Message::SwitchToTab(tab_id) => {
                self.app_state.switch_to_tab(&tab_id);
                Task::none()
            }
            
            Message::CloseTab(tab_id) => {
                self.app_state.close_tab(&tab_id);
                Task::none()
            }
            
            Message::ChangeTheme(theme_type) => {
                self.current_theme = Theme::from_type(theme_type);
                Task::none()
            }
            
            Message::ToggleUserList => {
                self.layout_config.show_user_list = !self.layout_config.show_user_list;
                Task::none()
            }
            
            Message::ToggleServerTree => {
                self.layout_config.show_server_tree = !self.layout_config.show_server_tree;
                Task::none()
            }
            
            Message::ShowConnectDialog => {
                self.show_connect_dialog = true;
                Task::none()
            }
            
            Message::ShowPreferencesDialog => {
                self.show_preferences_dialog = true;
                Task::none()
            }
            
            Message::ShowAboutDialog => {
                self.show_about_dialog = true;
                Task::none()
            }
            
            Message::CoreEvent(event) => {
                self.handle_core_event(event)
            }
            
            Message::Error(error) => {
                error!("GUI Error: {}", error);
                // Could show error dialog or status message
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let content = self.build_main_layout();
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Self::Theme {
        self.current_theme.clone()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Subscribe to core events
        Subscription::none() // Will be implemented with actual event subscription
    }
}

impl RustIrcGui {
    /// Run the GUI application
    pub fn run() -> Result<(), iced::Error> {
        let settings = Settings {
            window: iced::window::Settings {
                size: Size::new(1200.0, 800.0),
                min_size: Some(Size::new(800.0, 600.0)),
                ..Default::default()
            },
            ..Default::default()
        };
        
        iced::run(settings, RustIrcGui::new)
    }
    
    /// Build the main application layout
    fn build_main_layout(&self) -> Element<Message> {
        let tab_bar = self.tab_bar.view(&self.app_state).map(Message::TabBar);
        let status_bar = self.status_bar.view(&self.app_state).map(Message::StatusBar);
        
        // Build main content area with pane grid
        let pane_grid = pane_grid::PaneGrid::new(&self.panes, |pane, pane_type, _focused| {
            self.pane_content(*pane_type)
        })
        .on_resize(10, Message::PaneResized)
        .on_click(Message::PaneClicked)
        .on_drag(Message::PaneDragged)
        .height(Length::Fill);
        
        column![
            tab_bar,
            row![
                // Server tree (conditionally shown)
                if self.layout_config.show_server_tree {
                    container(
                        self.server_tree.view(&self.app_state)
                            .map(Message::ServerTree)
                    )
                    .width(Length::Fixed(self.layout_config.server_tree_width))
                    .height(Length::Fill)
                    .into()
                } else {
                    container(text(""))
                        .width(Length::Fixed(0.0))
                        .into()
                },
                
                // Main content area with pane grid
                container(pane_grid)
                    .width(Length::Fill)
                    .height(Length::Fill),
                
                // User list (conditionally shown)
                if self.layout_config.show_user_list {
                    container(
                        self.user_list.view(&self.app_state)
                            .map(Message::UserList)
                    )
                    .width(Length::Fixed(self.layout_config.user_list_width))
                    .height(Length::Fill)
                    .into()
                } else {
                    container(text(""))
                        .width(Length::Fixed(0.0))
                        .into()
                }
            ].height(Length::Fill),
            status_bar
        ]
        .into()
    }
    
    /// Build content for a specific pane
    fn pane_content(&self, pane_type: PaneType) -> Element<Message> {
        match pane_type {
            PaneType::ServerTree => {
                self.server_tree.view(&self.app_state).map(Message::ServerTree)
            }
            PaneType::MessageArea => {
                column![
                    self.message_view.view(&self.app_state)
                        .map(Message::MessageView),
                    self.input_area.view(&self.app_state)
                        .map(Message::InputArea)
                ]
                .into()
            }
            PaneType::UserList => {
                self.user_list.view(&self.app_state).map(Message::UserList)
            }
        }
    }
    
    /// Handle IRC messages received from the core
    fn handle_irc_message(&mut self, server_id: &str, message: &IrcMessage) {
        match message.command.as_str() {
            "PRIVMSG" | "NOTICE" => {
                if message.params.len() >= 2 {
                    let target = &message.params[0];
                    let text = &message.params[1];
                    
                    // Determine tab ID (channel or private message)
                    let tab_id = if target.starts_with('#') || target.starts_with('&') {
                        format!("{}:{}", server_id, target)
                    } else {
                        // Private message - create tab with sender nick
                        if let Some(ref prefix) = message.prefix {
                            if let Some(nick) = prefix.split('!').next() {
                                format!("{}:{}", server_id, nick)
                            } else {
                                format!("{}:unknown", server_id)
                            }
                        } else {
                            format!("{}:server", server_id)
                        }
                    };
                    
                    // Ensure tab exists
                    if !self.app_state.has_tab(&tab_id) {
                        if target.starts_with('#') || target.starts_with('&') {
                            self.app_state.add_channel_tab(server_id, target);
                        } else {
                            if let Some(ref prefix) = message.prefix {
                                if let Some(nick) = prefix.split('!').next() {
                                    self.app_state.add_private_tab(server_id, nick);
                                }
                            }
                        }
                    }
                    
                    // Add message to tab
                    self.app_state.add_message_to_tab(&tab_id, message.clone());
                }
            }
            "JOIN" => {
                if !message.params.is_empty() {
                    let channel = &message.params[0];
                    let tab_id = format!("{}:{}", server_id, channel);
                    
                    if !self.app_state.has_tab(&tab_id) {
                        self.app_state.add_channel_tab(server_id, channel);
                    }
                    
                    // Add user to channel
                    if let Some(ref prefix) = message.prefix {
                        if let Some(nick) = prefix.split('!').next() {
                            self.app_state.add_user_to_channel(&tab_id, nick);
                        }
                    }
                }
            }
            "PART" | "QUIT" => {
                // Remove user from channels or close tabs
                if let Some(ref prefix) = message.prefix {
                    if let Some(nick) = prefix.split('!').next() {
                        if message.command == "PART" && !message.params.is_empty() {
                            let channel = &message.params[0];
                            let tab_id = format!("{}:{}", server_id, channel);
                            self.app_state.remove_user_from_channel(&tab_id, nick);
                        } else {
                            // QUIT - remove from all channels
                            self.app_state.remove_user_from_server(server_id, nick);
                        }
                    }
                }
            }
            "353" => {
                // Names reply
                if message.params.len() >= 3 {
                    let channel = &message.params[2];
                    let tab_id = format!("{}:{}", server_id, channel);
                    
                    if let Some(names) = message.params.last() {
                        for name in names.split_whitespace() {
                            let nick = name.trim_start_matches(['@', '+', '%', '&', '~']);
                            self.app_state.add_user_to_channel(&tab_id, nick);
                        }
                    }
                }
            }
            _ => {
                // Handle other message types
                debug!("Unhandled IRC message: {} {:?}", message.command, message.params);
            }
        }
    }
    
    /// Handle user commands (slash commands)
    fn handle_user_command(&mut self, command_text: &str) {
        let parts: Vec<&str> = command_text.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }
        
        let command = parts[0].to_lowercase();
        let args = &parts[1..];
        
        match command.as_str() {
            "connect" => {
                if args.len() >= 1 {
                    let server = args[0];
                    let port = if args.len() >= 2 {
                        args[1].parse().unwrap_or(6667)
                    } else {
                        6667
                    };
                    
                    let config = ConnectionConfig {
                        server: server.to_string(),
                        port,
                        ..Default::default()
                    };
                    
                    // This would trigger the connect message
                    // For now, just log it
                    info!("User command: connect to {}:{}", server, port);
                }
            }
            "join" | "j" => {
                if !args.is_empty() {
                    let channel = args[0];
                    if let Some(current_tab) = self.app_state.current_tab() {
                        if let Some(server_id) = current_tab.server_id() {
                            info!("User command: join {}", channel);
                            // This would trigger the join message
                        }
                    }
                }
            }
            "part" | "leave" => {
                if let Some(current_tab) = self.app_state.current_tab() {
                    if current_tab.is_channel() {
                        let reason = if !args.is_empty() {
                            Some(args.join(" "))
                        } else {
                            None
                        };
                        info!("User command: part current channel with reason: {:?}", reason);
                    }
                }
            }
            "quit" => {
                let reason = if !args.is_empty() {
                    Some(args.join(" "))
                } else {
                    Some("RustIRC".to_string())
                };
                info!("User command: quit with reason: {:?}", reason);
                // This would trigger application quit
            }
            _ => {
                warn!("Unknown command: {}", command);
            }
        }
    }
    
    /// Handle core events from the IRC engine
    fn handle_core_event(&mut self, event: CoreEventMessage) -> Task<Message> {
        match event {
            CoreEventMessage::Connected { connection_id } => {
                info!("Connected to {}", connection_id);
                self.app_state.add_server(&connection_id);
                Task::none()
            }
            CoreEventMessage::Disconnected { connection_id, reason } => {
                info!("Disconnected from {}: {}", connection_id, reason);
                self.app_state.remove_server(&connection_id);
                Task::none()
            }
            CoreEventMessage::MessageReceived { connection_id, message } => {
                self.handle_irc_message(&connection_id, &message);
                Task::none()
            }
            CoreEventMessage::MessageSent { connection_id, message } => {
                debug!("Sent message to {}: {}", connection_id, message);
                Task::none()
            }
            CoreEventMessage::ChannelJoined { connection_id, channel } => {
                info!("Joined channel {} on {}", channel, connection_id);
                self.app_state.add_channel_tab(&connection_id, &channel);
                Task::none()
            }
            CoreEventMessage::ChannelLeft { connection_id, channel } => {
                info!("Left channel {} on {}", channel, connection_id);
                let tab_id = format!("{}:{}", connection_id, channel);
                self.app_state.close_tab(&tab_id);
                Task::none()
            }
            CoreEventMessage::UserJoined { connection_id, channel, user } => {
                debug!("User {} joined {} on {}", user, channel, connection_id);
                let tab_id = format!("{}:{}", connection_id, channel);
                self.app_state.add_user_to_channel(&tab_id, &user);
                Task::none()
            }
            CoreEventMessage::UserLeft { connection_id, channel, user } => {
                debug!("User {} left {} on {}", user, channel, connection_id);
                let tab_id = format!("{}:{}", connection_id, channel);
                self.app_state.remove_user_from_channel(&tab_id, &user);
                Task::none()
            }
            CoreEventMessage::NickChanged { connection_id, old_nick, new_nick } => {
                info!("Nick changed from {} to {} on {}", old_nick, new_nick, connection_id);
                self.app_state.handle_nick_change(&connection_id, &old_nick, &new_nick);
                Task::none()
            }
            CoreEventMessage::TopicChanged { connection_id, channel, topic } => {
                info!("Topic changed in {} on {}: {}", channel, connection_id, topic);
                let tab_id = format!("{}:{}", connection_id, channel);
                self.app_state.set_channel_topic(&tab_id, &topic);
                Task::none()
            }
            CoreEventMessage::Error { connection_id, error } => {
                error!("IRC error on {:?}: {}", connection_id, error);
                Task::done(Message::Error(format!("IRC Error: {}", error)))
            }
            CoreEventMessage::StateChanged { connection_id, state } => {
                debug!("Connection state changed for {}: {:?}", connection_id, state);
                self.app_state.update_connection_state(&connection_id, state.into());
                Task::none()
            }
        }
    }
    
    /// Async helper for connecting to server
    async fn connect_to_server_async(
        &self,
        server_id: String,
        config: ConnectionConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = self.connection_manager
            .add_connection(server_id.clone(), config)
            .await?;
        
        // Start connection (this would be done in the background)
        info!("Connection request for {} queued", server_id);
        Ok(())
    }
    
    /// Async helper for disconnecting from server
    async fn disconnect_from_server_async(
        &self,
        server_id: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(connection) = self.connection_manager.get_connection(&server_id).await {
            connection.disconnect().await?;
        }
        Ok(())
    }
    
    /// Set up event subscription to forward core events to GUI messages
    pub fn subscription(&self) -> Subscription<Message> {
        // For now, return an empty subscription
        // In a real implementation, this would create a subscription
        // that listens to the EventBus and forwards events
        Subscription::none()
    }
    
    /// Initialize event handling by registering the GUI event handler
    pub async fn initialize_event_handling(&self, message_sender: mpsc::UnboundedSender<Message>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let gui_handler = GuiEventHandler::new(message_sender);
        self.event_bus.register(gui_handler).await;
        info!("GUI event handler registered with EventBus");
        Ok(())
    }
}

impl Default for RustIrcGui {
    fn default() -> Self {
        let (app, _) = Self::new();
        app
    }
}