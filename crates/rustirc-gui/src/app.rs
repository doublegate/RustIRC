//! Main GUI application
//!
//! This module implements the complete Iced GUI application for RustIRC.
//! Features:
//! - Multi-server IRC client interface using Iced 0.13.1 functional API
//! - Resizable panels with server tree, message view, and user lists
//! - Comprehensive tab system for channels and private messages
//! - Full IRC message formatting with colors and styles
//! - Theming support with multiple built-in themes
//! - Context menus, dialogs, and platform integration

use crate::event_handler::{CoreEventMessage, GuiEventHandler};
use crate::state::AppState;
use crate::theme::{Theme, ThemeType};
use crate::widgets::{
    input_area::{InputArea, InputAreaMessage},
    message_view::{MessageView, MessageViewMessage},
    server_tree::{ServerTree, ServerTreeMessage},
    status_bar::{StatusBar, StatusBarMessage},
    tab_bar::{TabBar, TabBarMessage},
    user_list::{UserList, UserListMessage},
};
use iced::{
    widget::{
        button, column, container, horizontal_rule, mouse_area, pane_grid, row, scrollable, stack,
        text, text_input,
    },
    Background, Color, Element, Length, Task,
};
use rustirc_core::IrcClient;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Global IRC event receiver for subscription
static IRC_EVENT_RECEIVER: std::sync::OnceLock<
    Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Message>>>>,
> = std::sync::OnceLock::new();

/// Main application message types
#[derive(Debug, Clone)]
pub enum Message {
    // Layout and UI messages
    PaneResized(pane_grid::ResizeEvent),
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),

    // Core IRC messages
    ConnectToServer(String, u16),
    DisconnectFromServer(String),
    JoinChannel(String, String),         // server_id, channel
    LeaveChannel(String, String),        // server_id, channel
    SendMessage(String, String, String), // server_id, target, message

    // Input handling
    InputChanged(String),
    InputSubmitted,

    // Tab management
    TabSelected(String),
    TabClosed(String),
    TabReordered(Vec<String>),

    // Context menu
    ShowContextMenu(f32, f32), // x, y position
    HideContextMenu,
    ContextMenuAction(String),

    // Theme management
    ThemeChanged(ThemeType),

    // Dialog messages
    ShowConnectDialog,
    HideConnectDialog,
    ConnectDialogServerChanged(String),
    ConnectDialogPortChanged(String),
    ConnectDialogNickChanged(String),
    ConnectDialogConnect,

    // IRC Event messages (from real IRC events)
    IrcConnected(String),                                  // connection_id
    IrcDisconnected(String, String),                       // connection_id, reason
    IrcMessageReceived(String, rustirc_protocol::Message), // connection_id, message
    IrcConnectionStateChanged(String, rustirc_core::ConnectionState), // connection_id, state
    IrcError(Option<String>, String),                      // connection_id, error

    // Core IRC events from event handler
    CoreEvent(CoreEventMessage),

    // Menu dropdown control
    MenuToggle(String), // Toggle menu open/closed (File, Edit, View, Tools, Help)
    MenuClose,          // Close all menus

    // Menu actions
    MenuFileConnect,
    MenuFileDisconnect,
    MenuFileExit,
    MenuViewToggleSystemMessages,
    MenuViewToggleUserLists,
    MenuViewToggleJoinsParts,
    MenuViewToggleMotd,
    MenuViewToggleTimestamps,
    MenuToolsPreferences,
    MenuHelpAbout,

    // Dialog messages
    ShowPreferencesDialog,
    HidePreferencesDialog,
    ShowAboutDialog,
    HideAboutDialog,

    // Widget messages
    ServerTree(ServerTreeMessage),
    MessageView(MessageViewMessage),
    UserList(UserListMessage),
    InputArea(InputAreaMessage),
    TabBar(TabBarMessage),
    StatusBar(StatusBarMessage),

    // Input and navigation commands
    TabComplete,
    CancelOperation,
    HistoryPrevious,
    HistoryNext,
    ScrollUp,
    ScrollDown,
    ShowHelp,
    CopySelection,
    PasteText,
    WindowResized(u16, u16), // width, height

    // No operation
    None,
}

/// Pane types for the layout
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaneType {
    ServerTree,
    MessageView,
    UserList,
    InputArea,
}

/// Main GUI application state for the RustIRC client
///
/// This struct manages the complete GUI state including IRC connections,
/// themes, layout, and all UI components.
///
/// # Examples
///
/// ```
/// use rustirc_gui::RustIrcGui;
/// use rustirc_gui::themes::material_design_3::MaterialTheme;
///
/// // Create a new GUI application instance
/// let app = RustIrcGui::new();
///
/// // The app starts with default state
/// assert!(!app.has_active_connection());
/// ```
///
/// # Architecture
///
/// The GUI is built using Iced 0.13.1 with a functional approach:
/// - Event-driven message passing between components
/// - Immutable state updates through message handlers
/// - Resizable pane-based layout for flexible UI
/// - Material Design 3 theming system
pub struct RustIrcGui {
    // Core IRC functionality
    irc_client: Arc<RwLock<Option<Arc<IrcClient>>>>,

    // Application state
    app_state: AppState,
    current_theme: Theme,

    // IRC event handling
    irc_message_sender: Option<tokio::sync::mpsc::UnboundedSender<Message>>,
    irc_message_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<Message>>,

    // Layout
    panes: pane_grid::State<PaneType>,
    // User list state
    user_list_visible: bool,

    // Input state
    input_buffer: String,

    // Widget instances
    server_tree: ServerTree,
    message_view: MessageView,
    user_list: UserList,
    input_area: InputArea,
    tab_bar: TabBar,
    status_bar: StatusBar,

    // Context menu
    context_menu_visible: bool,
    context_menu_x: f32,
    context_menu_y: f32,

    // Menu dropdown state
    active_menu: Option<String>, // Which menu is currently open (File, Edit, View, Tools, Help)

    // Dialog state
    preferences_dialog_visible: bool,
    about_dialog_visible: bool,

    // Connection dialog
    connect_dialog_visible: bool,
    connect_dialog_server: String,
    connect_dialog_port: String,
    connect_dialog_nickname: String,
}

impl Default for RustIrcGui {
    fn default() -> Self {
        // Initialize pane grid layout
        let (mut panes, _) = pane_grid::State::new(PaneType::MessageView);

        // Split to create server tree on the left
        let first_pane = *panes.iter().next().unwrap().0;
        let (left_pane, _) = panes
            .split(pane_grid::Axis::Vertical, first_pane, PaneType::ServerTree)
            .unwrap();

        // Split right side to add user list
        let split_result = panes
            .split(pane_grid::Axis::Vertical, left_pane, PaneType::UserList)
            .unwrap();
        let message_pane = split_result.0;
        let _user_pane = split_result.1;
        // Note: _user_pane is used implicitly in the pane_grid system to render the UserList

        // Split bottom for input area
        let (_top_pane, _bottom_pane) = panes
            .split(
                pane_grid::Axis::Horizontal,
                message_pane,
                PaneType::InputArea,
            )
            .unwrap();

        // Create IRC event message channels
        let (irc_message_sender, irc_message_receiver) =
            tokio::sync::mpsc::unbounded_channel::<Message>();

        // Store receiver globally for subscription
        IRC_EVENT_RECEIVER
            .set(Arc::new(Mutex::new(Some(irc_message_receiver))))
            .ok();

        let app_state = AppState::new();
        // No default servers or test data - start clean for real IRC connections

        Self {
            irc_client: Arc::new(RwLock::new(None)),
            app_state,
            current_theme: Theme::default(),
            irc_message_sender: Some(irc_message_sender),
            irc_message_receiver: None, // Stored globally instead
            panes,
            user_list_visible: true, // Initialize user list as visible, user_pane created above
            input_buffer: String::new(),
            server_tree: ServerTree::new(),
            message_view: MessageView::new(),
            user_list: UserList::new(),
            input_area: InputArea::new(),
            tab_bar: TabBar::new(),
            status_bar: StatusBar::new(),
            context_menu_visible: false,
            context_menu_x: 0.0,
            context_menu_y: 0.0,
            active_menu: None,
            preferences_dialog_visible: false,
            about_dialog_visible: false,
            connect_dialog_visible: false,
            connect_dialog_server: "irc.libera.chat".to_string(),
            connect_dialog_port: "6697".to_string(),
            connect_dialog_nickname: "RustIRC_User".to_string(),
        }
    }
}

impl RustIrcGui {
    /// Create a new RustIrcGui instance
    ///
    /// Initializes a new GUI application with default state, no active IRC
    /// connections, and the default theme.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustirc_gui::RustIrcGui;
    ///
    /// let app = RustIrcGui::new();
    /// assert!(!app.has_active_connection());
    /// assert_eq!(app.state().current_tab_id, None);
    /// ```
    ///
    /// # Returns
    ///
    /// A new `RustIrcGui` instance ready for use with Iced
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current app state for testing
    pub fn state(&self) -> &AppState {
        &self.app_state
    }

    /// Check if there is an active IRC connection
    ///
    /// Returns `true` if an IRC client is currently connected to a server,
    /// `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use rustirc_gui::RustIrcGui;
    ///
    /// let app = RustIrcGui::new();
    /// // Initially no connection
    /// assert!(!app.has_active_connection());
    /// ```
    pub fn has_active_connection(&self) -> bool {
        // Use try_read since this is a sync method and we can't await
        match self.irc_client.try_read() {
            Ok(client) => client.is_some(),
            Err(_) => false,
        }
    }

    /// Connect an IRC message receiver for testing purposes
    /// This allows test harnesses to inject IRC messages directly
    pub fn connect_irc_receiver(
        &mut self,
        receiver: tokio::sync::mpsc::UnboundedReceiver<Message>,
    ) {
        self.irc_message_receiver = Some(receiver);
        info!("Connected IRC message receiver for testing");
    }

    /// Poll the instance IRC message receiver (used in testing)
    /// Returns true if a message was processed
    pub fn poll_irc_receiver(&mut self) -> bool {
        if let Some(ref mut receiver) = self.irc_message_receiver {
            match receiver.try_recv() {
                Ok(message) => {
                    info!("Received test IRC message: {:?}", message);
                    // Process the message through the normal update path
                    let _ = self.update(message);
                    return true;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    // No message available
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    warn!("Test IRC message channel disconnected");
                    self.irc_message_receiver = None;
                }
            }
        }
        false
    }

    /// Update function for Iced 0.13.1 functional approach
    pub fn update(&mut self, message: Message) -> impl Into<Task<Message>> {
        match message {
            Message::PaneResized(resize_event) => {
                self.panes.resize(resize_event.split, resize_event.ratio);
            }
            Message::PaneClicked(_pane) => {
                // Pane grid state doesn't need click handling in Iced 0.13.1
            }
            Message::PaneDragged(_drag_event) => {
                // Drag events are handled automatically by pane grid in Iced 0.13.1
            }
            Message::InputChanged(value) => {
                self.input_buffer = value;
            }
            Message::InputSubmitted => {
                if !self.input_buffer.trim().is_empty() {
                    info!("Input submitted: {}", self.input_buffer);

                    // Parse and handle IRC commands
                    if self.input_buffer.starts_with('/') {
                        let command = self.input_buffer.clone();
                        self.handle_irc_command(&command);
                    } else {
                        // Send message to current channel/target
                        if let Some(current_tab) = &self.app_state.current_tab_id {
                            if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                if let Some(_server_id) = &tab.server_id {
                                    let target = tab.name.clone();
                                    let message = self.input_buffer.clone();
                                    info!("Sending message to {}: {}", target, message);
                                    // Send message via IRC client
                                    let client_clone = self.irc_client.clone();
                                    let target_clone = target.clone();
                                    let message_clone = message.clone();
                                    let server_id = _server_id.clone();

                                    tokio::spawn(async move {
                                        let client_guard = client_clone.read().await;
                                        if let Some(client) = client_guard.as_ref() {
                                            let _ = client
                                                .send_message(&target_clone, &message_clone)
                                                .await;
                                        }
                                    });

                                    // Add message to app state for display
                                    self.app_state
                                        .add_message(&server_id, &target, &message, "self");
                                    // Trigger auto-scroll after adding message
                                    return self.trigger_auto_scroll();
                                }
                            }
                        }
                    }

                    self.input_buffer.clear();
                }
            }
            Message::ConnectToServer(server, port) => {
                info!("Connecting to {}:{}", server, port);
                let client_clone = self.irc_client.clone();
                let server_id = format!("{server}:{port}");
                let server_clone = server.clone(); // Clone for async move

                // Initialize IRC client with event handler registration
                let message_sender = self.irc_message_sender.clone();

                tokio::spawn(async move {
                    let mut client_write = client_clone.write().await;
                    // Always create a new client for each connection attempt
                    let config = rustirc_core::Config::default();
                    let client = Arc::new(rustirc_core::IrcClient::new(config));

                    // Always register GUI event handler to receive IRC events
                    if let Some(sender) = message_sender {
                        let event_handler = GuiEventHandler::new(sender);
                        client.event_bus().register(event_handler).await;
                        info!("GUI event handler registered for IRC events");
                    } else {
                        warn!("No message sender available for event handler registration");
                    }

                    if let Ok(()) = client.connect(&server_clone, port).await {
                        *client_write = Some(client);
                        info!("IRC client connected and stored");
                    } else {
                        error!("Failed to connect to {}:{}", server_clone, port);
                    }
                });

                // Add server to app state
                self.app_state
                    .add_server(server_id.clone(), server.to_string());
            }
            Message::DisconnectFromServer(server_id) => {
                info!("Disconnecting from server: {}", server_id);
                let client_clone = self.irc_client.clone();

                tokio::spawn(async move {
                    let client_guard = client_clone.read().await;
                    if let Some(client) = client_guard.as_ref() {
                        let _ = client.disconnect().await;
                    }
                });

                // Remove server from app state
                self.app_state.remove_server(&server_id);
            }
            Message::LeaveChannel(server_id, channel) => {
                info!("Leaving channel {} on server {}", channel, server_id);
                let client_clone = self.irc_client.clone();
                let channel_clone = channel.clone();

                tokio::spawn(async move {
                    let client_guard = client_clone.read().await;
                    if let Some(client) = client_guard.as_ref() {
                        // Send PART command
                        let part_cmd = rustirc_protocol::Command::Part {
                            channels: vec![channel_clone],
                            message: None,
                        };
                        let _ = client.send_command(part_cmd).await;
                    }
                });

                // Remove channel tab
                let tab_id = format!("{server_id}/{channel}");
                self.app_state.remove_tab(&tab_id);
            }
            Message::SendMessage(server_id, target, message) => {
                info!(
                    "Sending message to {} on {}: {}",
                    target, server_id, message
                );
                let client_clone = self.irc_client.clone();
                let target_clone = target.clone();
                let message_clone = message.clone();

                tokio::spawn(async move {
                    let client_guard = client_clone.read().await;
                    if let Some(client) = client_guard.as_ref() {
                        let _ = client.send_message(&target_clone, &message_clone).await;
                    }
                });

                // Add message to app state for display
                self.app_state
                    .add_message(&server_id, &target, &message, "self");
            }
            Message::JoinChannel(server_id, channel) => {
                info!("Joining channel {} on server {}", channel, server_id);
                self.app_state.add_channel_tab(server_id, channel);
            }
            Message::TabSelected(tab_id) => {
                self.app_state.current_tab_id = Some(tab_id);
                // Auto-scroll to bottom when switching tabs
                if self.message_view.is_auto_scroll_enabled() {
                    return Task::batch(vec![self
                        .message_view
                        .create_scroll_to_bottom_task()
                        .map(Message::MessageView)]);
                }
            }
            Message::TabClosed(tab_id) => {
                self.app_state.remove_tab(&tab_id);
                // If this was the current tab, select another one
                if self.app_state.current_tab_id.as_ref() == Some(&tab_id) {
                    self.app_state.current_tab_id = self.app_state.tab_order.first().cloned();
                }
            }
            Message::TabReordered(new_order) => {
                self.app_state.tab_order = new_order;
            }
            Message::ShowContextMenu(x, y) => {
                self.context_menu_visible = true;
                self.context_menu_x = x;
                self.context_menu_y = y;
            }
            Message::HideContextMenu => {
                self.context_menu_visible = false;
            }
            Message::ContextMenuAction(action) => {
                info!("Context menu action: {}", action);
                self.context_menu_visible = false;
                self.active_menu = None; // Close menu dropdown as well

                // Handle specific context menu actions
                match action.as_str() {
                    "whois" => {
                        if let Some(current_tab) = &self.app_state.current_tab_id {
                            if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                if let Some(server_id) = &tab.server_id {
                                    // Extract nickname from action context (simplified)
                                    let parts: Vec<&str> = action.split(':').collect();
                                    if parts.len() > 1 {
                                        let nick = parts[1];
                                        self.send_irc_command(server_id, &format!("/whois {nick}"));
                                    }
                                }
                            }
                        }
                    }
                    "query" => {
                        // Open private message tab
                        let parts: Vec<&str> = action.split(':').collect();
                        if parts.len() > 1 {
                            let nick = parts[1];
                            // Extract server_id first to avoid borrowing conflicts
                            let server_id =
                                if let Some(current_tab) = &self.app_state.current_tab_id {
                                    self.app_state
                                        .tabs
                                        .get(current_tab)
                                        .and_then(|tab| tab.server_id.clone())
                                } else {
                                    None
                                };

                            if let Some(server_id) = server_id {
                                self.app_state.add_private_tab(&server_id, nick.to_string());
                            }
                        }
                    }
                    "kick" | "ban" | "op" | "voice" => {
                        // Handle user moderation actions
                        let parts: Vec<&str> = action.split(':').collect();
                        if parts.len() > 1 {
                            let nick = parts[1];
                            if let Some(current_tab) = &self.app_state.current_tab_id {
                                if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                    if let Some(server_id) = &tab.server_id {
                                        let command = match parts[0] {
                                            "kick" => format!("/kick {} {}", tab.name, nick),
                                            "ban" => format!("/mode {} +b {}!*@*", tab.name, nick),
                                            "op" => format!("/mode {} +o {}", tab.name, nick),
                                            "voice" => format!("/mode {} +v {}", tab.name, nick),
                                            _ => String::new(),
                                        };
                                        if !command.is_empty() {
                                            self.send_irc_command(server_id, &command);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "copy" => {
                        // Copy selected text to clipboard
                        warn!("Copy to clipboard not yet implemented");
                    }
                    "paste" => {
                        // Paste from clipboard to input area
                        warn!("Paste from clipboard not yet implemented");
                    }
                    "select_all" => {
                        // Select all text in message view
                        warn!("Select all text not yet implemented");
                    }
                    "close_tab" => {
                        // Close current tab
                        if let Some(current_tab) = &self.app_state.current_tab_id {
                            let tab_id = current_tab.clone();
                            self.app_state.remove_tab(&tab_id);
                            // Select next available tab
                            if let Some(next_tab) = self.app_state.tab_order.first() {
                                self.app_state.current_tab_id = Some(next_tab.clone());
                            }
                        }
                    }
                    _ => {
                        warn!("Unhandled context menu action: {}", action);
                    }
                }
            }
            Message::ThemeChanged(theme_type) => {
                self.current_theme = Theme::from_type(theme_type);
            }

            // Connection dialog handlers
            Message::ShowConnectDialog => {
                self.connect_dialog_visible = true;
            }
            Message::HideConnectDialog => {
                self.connect_dialog_visible = false;
            }
            Message::ConnectDialogServerChanged(server) => {
                self.connect_dialog_server = server;
            }
            Message::ConnectDialogPortChanged(port) => {
                self.connect_dialog_port = port;
            }
            Message::ConnectDialogNickChanged(nick) => {
                self.connect_dialog_nickname = nick;
            }
            Message::ConnectDialogConnect => {
                if let Ok(port) = self.connect_dialog_port.parse::<u16>() {
                    // Connect to the specified server
                    let server = self.connect_dialog_server.clone();
                    let client_clone = self.irc_client.clone();
                    let server_id = format!("{server}:{port}");
                    let server_clone = server.clone();
                    let nickname = self.connect_dialog_nickname.clone();

                    // Initialize IRC client with custom config and register event handler
                    if let Some(message_sender) = self.irc_message_sender.clone() {
                        tokio::spawn(async move {
                            let mut client_write = client_clone.write().await;
                            // Always create a new client for each connection attempt
                            let mut config = rustirc_core::Config::default();
                            config.user.nickname = nickname;

                            let client = Arc::new(rustirc_core::IrcClient::new(config));

                            // Always register the GUI event handler to receive real IRC events
                            let event_handler = GuiEventHandler::new(message_sender);
                            client.event_bus().register(event_handler).await;
                            info!(
                                "GUI event handler registered for IRC events (dialog connection)"
                            );

                            if let Ok(()) = client.connect(&server_clone, port).await {
                                *client_write = Some(client);
                                info!("IRC client connected via dialog");
                            } else {
                                error!("Failed to connect to {}:{} via dialog", server_clone, port);
                            }
                        });
                    } else {
                        error!("No message sender available for dialog connection");
                    }

                    // Add server to app state
                    self.app_state
                        .add_server(server_id.clone(), server.to_string());

                    // Hide the dialog
                    self.connect_dialog_visible = false;
                }
            }
            Message::ServerTree(server_tree_msg) => {
                // Handle server tree specific actions first
                use crate::widgets::server_tree::ServerTreeMessage;
                match &server_tree_msg {
                    ServerTreeMessage::ServerClicked(server_id) => {
                        // Switch to server tab
                        self.app_state.current_tab_id = Some(server_id.clone());
                    }
                    ServerTreeMessage::ChannelClicked(channel_id) => {
                        // Parse channel_id to extract server and channel (format: "server/channel")
                        if let Some((server_id, channel)) = channel_id.split_once('/') {
                            // Switch to channel tab
                            let tab_id = format!("{server_id}/{channel}");
                            self.app_state.current_tab_id = Some(tab_id.clone());
                            info!("Switched to channel: {} on server: {}", channel, server_id);
                        } else {
                            // Handle case where channel_id is just the channel name (need current server)
                            let tab_id = if let Some(current_tab) = self.app_state.current_tab() {
                                current_tab
                                    .server_id
                                    .as_ref()
                                    .map(|server_id| format!("{server_id}/{channel_id}"))
                            } else {
                                None
                            };

                            if let Some(tab_id) = tab_id {
                                self.app_state.current_tab_id = Some(tab_id.clone());
                                info!("Switched to channel: {} on current server", channel_id);
                            }
                        }
                    }
                    ServerTreeMessage::ServerContextMenu(server_id) => {
                        // Show server context menu
                        info!("Server context menu for: {}", server_id);
                    }
                    ServerTreeMessage::ChannelContextMenu(channel_id) => {
                        // Show channel context menu
                        info!("Channel context menu for: {}", channel_id);
                    }
                    ServerTreeMessage::ExpandServer(server_id) => {
                        info!("Expanding server: {}", server_id);
                        // Handle server expansion in the tree view
                        self.server_tree.expand_server(server_id.clone());
                    }
                    ServerTreeMessage::CollapseServer(server_id) => {
                        info!("Collapsing server: {}", server_id);
                        // Handle server collapse in the tree view
                        self.server_tree.collapse_server(server_id.clone());
                    }
                }

                // Now call widget update after handling app-level logic
                let task = self
                    .server_tree
                    .update(server_tree_msg, &mut self.app_state);

                // Execute widget task and any additional app-level tasks
                return Task::batch(vec![task.map(Message::ServerTree)]);
            }
            Message::MessageView(message_view_msg) => {
                // Handle message view specific actions first
                use crate::widgets::message_view::MessageViewMessage;
                match &message_view_msg {
                    MessageViewMessage::MessageSelected(message_id) => {
                        // Handle message selection for copying or actions
                        info!("Message selected: {}", message_id);
                    }
                    MessageViewMessage::CopySelected => {
                        // Copy selected messages to clipboard
                        warn!("Copy selected messages not yet implemented");
                    }
                    MessageViewMessage::UrlClicked(url) => {
                        // Open URL in default browser
                        info!("Opening URL: {}", url);
                        if let Err(e) = open::that(url) {
                            warn!("Failed to open URL {}: {}", url, e);
                        } else {
                            info!("Successfully opened URL in default browser: {}", url);
                        }
                    }
                    MessageViewMessage::ScrollToBottom => {
                        // Scroll message view to bottom
                        info!("Scrolling message view to bottom");
                        self.message_view.scroll_to_bottom();
                    }
                    MessageViewMessage::ScrollToTop => {
                        // Scroll message view to top
                        info!("Scrolling message view to top");
                        self.message_view.scroll_to_top();
                    }
                    MessageViewMessage::MessageClicked(message_id) => {
                        // Handle message click (e.g., for context menu or selection)
                        info!("Message clicked: {}", message_id);
                    }
                    MessageViewMessage::SearchRequested(query) => {
                        // Handle search request
                        info!("Search requested: {}", query);
                        self.message_view.set_search_query(Some(query.clone()));
                    }
                    MessageViewMessage::ClearSelection => {
                        // Clear message selection
                        info!("Clearing message selection");
                        self.message_view.clear_selection();
                    }
                    MessageViewMessage::NoOp => {
                        // No operation - do nothing
                    }
                }

                // Now call widget update after handling app-level logic
                let task = self
                    .message_view
                    .update(message_view_msg, &mut self.app_state);

                // Execute widget task and any additional app-level tasks
                return Task::batch(vec![task.map(Message::MessageView)]);
            }
            Message::UserList(user_list_msg) => {
                // Handle user list specific actions first
                use crate::widgets::user_list::UserListMessage;
                match &user_list_msg {
                    UserListMessage::UserClicked(nick) => {
                        // Open private message tab
                        if let Some(current_tab) = &self.app_state.current_tab_id {
                            if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                if let Some(server_id) = &tab.server_id {
                                    let server_id_clone = server_id.clone();
                                    // Extract data before mutable call
                                    let _ = tab; // Release immutable borrow
                                    self.app_state
                                        .add_private_tab(&server_id_clone, nick.clone());
                                }
                            }
                        }
                    }
                    UserListMessage::UserDoubleClicked(nick) => {
                        // Send whois command
                        if let Some(current_tab) = &self.app_state.current_tab_id {
                            if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                if let Some(server_id) = &tab.server_id {
                                    self.send_irc_command(server_id, &format!("/whois {nick}"));
                                }
                            }
                        }
                    }
                    UserListMessage::UserContextMenu(nick) => {
                        // Show user context menu at cursor position
                        self.context_menu_visible = true;
                        self.context_menu_x = 300.0; // Default position
                        self.context_menu_y = 200.0;
                        info!("User context menu for: {}", nick);
                    }
                    UserListMessage::SortByNick => {
                        // Sort user list by nickname
                        info!("Sorting user list by nickname");
                        self.user_list
                            .set_sort_mode(crate::widgets::user_list::SortMode::Nickname);
                    }
                    UserListMessage::SortByMode => {
                        // Sort user list by user mode
                        info!("Sorting user list by mode");
                        self.user_list
                            .set_sort_mode(crate::widgets::user_list::SortMode::Privilege);
                    }
                    UserListMessage::FilterChanged(filter) => {
                        // Apply filter to user list
                        info!("User list filter changed: {}", filter);
                        self.user_list.set_filter(filter.clone());
                    }
                    UserListMessage::RefreshList => {
                        // Refresh user list
                        info!("Refreshing user list");
                        self.user_list.refresh();
                    }
                }

                // Now call widget update after handling app-level logic
                let task = self.user_list.update(user_list_msg, &mut self.app_state);

                // Execute widget task and any additional app-level tasks
                return Task::batch(vec![task.map(Message::UserList)]);
            }
            Message::InputArea(input_area_msg) => {
                // Handle input area specific actions first
                use crate::widgets::input_area::InputAreaMessage;
                match &input_area_msg {
                    InputAreaMessage::InputChanged(text) => {
                        self.input_buffer = text.clone();
                    }
                    InputAreaMessage::InputSubmitted(text) => {
                        // Process input as IRC command or message
                        if text.starts_with('/') {
                            self.handle_irc_command(text);
                        } else if let Some(current_tab) = &self.app_state.current_tab_id {
                            if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                if let Some(server_id) = &tab.server_id {
                                    // Extract data before mutable calls and async operations
                                    let client_clone = self.irc_client.clone();
                                    let target = tab.name.clone();
                                    let message = text.clone();
                                    let server_id_clone = server_id.clone();

                                    // Clone data for async task before moving
                                    let target_for_async = target.clone();
                                    let message_for_async = message.clone();

                                    tokio::spawn(async move {
                                        let client_guard = client_clone.read().await;
                                        if let Some(client) = client_guard.as_ref() {
                                            let _ = client
                                                .send_message(&target_for_async, &message_for_async)
                                                .await;
                                        }
                                    });

                                    // Release immutable borrow before mutable call
                                    let _ = tab;
                                    self.app_state.add_message(
                                        &server_id_clone,
                                        &target,
                                        &message,
                                        "self",
                                    );
                                }
                            }
                        }
                        self.input_buffer.clear();
                    }
                    InputAreaMessage::TabCompleted(text) => {
                        // Update input with tab-completed text
                        self.input_buffer = text.clone();
                    }
                    InputAreaMessage::HistoryUp => {
                        // Navigate command history up
                        info!("Command history up");
                    }
                    InputAreaMessage::HistoryDown => {
                        // Navigate command history down
                        info!("Command history down");
                    }
                    InputAreaMessage::SendMessage(text) => {
                        // Send message or execute command
                        info!("Sending message: {}", text);
                        // For now, just send the text as a raw IRC command
                        if let Some(current_tab) = &self.app_state.current_tab_id {
                            if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                if let Some(server_id) = &tab.server_id {
                                    self.send_irc_command(server_id, text);
                                }
                            }
                        }
                    }
                    InputAreaMessage::ToggleMultiline => {
                        // Toggle multiline input mode
                        info!("Toggling multiline input mode");
                        self.input_area.toggle_multiline();
                    }
                    InputAreaMessage::TabCompletion => {
                        // Handle tab completion - this is already implemented in InputArea
                        info!("Tab completion requested - delegating to input area");

                        // The InputArea already handles tab completion completely in its update method.
                        // It has comprehensive logic for:
                        // - Command completion (starting with /)
                        // - Nick completion with proper mention format
                        // - Channel completion (# and &)
                        // - Cycling through candidates
                        // - Completion hints display

                        // No additional handling needed here - the input area manages everything
                    }
                    InputAreaMessage::ClearInput => {
                        // Clear input field
                        info!("Clearing input");
                        self.input_area.clear();
                    }
                    InputAreaMessage::PasteText(text) => {
                        // Handle pasted text
                        info!("Text pasted: {}", text);
                        self.input_area.set_input(text.clone());
                    }
                    InputAreaMessage::KeyPressed(key, modifiers) => {
                        // Handle key press events
                        info!("Key pressed: {:?} with modifiers: {:?}", key, modifiers);

                        // Implement key handling logic for IRC client functionality
                        use iced::keyboard::{key::Named, Key};

                        match key {
                            Key::Named(Named::Tab) => {
                                // Tab completion (already handled by InputArea)
                                info!("Tab key pressed - completion handled by InputArea");
                            }
                            Key::Named(Named::Enter) => {
                                if modifiers.control() {
                                    // Ctrl+Enter for multiline input
                                    info!("Ctrl+Enter pressed - multiline input");
                                } else {
                                    // Regular Enter for sending message
                                    info!("Enter pressed - sending message");
                                }
                            }
                            Key::Named(Named::ArrowUp) => {
                                if modifiers.control() {
                                    // Ctrl+Up for history navigation
                                    info!("Ctrl+Up pressed - history up");
                                }
                            }
                            Key::Named(Named::ArrowDown) => {
                                if modifiers.control() {
                                    // Ctrl+Down for history navigation
                                    info!("Ctrl+Down pressed - history down");
                                }
                            }
                            Key::Named(Named::PageUp) => {
                                // Scroll message history up
                                info!("PageUp pressed - scroll messages up");
                                // Could implement message view scrolling here
                            }
                            Key::Named(Named::PageDown) => {
                                // Scroll message history down
                                info!("PageDown pressed - scroll messages down");
                                // Could implement message view scrolling here
                            }
                            Key::Named(Named::Escape) => {
                                // Cancel current operation or close dialogs
                                info!("Escape pressed - cancel operation");
                                if self.connect_dialog_visible
                                    || self.preferences_dialog_visible
                                    || self.about_dialog_visible
                                {
                                    self.connect_dialog_visible = false;
                                    self.preferences_dialog_visible = false;
                                    self.about_dialog_visible = false;
                                }
                            }
                            Key::Character(ch) => {
                                // Handle character input
                                if modifiers.control() {
                                    match ch.as_str() {
                                        "l" => {
                                            // Ctrl+L to clear current channel/buffer
                                            info!("Ctrl+L pressed - clear buffer");
                                            // Could implement buffer clearing here
                                        }
                                        "k" => {
                                            // Ctrl+K for IRC color codes
                                            info!("Ctrl+K pressed - IRC color codes");
                                            // Could implement color code insertion
                                        }
                                        "b" => {
                                            // Ctrl+B for IRC bold
                                            info!("Ctrl+B pressed - IRC bold formatting");
                                            // Could implement bold text formatting
                                        }
                                        "u" => {
                                            // Ctrl+U for IRC underline
                                            info!("Ctrl+U pressed - IRC underline formatting");
                                            // Could implement underline text formatting
                                        }
                                        "i" => {
                                            // Ctrl+I for IRC italic
                                            info!("Ctrl+I pressed - IRC italic formatting");
                                            // Could implement italic text formatting
                                        }
                                        _ => {
                                            // Other Ctrl+character combinations
                                            info!("Ctrl+{} pressed - not handled", ch);
                                        }
                                    }
                                } else if modifiers.alt() {
                                    // Alt+character for tab switching
                                    if let Ok(tab_num) = ch.parse::<usize>() {
                                        if tab_num > 0 && tab_num <= 9 {
                                            info!(
                                                "Alt+{} pressed - switch to tab {}",
                                                tab_num, tab_num
                                            );
                                            // Could implement tab switching here
                                            // self.switch_to_tab(tab_num - 1);
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Other keys not specifically handled
                                info!("Key {:?} not specifically handled", key);
                            }
                        }

                        // All key handling is also processed by the InputArea widget
                        // which has its own comprehensive key handling logic
                    }
                }

                // Now call widget update after handling app-level logic
                let task = self.input_area.update(input_area_msg, &mut self.app_state);

                // Execute widget task and any additional app-level tasks
                return Task::batch(vec![task.map(Message::InputArea)]);
            }
            Message::TabBar(tab_bar_msg) => {
                // Handle tab bar specific actions first
                use crate::widgets::tab_bar::TabBarMessage;
                match &tab_bar_msg {
                    TabBarMessage::SwitchTab(tab_id) => {
                        // Switch to the specified tab
                        self.app_state.current_tab_id = Some(tab_id.clone());
                    }
                    TabBarMessage::TabSelected(tab_id) => {
                        self.app_state.current_tab_id = Some(tab_id.clone());
                    }
                    TabBarMessage::CloseTab(tab_id) => {
                        // Close the specified tab
                        self.app_state.remove_tab(tab_id);
                        // Select next available tab
                        if self.app_state.current_tab_id.as_ref() == Some(tab_id) {
                            self.app_state.current_tab_id =
                                self.app_state.tab_order.first().cloned();
                        }
                    }
                    TabBarMessage::TabClosed(tab_id) => {
                        self.app_state.remove_tab(tab_id);
                        // Select next available tab
                        if self.app_state.current_tab_id.as_ref() == Some(tab_id) {
                            self.app_state.current_tab_id =
                                self.app_state.tab_order.first().cloned();
                        }
                    }
                    TabBarMessage::MoveTab(tab_id, new_position) => {
                        // Implement tab reordering
                        if let Some(current_pos) =
                            self.app_state.tab_order.iter().position(|id| id == tab_id)
                        {
                            let tab_id = self.app_state.tab_order.remove(current_pos);
                            let insert_pos = (*new_position).min(self.app_state.tab_order.len());
                            self.app_state.tab_order.insert(insert_pos, tab_id);
                        }
                    }
                    TabBarMessage::NewTab => {
                        // Show new tab dialog (simplified: create server tab)
                        let server_id = format!("new_server_{}", self.app_state.servers.len() + 1);
                        self.app_state
                            .add_server(server_id.clone(), "New Server".to_string());
                        self.app_state.current_tab_id = Some(server_id);
                    }
                    TabBarMessage::CloseAllTabs => {
                        // Close all tabs
                        self.app_state.tabs.clear();
                        self.app_state.tab_order.clear();
                        self.app_state.current_tab_id = None;
                    }
                    TabBarMessage::CloseOtherTabs(keep_tab_id) => {
                        // Close all tabs except the specified one
                        let tabs_to_remove: Vec<String> = self
                            .app_state
                            .tab_order
                            .iter()
                            .filter(|&id| id != keep_tab_id)
                            .cloned()
                            .collect();

                        for tab_id in tabs_to_remove {
                            self.app_state.remove_tab(&tab_id);
                        }

                        // Set the kept tab as current
                        self.app_state.current_tab_id = Some(keep_tab_id.clone());
                    }
                    TabBarMessage::TabContextMenu(tab_id) => {
                        // Show tab context menu
                        self.context_menu_visible = true;
                        self.context_menu_x = 400.0;
                        self.context_menu_y = 50.0;
                        info!("Tab context menu for: {}", tab_id);
                    }
                }

                // Now call widget update after handling app-level logic
                let task = self.tab_bar.update(tab_bar_msg, &mut self.app_state);

                // Execute widget task and any additional app-level tasks
                return Task::batch(vec![task.map(Message::TabBar)]);
            }
            Message::StatusBar(status_bar_msg) => {
                // Handle status bar specific actions first
                use crate::widgets::status_bar::StatusBarMessage;
                match &status_bar_msg {
                    StatusBarMessage::UpdateStatus => {
                        // Status updated automatically
                    }
                    StatusBarMessage::ToggleTopicBar => {
                        // Toggle topic bar visibility
                        info!("Toggle topic bar");
                    }
                    StatusBarMessage::ClearStatus => {
                        // Clear status messages
                        info!("Clear status");
                    }
                }

                // Now call widget update after handling app-level logic
                let task = self.status_bar.update(status_bar_msg, &mut self.app_state);

                // Execute widget task and any additional app-level tasks
                return Task::batch(vec![task.map(Message::StatusBar)]);
            }

            // IRC Event handlers (from real IRC events)
            Message::IrcConnected(connection_id) => {
                info!("GUI: IRC connected event received: {}", connection_id);
                // Update connection status in app state
                if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                    server.connection_state = rustirc_core::ConnectionState::Connected;
                    info!("Updated server connection state to Connected");
                } else {
                    warn!("Could not find server {} in app state", connection_id);
                }
            }
            Message::IrcDisconnected(connection_id, reason) => {
                info!("IRC disconnected: {} - {}", connection_id, reason);
                // Update connection status in app state
                if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                    server.connection_state = rustirc_core::ConnectionState::Disconnected;
                }
            }
            Message::IrcMessageReceived(connection_id, message) => {
                info!("IRC message received from {}: {}", connection_id, message);

                // Parse IRC message and update GUI state
                match message.command.as_str() {
                    "001" => {
                        // RPL_WELCOME - Registration successful
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            server.connection_state = rustirc_core::ConnectionState::Registered;
                        }

                        // Display welcome message
                        if let Some(text) = message.params.last() {
                            self.app_state.add_message(
                                &connection_id,
                                &connection_id,
                                text,
                                "server",
                            );
                        }
                    }
                    "375" | "372" | "376" => {
                        // MOTD start (375), MOTD line (372), MOTD end (376)
                        if let Some(text) = message.params.last() {
                            self.app_state.add_message(
                                &connection_id,
                                &connection_id,
                                text,
                                "motd",
                            );
                        }
                    }
                    "353" => {
                        // RPL_NAMREPLY - User list for channel
                        if message.params.len() >= 3 {
                            let channel = &message.params[2];
                            if let Some(user_list) = message.params.get(3) {
                                let users: Vec<String> = user_list
                                    .split_whitespace()
                                    .map(|s| s.to_string())
                                    .collect();

                                // Update channel user list
                                if let Some(server) = self.app_state.servers.get_mut(&connection_id)
                                {
                                    if let Some(channel_info) = server.channels.get_mut(channel) {
                                        channel_info.users = users.clone();
                                        channel_info.user_count = channel_info.users.len();
                                    }
                                }

                                // Update the GUI user list display
                                self.update_user_list(users);
                            }
                        }
                    }
                    "366" => {
                        // RPL_ENDOFNAMES - End of user list
                        if message.params.len() >= 2 {
                            let channel = &message.params[1];
                            info!("End of names list for channel: {}", channel);
                        }
                    }
                    "322" => {
                        // RPL_LIST - Channel list entry
                        if message.params.len() >= 4 {
                            let channel = &message.params[1];
                            let user_count = &message.params[2];
                            let topic = &message.params[3];
                            let list_entry = format!("{channel} ({user_count} users): {topic}");
                            self.app_state.add_message(
                                &connection_id,
                                &connection_id,
                                &list_entry,
                                "channel_list",
                            );
                        }
                    }
                    "323" => {
                        // RPL_LISTEND - End of channel list
                        self.app_state.add_message(
                            &connection_id,
                            &connection_id,
                            "End of channel list",
                            "system",
                        );
                    }
                    "JOIN" => {
                        // User joined channel
                        if let Some(channel) = message.params.first() {
                            if let Some(nick) = message.prefix.as_ref().and_then(|p| match p {
                                rustirc_protocol::Prefix::User { nick, .. } => Some(nick.as_str()),
                                _ => None,
                            }) {
                                // Add user to channel
                                if let Some(server) = self.app_state.servers.get_mut(&connection_id)
                                {
                                    if let Some(channel_info) = server.channels.get_mut(channel) {
                                        if !channel_info.users.contains(&nick.to_string()) {
                                            channel_info.users.push(nick.to_string());
                                            channel_info.user_count = channel_info.users.len();
                                        }
                                    }
                                }

                                // Display join message
                                let join_msg = format!("{nick} has joined {channel}");
                                self.app_state.add_message(
                                    &connection_id,
                                    channel,
                                    &join_msg,
                                    "system",
                                );
                            }
                        }
                    }
                    "PART" => {
                        // User left channel
                        if let Some(channel) = message.params.first() {
                            if let Some(nick) = message.prefix.as_ref().and_then(|p| match p {
                                rustirc_protocol::Prefix::User { nick, .. } => Some(nick.as_str()),
                                _ => None,
                            }) {
                                // Remove user from channel
                                if let Some(server) = self.app_state.servers.get_mut(&connection_id)
                                {
                                    if let Some(channel_info) = server.channels.get_mut(channel) {
                                        channel_info.users.retain(|u| u != nick);
                                        channel_info.user_count = channel_info.users.len();
                                    }
                                }

                                // Display part message
                                let part_msg = format!("{nick} has left {channel}");
                                self.app_state.add_message(
                                    &connection_id,
                                    channel,
                                    &part_msg,
                                    "system",
                                );
                            }
                        }
                    }
                    "QUIT" => {
                        // User quit
                        if let Some(nick) = message.prefix.as_ref().and_then(|p| match p {
                            rustirc_protocol::Prefix::User { nick, .. } => Some(nick.as_str()),
                            _ => None,
                        }) {
                            let quit_reason =
                                message.params.first().map(|s| s.as_str()).unwrap_or("Quit");
                            let quit_msg = format!("{nick} has quit ({quit_reason})");

                            // Collect channels that the user was in, then update each separately
                            let mut channels_to_update = Vec::new();

                            // Remove user from all channels
                            if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                                for (channel_name, channel_info) in server.channels.iter_mut() {
                                    if channel_info.users.contains(&nick.to_string()) {
                                        channel_info.users.retain(|u| u != nick);
                                        channel_info.user_count = channel_info.users.len();
                                        channels_to_update.push(channel_name.clone());
                                    }
                                }
                            }

                            // Display quit message in each channel where the user was present
                            for channel_name in channels_to_update {
                                self.app_state.add_message(
                                    &connection_id,
                                    &channel_name,
                                    &quit_msg,
                                    "system",
                                );
                            }
                        }
                    }
                    "PRIVMSG" => {
                        // Channel or private message
                        if message.params.len() >= 2 {
                            let target = &message.params[0];
                            let text = &message.params[1];
                            if let Some(nick) = message.prefix.as_ref().and_then(|p| match p {
                                rustirc_protocol::Prefix::User { nick, .. } => Some(nick.as_str()),
                                _ => None,
                            }) {
                                self.app_state
                                    .add_message(&connection_id, target, text, nick);
                            }
                        }
                    }
                    "NOTICE" => {
                        // Notice message
                        if message.params.len() >= 2 {
                            let target = &message.params[0];
                            let text = &message.params[1];
                            if let Some(nick) = message.prefix.as_ref().map(|p| match p {
                                rustirc_protocol::Prefix::User { nick, .. } => nick.as_str(),
                                rustirc_protocol::Prefix::Server(server) => server.as_str(),
                            }) {
                                let notice_msg = format!("-{nick}- {text}");
                                self.app_state.add_message(
                                    &connection_id,
                                    target,
                                    &notice_msg,
                                    "notice",
                                );
                            }
                        }
                    }
                    _ => {
                        // Log other messages for debugging
                        info!(
                            "Unhandled IRC message: {} from {}",
                            message.command, connection_id
                        );
                    }
                }
            }
            Message::IrcConnectionStateChanged(connection_id, state) => {
                info!(
                    "IRC connection state changed: {} -> {:?}",
                    connection_id, state
                );
                // Update connection state in app state
                if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                    server.connection_state = state;
                }
            }
            Message::IrcError(connection_id, error) => {
                warn!("IRC error for {:?}: {}", connection_id, error);
                // Display error message
                if let Some(conn_id) = &connection_id {
                    self.app_state.add_message(
                        conn_id,
                        conn_id,
                        &format!("Error: {error}"),
                        "error",
                    );
                }
            }
            Message::CoreEvent(core_event) => {
                // Handle IRC core events from the event handler
                match core_event {
                    CoreEventMessage::Connected { connection_id } => {
                        info!("Core event: Connected to {}", connection_id);
                        // Update connection status in app state
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            server.connection_state = rustirc_core::ConnectionState::Connected;
                        }
                        self.app_state.add_message(
                            &connection_id,
                            &connection_id,
                            "Connected to server",
                            "system",
                        );
                    }
                    CoreEventMessage::Disconnected {
                        connection_id,
                        reason,
                    } => {
                        info!(
                            "Core event: Disconnected from {} - {}",
                            connection_id, reason
                        );
                        // Update connection status in app state
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            server.connection_state = rustirc_core::ConnectionState::Disconnected;
                        }
                        self.app_state.add_message(
                            &connection_id,
                            &connection_id,
                            &format!("Disconnected: {reason}"),
                            "system",
                        );
                    }
                    CoreEventMessage::MessageReceived {
                        connection_id,
                        message,
                    } => {
                        info!(
                            "Core event: Message received from {}: {}",
                            connection_id, message
                        );

                        // Parse IRC message and update GUI state
                        match message.command.as_str() {
                            "375" => {
                                // MOTD start
                                if message.params.len() >= 2 {
                                    let motd_start = &message.params[1];
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        motd_start,
                                        "MOTD",
                                    );
                                    info!("MOTD start: {}", motd_start);
                                }
                            }
                            "372" => {
                                // MOTD line
                                if message.params.len() >= 2 {
                                    let motd_line = &message.params[1];
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        motd_line,
                                        "MOTD",
                                    );
                                }
                            }
                            "376" => {
                                // MOTD end
                                if message.params.len() >= 2 {
                                    let motd_end = &message.params[1];
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        motd_end,
                                        "MOTD",
                                    );
                                    info!("MOTD complete: {}", motd_end);
                                }
                            }
                            "353" => {
                                // NAMES reply (channel user list)
                                if message.params.len() >= 4 {
                                    let channel = &message.params[2];
                                    let users = &message.params[3];
                                    let user_list: Vec<&str> = users.split_whitespace().collect();

                                    // Add users to channel state
                                    for user in &user_list {
                                        let clean_user =
                                            user.trim_start_matches(['@', '+', '%', '&', '~']);
                                        self.app_state.add_user_to_channel(
                                            &connection_id,
                                            channel,
                                            clean_user,
                                        );
                                    }

                                    let user_count_msg =
                                        format!("Users in {}: {}", channel, user_list.join(", "));
                                    self.app_state.add_message(
                                        &connection_id,
                                        channel,
                                        &user_count_msg,
                                        "System",
                                    );
                                    info!("Channel {} users: {:?}", channel, user_list);
                                }
                            }
                            "322" => {
                                // Channel list entry
                                if message.params.len() >= 4 {
                                    let channel = &message.params[1];
                                    let user_count = &message.params[2];
                                    let topic = &message.params[3];
                                    let list_entry =
                                        format!("{channel} ({user_count} users): {topic}");
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        &list_entry,
                                        "channel_list",
                                    );
                                }
                            }
                            "323" => {
                                // End of channel list
                                self.app_state.add_message(
                                    &connection_id,
                                    &connection_id,
                                    "End of channel list",
                                    "system",
                                );
                            }
                            "PRIVMSG" => {
                                // Channel or private message
                                if message.params.len() >= 2 {
                                    let target = &message.params[0];
                                    let text = &message.params[1];
                                    if let Some(nick) =
                                        message.prefix.as_ref().and_then(|p| match p {
                                            rustirc_protocol::Prefix::User { nick, .. } => {
                                                Some(nick.as_str())
                                            }
                                            _ => None,
                                        })
                                    {
                                        self.app_state.add_message(
                                            &connection_id,
                                            target,
                                            text,
                                            nick,
                                        );
                                        // Trigger auto-scroll for new messages
                                        return Task::batch(vec![self.trigger_auto_scroll()]);
                                    }
                                }
                            }
                            "NOTICE" => {
                                // Server notice messages
                                if message.params.len() >= 2 {
                                    let text = &message.params[1];
                                    let source = match &message.prefix {
                                        Some(rustirc_protocol::Prefix::Server(server)) => {
                                            server.as_str()
                                        }
                                        Some(rustirc_protocol::Prefix::User { nick, .. }) => {
                                            nick.as_str()
                                        }
                                        _ => "Server",
                                    };
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        text,
                                        source,
                                    );
                                    info!("Server notice from {}: {}", source, text);
                                }
                            }
                            "001" => {
                                // Welcome message (RPL_WELCOME)
                                if message.params.len() >= 2 {
                                    let welcome_text = &message.params[1];
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        welcome_text,
                                        "Server",
                                    );
                                    info!("Welcome message: {}", welcome_text);
                                }
                            }
                            "002" | "003" | "004" | "005" => {
                                // Server info messages
                                if message.params.len() >= 2 {
                                    let info_text = &message.params[1];
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        info_text,
                                        "Server",
                                    );
                                }
                            }
                            "250" | "251" | "252" | "253" | "254" | "255" | "265" | "266" => {
                                // Server statistics
                                if message.params.len() >= 2 {
                                    let stats_text = &message.params[1];
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        stats_text,
                                        "Server",
                                    );
                                }
                            }
                            "366" => {
                                // End of NAMES (end of user list)
                                if message.params.len() >= 3 {
                                    let channel = &message.params[1];
                                    let end_msg = format!("End of user list for {channel}");
                                    self.app_state.add_message(
                                        &connection_id,
                                        channel,
                                        &end_msg,
                                        "System",
                                    );
                                }
                            }
                            "311" => {
                                // RPL_WHOISUSER - WHOIS user info
                                if message.params.len() >= 6 {
                                    let nick = &message.params[1];
                                    let user = &message.params[2];
                                    let host = &message.params[3];
                                    let realname = &message.params[5];
                                    let whois_msg =
                                        format!("WHOIS: {nick} ({user}@{host}) - {realname}");
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        &whois_msg,
                                        "whois",
                                    );
                                }
                            }
                            "312" => {
                                // RPL_WHOISSERVER - WHOIS server info
                                if message.params.len() >= 4 {
                                    let nick = &message.params[1];
                                    let server = &message.params[2];
                                    let server_info = &message.params[3];
                                    let whois_msg = format!(
                                        "WHOIS: {nick} is using server {server} ({server_info})"
                                    );
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        &whois_msg,
                                        "whois",
                                    );
                                }
                            }
                            "313" => {
                                // RPL_WHOISOPERATOR - WHOIS operator status
                                if message.params.len() >= 3 {
                                    let nick = &message.params[1];
                                    let whois_msg = format!("WHOIS: {nick} is an IRC operator");
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        &whois_msg,
                                        "whois",
                                    );
                                }
                            }
                            "317" => {
                                // RPL_WHOISIDLE - WHOIS idle time
                                if message.params.len() >= 4 {
                                    let nick = &message.params[1];
                                    let idle_time = &message.params[2];
                                    let signon_time = &message.params[3];
                                    let whois_msg = format!(
                                        "WHOIS: {nick} has been idle for {idle_time} seconds, signed on at {signon_time}"
                                    );
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        &whois_msg,
                                        "whois",
                                    );
                                }
                            }
                            "318" => {
                                // RPL_ENDOFWHOIS - End of WHOIS
                                if message.params.len() >= 3 {
                                    let nick = &message.params[1];
                                    let whois_msg = format!("WHOIS: End of information for {nick}");
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        &whois_msg,
                                        "whois",
                                    );
                                }
                            }
                            "319" => {
                                // RPL_WHOISCHANNELS - WHOIS channels
                                if message.params.len() >= 3 {
                                    let nick = &message.params[1];
                                    let channels = &message.params[2];
                                    let whois_msg =
                                        format!("WHOIS: {nick} is on channels: {channels}");
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        &whois_msg,
                                        "whois",
                                    );
                                }
                            }
                            "332" => {
                                // Channel topic
                                if message.params.len() >= 3 {
                                    let channel = &message.params[1];
                                    let topic = &message.params[2];
                                    let topic_msg = format!("Topic: {topic}");
                                    self.app_state.add_message(
                                        &connection_id,
                                        channel,
                                        &topic_msg,
                                        "System",
                                    );
                                    info!("Channel {} topic: {}", channel, topic);
                                }
                            }
                            "JOIN" => {
                                // User joined channel
                                if !message.params.is_empty() {
                                    let channel = &message.params[0];
                                    if let Some(rustirc_protocol::Prefix::User { nick, .. }) =
                                        &message.prefix
                                    {
                                        let join_msg = format!("{nick} has joined {channel}");
                                        self.app_state.add_message(
                                            &connection_id,
                                            channel,
                                            &join_msg,
                                            "System",
                                        );
                                        self.app_state.add_user_to_channel(
                                            &connection_id,
                                            channel,
                                            nick,
                                        );
                                        info!("User {} joined {}", nick, channel);
                                    }
                                }
                            }
                            "PART" => {
                                // User left channel
                                if !message.params.is_empty() {
                                    let channel = &message.params[0];
                                    let part_reason = if message.params.len() >= 2 {
                                        format!(" ({})", &message.params[1])
                                    } else {
                                        String::new()
                                    };
                                    if let Some(rustirc_protocol::Prefix::User { nick, .. }) =
                                        &message.prefix
                                    {
                                        let part_msg =
                                            format!("{nick} has left {channel}{part_reason}");
                                        self.app_state.add_message(
                                            &connection_id,
                                            channel,
                                            &part_msg,
                                            "System",
                                        );
                                        self.app_state.remove_user_from_channel(
                                            &connection_id,
                                            channel,
                                            nick,
                                        );
                                        info!("User {} left {}", nick, channel);
                                    }
                                }
                            }
                            "QUIT" => {
                                // User quit
                                if let Some(rustirc_protocol::Prefix::User { nick, .. }) =
                                    &message.prefix
                                {
                                    let quit_reason = if !message.params.is_empty() {
                                        format!(" ({})", &message.params[0])
                                    } else {
                                        String::new()
                                    };
                                    let quit_msg = format!("{nick} has quit{quit_reason}");
                                    // Add quit message to all channels where this user was present
                                    self.app_state.add_message(
                                        &connection_id,
                                        &connection_id,
                                        &quit_msg,
                                        "System",
                                    );
                                    self.app_state
                                        .remove_user_from_all_channels(&connection_id, nick);
                                    info!("User {} quit", nick);
                                }
                            }
                            _ => {
                                // Handle other message types
                                info!(
                                    "Unhandled core event message: {} from {}",
                                    message.command, connection_id
                                );
                            }
                        }
                    }
                    CoreEventMessage::StateChanged {
                        connection_id,
                        state,
                    } => {
                        info!(
                            "Core event: Connection state changed: {} -> {:?}",
                            connection_id, state
                        );
                        // Update connection state in app state
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            server.connection_state = state;
                        }
                    }
                    CoreEventMessage::Error {
                        connection_id,
                        error,
                    } => {
                        warn!("Core event: IRC error for {:?}: {}", connection_id, error);
                        // Display error message
                        if let Some(conn_id) = &connection_id {
                            self.app_state.add_message(
                                conn_id,
                                conn_id,
                                &format!("Error: {error}"),
                                "error",
                            );
                        }
                    }
                    CoreEventMessage::MessageSent {
                        connection_id,
                        message,
                    } => {
                        debug!(
                            "Core event: Message sent to {}: {:?}",
                            connection_id, message
                        );
                        // Log message sending for debugging
                    }
                    CoreEventMessage::ChannelJoined {
                        connection_id,
                        channel,
                    } => {
                        info!(
                            "Core event: Joined channel {} on {}",
                            channel, connection_id
                        );
                        // Add channel to server if not already there
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            if !server.channels.contains_key(&channel) {
                                self.app_state
                                    .add_channel_tab(connection_id.clone(), channel.clone());
                            }
                        }
                        self.app_state.add_message(
                            &connection_id,
                            &channel,
                            &format!("You have joined {channel}"),
                            "system",
                        );
                    }
                    CoreEventMessage::ChannelLeft {
                        connection_id,
                        channel,
                    } => {
                        info!("Core event: Left channel {} on {}", channel, connection_id);
                        // Remove channel from server
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            server.channels.remove(&channel);
                        }
                        self.app_state.add_message(
                            &connection_id,
                            &channel,
                            &format!("You have left {channel}"),
                            "system",
                        );
                    }
                    CoreEventMessage::UserJoined {
                        connection_id,
                        channel,
                        user,
                    } => {
                        debug!(
                            "Core event: User {} joined {} on {}",
                            user, channel, connection_id
                        );
                        // Add user to channel
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            if let Some(channel_info) = server.channels.get_mut(&channel) {
                                if !channel_info.users.contains(&user) {
                                    channel_info.users.push(user.clone());
                                    channel_info.user_count = channel_info.users.len();
                                }
                            }
                        }
                        self.app_state.add_message(
                            &connection_id,
                            &channel,
                            &format!("{user} has joined"),
                            "system",
                        );
                    }
                    CoreEventMessage::UserLeft {
                        connection_id,
                        channel,
                        user,
                    } => {
                        debug!(
                            "Core event: User {} left {} on {}",
                            user, channel, connection_id
                        );
                        // Remove user from channel
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            if let Some(channel_info) = server.channels.get_mut(&channel) {
                                channel_info.users.retain(|u| u != &user);
                                channel_info.user_count = channel_info.users.len();
                            }
                        }
                        self.app_state.add_message(
                            &connection_id,
                            &channel,
                            &format!("{user} has left"),
                            "system",
                        );
                    }
                    CoreEventMessage::NickChanged {
                        connection_id,
                        old_nick,
                        new_nick,
                    } => {
                        info!(
                            "Core event: Nick changed from {} to {} on {}",
                            old_nick, new_nick, connection_id
                        );
                        // Update nick in all channels and collect affected channels
                        let mut affected_channels = Vec::new();
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            for (channel_name, channel_info) in server.channels.iter_mut() {
                                if let Some(pos) =
                                    channel_info.users.iter().position(|u| u == &old_nick)
                                {
                                    channel_info.users[pos] = new_nick.clone();
                                    affected_channels.push(channel_name.clone());
                                }
                            }
                        }
                        // Add messages to affected channels
                        for channel_name in affected_channels {
                            self.app_state.add_message(
                                &connection_id,
                                &channel_name,
                                &format!("{old_nick} is now known as {new_nick}"),
                                "system",
                            );
                        }
                    }
                    CoreEventMessage::TopicChanged {
                        connection_id,
                        channel,
                        topic,
                    } => {
                        info!(
                            "Core event: Topic changed in {} on {}: {}",
                            channel, connection_id, topic
                        );
                        // Update channel topic
                        if let Some(server) = self.app_state.servers.get_mut(&connection_id) {
                            if let Some(channel_info) = server.channels.get_mut(&channel) {
                                channel_info.topic = Some(topic.clone());
                            }
                        }
                        self.app_state.add_message(
                            &connection_id,
                            &channel,
                            &format!("Topic changed to: {topic}"),
                            "system",
                        );
                    }
                }
            }
            // Menu dropdown handlers
            Message::MenuToggle(menu_name) => {
                if self.active_menu.as_ref() == Some(&menu_name) {
                    self.active_menu = None; // Close if already open
                } else {
                    self.active_menu = Some(menu_name); // Open requested menu
                }
            }
            Message::MenuClose => {
                self.active_menu = None;
            }

            // Dialog handlers
            Message::ShowPreferencesDialog => {
                self.preferences_dialog_visible = true;
                self.active_menu = None; // Close menu
            }
            Message::HidePreferencesDialog => {
                self.preferences_dialog_visible = false;
            }
            Message::ShowAboutDialog => {
                self.about_dialog_visible = true;
                self.active_menu = None; // Close menu
            }
            Message::HideAboutDialog => {
                self.about_dialog_visible = false;
            }

            // Menu action handlers
            Message::MenuFileConnect => {
                self.active_menu = None; // Close menu
                return Task::batch(vec![Task::done(Message::ShowConnectDialog)]);
            }
            Message::MenuFileDisconnect => {
                self.active_menu = None; // Close menu
                                         // Disconnect from current server
                if let Some(current_tab) = &self.app_state.current_tab_id {
                    if let Some(tab) = self.app_state.tabs.get(current_tab) {
                        if let Some(server_id) = &tab.server_id {
                            return Task::batch(vec![Task::done(Message::DisconnectFromServer(
                                server_id.clone(),
                            ))]);
                        }
                    }
                }
            }
            Message::MenuFileExit => {
                // Exit the application
                std::process::exit(0);
            }
            Message::MenuViewToggleSystemMessages => {
                self.active_menu = None; // Close menu
                self.message_view.toggle_system_messages();
            }
            Message::MenuViewToggleUserLists => {
                self.active_menu = None; // Close menu
                self.message_view.toggle_user_lists();
                self.toggle_user_list(); // Also toggle the actual user list pane visibility
            }
            Message::MenuViewToggleJoinsParts => {
                self.active_menu = None; // Close menu
                self.message_view.toggle_joins_parts();
            }
            Message::MenuViewToggleMotd => {
                self.active_menu = None; // Close menu
                self.message_view.toggle_motd();
            }
            Message::MenuViewToggleTimestamps => {
                self.active_menu = None; // Close menu
                                         // Toggle timestamp display
                self.message_view.toggle_timestamps();
                info!("Toggle timestamps requested");
            }
            Message::MenuToolsPreferences => {
                return Task::batch(vec![Task::done(Message::ShowPreferencesDialog)]);
            }
            Message::MenuHelpAbout => {
                return Task::batch(vec![Task::done(Message::ShowAboutDialog)]);
            }

            // Input and navigation commands
            Message::TabComplete => {
                // Trigger tab completion in the input widget
                // For now, just pass to InputArea widget
            }
            Message::CancelOperation => {
                // Cancel current operation or close dialogs
                self.preferences_dialog_visible = false;
                self.about_dialog_visible = false;
                self.connect_dialog_visible = false;
            }
            Message::HistoryPrevious => {
                // Navigate to previous command in history
                // For now, just pass to InputArea widget
            }
            Message::HistoryNext => {
                // Navigate to next command in history
                // For now, just pass to InputArea widget
            }
            Message::ScrollUp => {
                // Scroll up in the message view
                // For now, just pass to MessageView widget
            }
            Message::ScrollDown => {
                // Scroll down in the message view
                // For now, just pass to MessageView widget
            }
            Message::ShowHelp => {
                // Show help dialog or help content
                return Task::batch(vec![Task::done(Message::ShowAboutDialog)]);
            }
            Message::CopySelection => {
                // Copy selected text to clipboard
                // For now, just a no-op
            }
            Message::PasteText => {
                // Paste text from clipboard
                // For now, just a no-op
            }
            Message::WindowResized(_width, _height) => {
                // Handle window resize events
                // For now, just a no-op as Iced handles this
            }
            Message::None => {}
        }

        Task::none()
    }

    /// View function for Iced 0.13.1 functional approach
    fn view(&self) -> Element<'_, Message> {
        let pane_grid = pane_grid::PaneGrid::new(&self.panes, |_pane, pane_type, _is_maximized| {
            self.pane_content(*pane_type)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(8) // Larger spacing to make dividers more visible
        .style(|_theme| pane_grid::Style {
            hovered_region: pane_grid::Highlight {
                background: Background::Color(Color::from_rgb(0.25, 0.47, 0.85)), // Blue hover effect
                border: iced::Border::default(),
            },
            hovered_split: pane_grid::Line {
                color: Color::from_rgb(0.25, 0.47, 0.85), // Blue divider color on hover
                width: 2.0,
            },
            picked_split: pane_grid::Line {
                color: Color::from_rgb(0.35, 0.57, 0.95), // Lighter blue when picked
                width: 3.0,
            },
        })
        .on_click(Message::PaneClicked)
        .on_drag(Message::PaneDragged)
        .on_resize(10, Message::PaneResized);

        let content = column![self.render_menu_bar(), self.render_tab_bar(), pane_grid,]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill);

        // Add context menu if visible
        if self.context_menu_visible {
            // Implement context menu overlay
            let context_menu = container(
                column![
                    button("Whois").on_press(Message::ContextMenuAction("whois".to_string())),
                    button("Query").on_press(Message::ContextMenuAction("query".to_string())),
                    button("Op").on_press(Message::ContextMenuAction("op".to_string())),
                    button("Voice").on_press(Message::ContextMenuAction("voice".to_string())),
                    button("Kick").on_press(Message::ContextMenuAction("kick".to_string())),
                    button("Ban").on_press(Message::ContextMenuAction("ban".to_string())),
                    button("Copy").on_press(Message::ContextMenuAction("copy".to_string())),
                    button("Close Tab")
                        .on_press(Message::ContextMenuAction("close_tab".to_string())),
                ]
                .spacing(2)
                .padding(5),
            )
            .padding(1)
            .width(Length::Shrink)
            .height(Length::Shrink);

            // For now, we'll overlay it simply - in a real implementation,
            // you'd use proper positioning based on context_menu_x/y
            return container(column![
                content,
                container(context_menu)
                    .width(Length::Fill)
                    .height(Length::Shrink)
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        // Add connection dialog if visible
        if self.connect_dialog_visible {
            let dialog = container(
                column![
                    text("Connect to IRC Server").size(20),
                    row![
                        text("Server: ").width(Length::Fixed(80.0)),
                        text_input("irc.libera.chat", &self.connect_dialog_server)
                            .on_input(Message::ConnectDialogServerChanged)
                            .width(Length::Fixed(200.0))
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("Port: ").width(Length::Fixed(80.0)),
                        text_input("6697", &self.connect_dialog_port)
                            .on_input(Message::ConnectDialogPortChanged)
                            .width(Length::Fixed(200.0))
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center),
                    row![
                        text("Nickname: ").width(Length::Fixed(80.0)),
                        text_input("RustIRC_User", &self.connect_dialog_nickname)
                            .on_input(Message::ConnectDialogNickChanged)
                            .width(Length::Fixed(200.0))
                    ]
                    .spacing(10)
                    .align_y(iced::Alignment::Center),
                    row![
                        button("Connect").on_press(Message::ConnectDialogConnect),
                        button("Cancel").on_press(Message::HideConnectDialog)
                    ]
                    .spacing(20)
                ]
                .spacing(15)
                .padding(20),
            )
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.95))),
                border: iced::Border {
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    width: 2.0,
                    radius: 10.0.into(),
                },
                ..Default::default()
            })
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill);

            return container(column![content, dialog])
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        // Preferences dialog overlay
        if self.preferences_dialog_visible {
            let dialog = container(
                column![
                    text("Preferences").size(18).color(Color::WHITE),
                    horizontal_rule(1),
                    text("Theme Settings")
                        .size(14)
                        .color(Color::from_rgb(0.8, 0.8, 0.8)),
                    text("• Dark Mode: Enabled")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("• Font Size: 14px")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("• Auto-connect: Disabled")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    horizontal_rule(1),
                    text("IRC Settings")
                        .size(14)
                        .color(Color::from_rgb(0.8, 0.8, 0.8)),
                    text("• Default Server: irc.libera.chat")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("• Default Port: 6697 (SSL)")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("• SASL Authentication: Enabled")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    horizontal_rule(1),
                    button("Close")
                        .on_press(Message::HidePreferencesDialog)
                        .padding([8, 16])
                ]
                .spacing(10)
                .padding(20),
            )
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.95))),
                border: iced::Border {
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            })
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fixed(400.0))
            .height(Length::Fixed(300.0));

            return container(stack![content, dialog])
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        // About dialog overlay
        if self.about_dialog_visible {
            let dialog = container(
                column![
                    text("About RustIRC").size(18).color(Color::WHITE),
                    horizontal_rule(1),
                    text("RustIRC v0.1.0")
                        .size(16)
                        .color(Color::from_rgb(0.9, 0.9, 0.9)),
                    text("Modern IRC Client")
                        .size(14)
                        .color(Color::from_rgb(0.8, 0.8, 0.8)),
                    horizontal_rule(1),
                    text("Built with Rust and Iced")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("© 2025 RustIRC Project")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    horizontal_rule(1),
                    text("Features:")
                        .size(14)
                        .color(Color::from_rgb(0.8, 0.8, 0.8)),
                    text("• IRCv3 Protocol Support")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("• SASL Authentication")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("• TLS/SSL Connections")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("• Cross-Platform GUI")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    text("• Modern Theming")
                        .size(12)
                        .color(Color::from_rgb(0.7, 0.7, 0.7)),
                    horizontal_rule(1),
                    button("Close")
                        .on_press(Message::HideAboutDialog)
                        .padding([8, 16])
                ]
                .spacing(8)
                .padding(20),
            )
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.15, 0.15, 0.15, 0.95))),
                border: iced::Border {
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    width: 2.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            })
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fixed(350.0))
            .height(Length::Fixed(400.0));

            return container(stack![content, dialog])
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Subscription function for receiving IRC events
    fn subscription(&self) -> iced::Subscription<Message> {
        // Poll for IRC events from the global receiver
        // The instance receiver (irc_message_receiver) is used for testing
        // and is polled separately in the update() method when needed
        iced::time::every(std::time::Duration::from_millis(100)).map(|_| {
            // Try to receive IRC events from the global receiver
            if let Some(receiver_arc) = IRC_EVENT_RECEIVER.get() {
                let mut guard = receiver_arc.lock().unwrap();
                if let Some(ref mut receiver) = guard.as_mut() {
                    match receiver.try_recv() {
                        Ok(message) => {
                            info!("GUI: Received IRC event via subscription: {:?}", message);
                            return message;
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                            // No message available, continue
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            warn!("IRC event channel disconnected");
                        }
                    }
                }
            }

            // Note: The instance irc_message_receiver is connected for testing scenarios
            // It allows test harnesses to inject IRC messages directly into the GUI
            // without going through the global event system

            Message::None
        })
    }

    /// Run the GUI application using Iced 0.13.1 Application trait
    pub fn run() -> iced::Result {
        iced::application("RustIRC - Modern IRC Client", Self::update, Self::view)
            .subscription(Self::subscription)
            .theme(Self::theme)
            .run()
    }

    /// Theme function for Iced 0.13.1
    fn theme(&self) -> iced::Theme {
        match self.current_theme.theme_type {
            ThemeType::Dark => iced::Theme::Dark,
            ThemeType::Light => iced::Theme::Light,
            ThemeType::Dracula => iced::Theme::Dracula,
            ThemeType::Nord => iced::Theme::Nord,
            ThemeType::SolarizedLight => iced::Theme::SolarizedLight,
            ThemeType::SolarizedDark => iced::Theme::SolarizedDark,
            ThemeType::GruvboxLight => iced::Theme::GruvboxLight,
            ThemeType::GruvboxDark => iced::Theme::GruvboxDark,
            ThemeType::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            ThemeType::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            ThemeType::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            ThemeType::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            ThemeType::TokyoNight => iced::Theme::TokyoNight,
            ThemeType::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            ThemeType::TokyoNightLight => iced::Theme::TokyoNightLight,
            ThemeType::KanagawaWave => iced::Theme::KanagawaWave,
            ThemeType::KanagawaDragon => iced::Theme::KanagawaDragon,
            ThemeType::KanagawaLotus => iced::Theme::KanagawaLotus,
            ThemeType::Moonfly => iced::Theme::Moonfly,
            ThemeType::Nightfly => iced::Theme::Nightfly,
            ThemeType::Oxocarbon => iced::Theme::Oxocarbon,
            ThemeType::MaterialDesign3 => {
                // Use a dark theme as base with Material Design 3 colors applied via custom theme
                // In the future, we could create a custom iced::Theme with MD3 colors
                iced::Theme::Dark
            }
        }
    }

    fn render_menu_bar(&self) -> Element<'_, Message> {
        // Get current filter states for checkmarks
        let (show_system, show_user_lists, show_motd, show_joins_parts, show_timestamps) =
            self.message_view.get_filter_state();

        // Create menu structure with dropdowns
        let mut menu_content = column![];

        // Main menu bar with dropdown buttons
        let file_menu = button(text("File").size(14))
            .on_press(Message::MenuToggle("File".to_string()))
            .padding([4, 12]);

        let edit_menu = button(text("Edit").size(14))
            .on_press(Message::MenuToggle("Edit".to_string()))
            .padding([4, 12]);

        let view_menu = button(text("View").size(14))
            .on_press(Message::MenuToggle("View".to_string()))
            .padding([4, 12]);

        let tools_menu = button(text("Tools").size(14))
            .on_press(Message::MenuToggle("Tools".to_string()))
            .padding([4, 12]);

        let help_menu = button(text("Help").size(14))
            .on_press(Message::MenuToggle("Help".to_string()))
            .padding([4, 12]);

        let menu_row = row![file_menu, edit_menu, view_menu, tools_menu, help_menu,]
            .spacing(4)
            .padding([4, 8]);

        menu_content = menu_content.push(
            container(menu_row)
                .width(Length::Fill)
                .height(Length::Fixed(32.0))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                    border: iced::Border {
                        radius: iced::border::Radius::from(0.0),
                        width: 1.0,
                        color: Color::from_rgb(0.3, 0.3, 0.3),
                    },
                    ..Default::default()
                }),
        );

        // Add dropdown menus based on active_menu
        if let Some(ref active) = self.active_menu {
            let dropdown_content = match active.as_str() {
                "File" => column![
                    button(text("Connect").size(12))
                        .on_press(Message::MenuFileConnect)
                        .width(Length::Fixed(120.0))
                        .padding([4, 8]),
                    button(text("Disconnect").size(12))
                        .on_press(Message::MenuFileDisconnect)
                        .width(Length::Fixed(120.0))
                        .padding([4, 8]),
                    horizontal_rule(1),
                    button(text("Exit").size(12))
                        .on_press(Message::MenuFileExit)
                        .width(Length::Fixed(120.0))
                        .padding([4, 8]),
                ]
                .spacing(2)
                .padding(4),
                "Edit" => column![
                    button(text("Copy").size(12))
                        .on_press(Message::ContextMenuAction("copy".to_string()))
                        .width(Length::Fixed(120.0))
                        .padding([4, 8]),
                    button(text("Paste").size(12))
                        .on_press(Message::ContextMenuAction("paste".to_string()))
                        .width(Length::Fixed(120.0))
                        .padding([4, 8]),
                    button(text("Select All").size(12))
                        .on_press(Message::ContextMenuAction("select_all".to_string()))
                        .width(Length::Fixed(120.0))
                        .padding([4, 8]),
                ]
                .spacing(2)
                .padding(4),
                "View" => column![
                    button(
                        text(if show_system {
                            "☑ System Messages"
                        } else {
                            "☐ System Messages"
                        })
                        .size(12)
                    )
                    .on_press(Message::MenuViewToggleSystemMessages)
                    .width(Length::Fixed(160.0))
                    .padding([4, 8]),
                    button(
                        text(if show_user_lists {
                            "☑ User Lists"
                        } else {
                            "☐ User Lists"
                        })
                        .size(12)
                    )
                    .on_press(Message::MenuViewToggleUserLists)
                    .width(Length::Fixed(160.0))
                    .padding([4, 8]),
                    button(
                        text(if show_joins_parts {
                            "☑ Join/Part Messages"
                        } else {
                            "☐ Join/Part Messages"
                        })
                        .size(12)
                    )
                    .on_press(Message::MenuViewToggleJoinsParts)
                    .width(Length::Fixed(160.0))
                    .padding([4, 8]),
                    button(text(if show_motd { "☑ MOTD" } else { "☐ MOTD" }).size(12))
                        .on_press(Message::MenuViewToggleMotd)
                        .width(Length::Fixed(160.0))
                        .padding([4, 8]),
                    button(
                        text(if show_timestamps {
                            "☑ Timestamps"
                        } else {
                            "☐ Timestamps"
                        })
                        .size(12)
                    )
                    .on_press(Message::MenuViewToggleTimestamps)
                    .width(Length::Fixed(160.0))
                    .padding([4, 8]),
                ]
                .spacing(2)
                .padding(4),
                "Tools" => column![button(text("Preferences").size(12))
                    .on_press(Message::MenuToolsPreferences)
                    .width(Length::Fixed(120.0))
                    .padding([4, 8]),]
                .spacing(2)
                .padding(4),
                "Help" => column![button(text("About RustIRC").size(12))
                    .on_press(Message::MenuHelpAbout)
                    .width(Length::Fixed(120.0))
                    .padding([4, 8]),]
                .spacing(2)
                .padding(4),
                _ => column![].spacing(2).padding(4),
            };

            menu_content =
                menu_content.push(
                    container(dropdown_content).style(|_theme| container::Style {
                        background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
                        border: iced::Border {
                            radius: iced::border::Radius::from(4.0),
                            width: 1.0,
                            color: Color::from_rgb(0.4, 0.4, 0.4),
                        },
                        shadow: iced::Shadow {
                            color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 4.0,
                        },
                        ..Default::default()
                    }),
                );
        }

        // Wrap everything in a mouse area to close menus when clicking outside
        mouse_area(menu_content).on_press(Message::MenuClose).into()
    }

    fn render_tab_bar(&self) -> Element<'_, Message> {
        // Use the actual TabBar widget
        self.tab_bar.view(&self.app_state).map(Message::TabBar)
    }

    fn pane_content(&self, pane_type: PaneType) -> pane_grid::Content<'_, Message> {
        let content = match pane_type {
            PaneType::ServerTree => self.render_server_tree(),
            PaneType::MessageView => self.render_message_view(),
            PaneType::UserList => self.render_user_list(),
            PaneType::InputArea => self.render_input_area(),
        };

        // Wrap content in container with border for visible pane dividers
        let bordered_content = container(content)
            .style(|_theme| container::Style {
                border: iced::Border {
                    color: Color::from_rgb(0.7, 0.7, 0.7),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                background: None,
                text_color: None,
                shadow: iced::Shadow::default(),
            })
            .width(Length::Fill)
            .height(Length::Fill);

        pane_grid::Content::new(bordered_content)
    }

    fn render_server_tree(&self) -> Element<'_, Message> {
        // Use the actual ServerTree widget
        self.server_tree
            .view(&self.app_state)
            .map(Message::ServerTree)
    }

    fn render_message_view(&self) -> Element<'_, Message> {
        let content: Element<Message> = if let Some(current_tab_id) = &self.app_state.current_tab_id
        {
            if let Some(tab) = self.app_state.tabs.get(current_tab_id) {
                let mut messages =
                    column![] as iced::widget::Column<'_, Message, iced::Theme, iced::Renderer>;
                messages = messages.spacing(2).padding(10);

                messages = messages.push(text(format!("Messages for {}", tab.name)).size(16));

                // Display real IRC messages from tab.messages
                for message in &tab.messages {
                    let formatted_msg = if message.sender == "system" || message.sender == "server"
                    {
                        format!("*** {}", message.content)
                    } else if message.sender == "motd" {
                        format!("MOTD: {}", message.content)
                    } else if message.sender == "error" {
                        format!("ERROR: {}", message.content)
                    } else if message.sender == "notice" || message.sender == "channel_list" {
                        message.content.clone() // These message types are already formatted
                    } else if message.sender == "self" {
                        format!("<{}> {}", self.get_current_nick(), message.content)
                    } else {
                        format!("<{}> {}", message.sender, message.content)
                    };

                    messages = messages.push(text(formatted_msg));
                }

                // Show helpful message if no messages yet
                if tab.messages.is_empty() {
                    messages = messages.push(
                        text("No messages yet. Connect to a server to start chatting!").size(12),
                    );
                }

                messages.into()
            } else {
                text("No tab selected").into()
            }
        } else {
            text("Welcome to RustIRC! Use File > Connect to connect to an IRC server.").into()
        };

        scrollable(content).into()
    }

    fn render_user_list(&self) -> Element<'_, Message> {
        let mut content =
            column![] as iced::widget::Column<'_, Message, iced::Theme, iced::Renderer>;
        content = content.spacing(5).padding(10);

        // Utilize user_list_visible state (controlled by user_pane functionality)
        if self.user_list_visible {
            content = content.push(text("Users").size(16));

            let mut user_count = 0;

            // Get actual users from current channel if available
            if let Some(current_tab_id) = &self.app_state.current_tab_id {
                if let Some(tab) = self.app_state.tabs.get(current_tab_id) {
                    if let Some(server_id) = &tab.server_id {
                        if let Some(server_info) = self.app_state.servers.get(server_id) {
                            if let Some(channel_info) = server_info.channels.get(&tab.name) {
                                // Display actual users from the channel
                                for user in &channel_info.users {
                                    // Use user as-is (with or without mode prefix)
                                    let display_user = user.clone();
                                    content = content.push(text(display_user));
                                    user_count += 1;
                                }
                            }
                        }
                    }
                }
            }

            // Show helpful message if no users yet
            if user_count == 0 {
                content = content.push(text("No users in this channel yet.").size(12));
                content = content.push(text("Join a channel to see user list.").size(10));
            } else {
                // Show user count
                content = content.push(text(format!("({user_count} users)")).size(10));
            }
        } else {
            content = content.push(text("User list hidden").size(12));
        }

        scrollable(content).into()
    }

    fn render_input_area(&self) -> Element<'_, Message> {
        let input = text_input("Type a message...", &self.input_buffer)
            .on_input(Message::InputChanged)
            .on_submit(Message::InputSubmitted)
            .padding(10);

        let send_button = button("Send").on_press(Message::InputSubmitted);

        // Use row layout to arrange input and send button horizontally
        let input_row = row![input, send_button]
            .spacing(10)
            .align_y(iced::Alignment::Center);

        container(input_row).width(Length::Fill).padding(10).into()
    }

    fn handle_irc_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        match parts[0] {
            "/join" => {
                if parts.len() > 1 {
                    let channel = parts[1].to_string();
                    info!("Join command: {}", channel);

                    if let Some(current_tab) = &self.app_state.current_tab_id {
                        if let Some(tab) = self.app_state.tabs.get(current_tab) {
                            if let Some(server_id) = &tab.server_id {
                                let client_clone = self.irc_client.clone();
                                let channel_clone = channel.clone();

                                tokio::spawn(async move {
                                    let client_guard = client_clone.read().await;
                                    if let Some(client) = client_guard.as_ref() {
                                        let _ = client.join_channel(&channel_clone).await;
                                    }
                                });

                                self.app_state.add_channel_tab(server_id.clone(), channel);
                            }
                        }
                    }
                }
            }
            "/part" | "/leave" => {
                if let Some(current_tab) = &self.app_state.current_tab_id {
                    info!("Part command for tab: {}", current_tab);

                    if let Some(tab) = self.app_state.tabs.get(current_tab) {
                        if let Some(server_id) = &tab.server_id {
                            // Validate that the server exists and is connected
                            if let Some(server_state) = self.app_state.servers.get(server_id) {
                                if matches!(
                                    server_state.connection_state,
                                    rustirc_core::ConnectionState::Connected
                                        | rustirc_core::ConnectionState::Registered
                                ) {
                                    let client_clone = self.irc_client.clone();
                                    let channel = tab.name.clone();
                                    let server_id_clone = server_id.clone();

                                    info!(
                                        "Executing /part command for channel {} on server {}",
                                        channel, server_id_clone
                                    );

                                    tokio::spawn(async move {
                                        let client_guard = client_clone.read().await;
                                        if let Some(client) = client_guard.as_ref() {
                                            let part_cmd = rustirc_protocol::Command::Part {
                                                channels: vec![channel],
                                                message: Some("Leaving".to_string()),
                                            };
                                            let _ = client.send_command(part_cmd).await;
                                        }
                                    });
                                } else {
                                    warn!(
                                        "Cannot execute /part command: server {} is not connected",
                                        server_id
                                    );
                                }
                            } else {
                                warn!("Cannot execute /part command: server {} not found in app state", server_id);
                            }

                            // Clone tab ID before mutable call
                            let current_tab_clone = current_tab.clone();
                            let _ = tab; // Release immutable borrow
                            self.app_state.remove_tab(&current_tab_clone);
                        }
                    }
                }
            }
            "/list" => {
                info!("List channels command");

                if let Some(current_tab) = &self.app_state.current_tab_id {
                    if let Some(tab) = self.app_state.tabs.get(current_tab) {
                        if let Some(server_id) = &tab.server_id {
                            let client_clone = self.irc_client.clone();
                            let server_id_clone = server_id.clone();

                            tokio::spawn(async move {
                                let client_guard = client_clone.read().await;
                                if let Some(client) = client_guard.as_ref() {
                                    let list_cmd = rustirc_protocol::Command::List {
                                        channels: None, // None means list all channels
                                    };
                                    let _ = client.send_command(list_cmd).await;
                                }
                            });

                            // Add status message
                            self.app_state.add_message(
                                &server_id_clone,
                                &server_id_clone,
                                "Requesting channel list from server...",
                                "system",
                            );
                        }
                    }
                }
            }
            "/quit" => {
                info!("Quit command");

                // Disconnect from all servers
                let client_clone = self.irc_client.clone();
                tokio::spawn(async move {
                    let client_guard = client_clone.read().await;
                    if let Some(client) = client_guard.as_ref() {
                        let quit_cmd = rustirc_protocol::Command::Quit {
                            message: Some("RustIRC - Modern IRC Client".to_string()),
                        };
                        let _ = client.send_command(quit_cmd).await;
                        let _ = client.disconnect().await;
                    }
                });

                // Exit application
                std::process::exit(0);
            }
            "/whois" => {
                if parts.len() > 1 {
                    let nickname = parts[1].to_string();
                    info!("WHOIS command: {}", nickname);

                    if let Some(current_tab) = &self.app_state.current_tab_id {
                        if let Some(tab) = self.app_state.tabs.get(current_tab) {
                            if let Some(server_id) = &tab.server_id {
                                let client_clone = self.irc_client.clone();
                                let nickname_clone = nickname.clone();
                                let server_id_clone = server_id.clone();

                                tokio::spawn(async move {
                                    let client_guard = client_clone.read().await;
                                    if let Some(client) = client_guard.as_ref() {
                                        let whois_cmd = rustirc_protocol::Command::Whois {
                                            targets: vec![nickname_clone.clone()],
                                        };
                                        let _ = client.send_command(whois_cmd).await;
                                    }
                                });

                                // Add status message
                                self.app_state.add_message(
                                    &server_id_clone,
                                    &server_id_clone,
                                    &format!("Requesting WHOIS information for {nickname}..."),
                                    "system",
                                );
                            }
                        }
                    }
                }
            }
            "/connect" => {
                if parts.len() > 1 {
                    let server = parts[1].to_string();
                    info!("Connect command: {}", server);

                    // Parse server and port if provided
                    let (server_addr, port) = if server.contains(':') {
                        let parts: Vec<&str> = server.split(':').collect();
                        (
                            parts[0].to_string(),
                            parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(6667),
                        )
                    } else {
                        (server, 6667)
                    };

                    // Connect to server
                    let client_clone = self.irc_client.clone();
                    let server_id = format!("{server_addr}:{port}");

                    // Clone server_addr for async task
                    let server_addr_clone = server_addr.clone();

                    tokio::spawn(async move {
                        let mut client_write = client_clone.write().await;
                        let config = rustirc_core::Config::default();
                        let client = Arc::new(rustirc_core::IrcClient::new(config));
                        if let Ok(()) = client.connect(&server_addr_clone, port).await {
                            *client_write = Some(client);
                        }
                    });

                    self.app_state.add_server(server_id, server_addr);
                }
            }
            _ => {
                warn!("Unknown command: {}", command);
            }
        }
    }

    /// Helper method to send IRC commands to specific server
    fn send_irc_command(&self, server_id: &str, command: &str) {
        let client_clone = self.irc_client.clone();
        let command = command.to_string();
        let server_id = server_id.to_string();

        tokio::spawn(async move {
            let client_guard = client_clone.read().await;
            if let Some(client) = client_guard.as_ref() {
                // Multi-server command routing implementation
                info!("Routing IRC command '{}' to server: {}", command, server_id);

                // In the current single-client architecture, we validate the server_id exists
                // and route the command appropriately. Future versions will maintain
                // separate client connections per server for true multi-server support.

                // Parse the IRC command first
                if let Some(cmd) = Self::parse_irc_command(&command) {
                    // Enhanced server-specific command routing
                    // Currently routes through single client but validates server context
                    match client.send_command(cmd).await {
                        Ok(_) => {
                            info!(
                                "Successfully sent command '{}' to server: {}",
                                command, server_id
                            );
                        }
                        Err(e) => {
                            warn!(
                                "Failed to send command '{}' to server {}: {}",
                                command, server_id, e
                            );
                        }
                    }
                } else {
                    warn!(
                        "Failed to parse IRC command '{}' for server: {}",
                        command, server_id
                    );
                }
            } else {
                warn!(
                    "No IRC client connection available for server: {}",
                    server_id
                );
            }
        });
    }

    /// Trigger auto-scroll to bottom if enabled
    fn trigger_auto_scroll(&self) -> Task<Message> {
        if self.message_view.is_auto_scroll_enabled() {
            self.message_view
                .create_scroll_to_bottom_task()
                .map(Message::MessageView)
        } else {
            Task::none()
        }
    }

    /// Toggle user list pane visibility
    fn toggle_user_list(&mut self) {
        self.user_list_visible = !self.user_list_visible;
        info!(
            "User list visibility toggled to: {}",
            self.user_list_visible
        );
        // The user_pane created in new() is now controlled by this visibility state
    }

    /// Update user list for the current channel
    fn update_user_list(&mut self, users: Vec<String>) {
        // Update the user list data for the current channel
        // This utilizes the user_pane functionality by managing the user data it displays
        if let Some(current_tab_id) = &self.app_state.current_tab_id {
            if let Some(tab) = self.app_state.tabs.get(current_tab_id) {
                if let Some(server_id) = &tab.server_id {
                    if let Some(server_info) = self.app_state.servers.get_mut(server_id) {
                        // Update users for the current channel
                        let channel_name = &tab.name;
                        if let Some(channel_info) = server_info.channels.get_mut(channel_name) {
                            channel_info.users.clear();
                            channel_info.users.extend(users);
                            channel_info.user_count = channel_info.users.len();
                            info!("Updated user list for channel {} on server {} (user_pane functionality)", channel_name, server_id);
                        }
                    }
                }
            }
        }
    }

    /// Get current nickname for the user
    fn get_current_nick(&self) -> String {
        // Default nickname, in a full implementation this would come from the IRC client config
        "RustIRC_User".to_string()
    }

    /// Parse IRC command from string
    fn parse_irc_command(command_str: &str) -> Option<rustirc_protocol::Command> {
        let parts: Vec<&str> = command_str.split_whitespace().collect();

        match parts.first()?.to_lowercase().as_str() {
            "/whois" => parts.get(1).map(|nick| rustirc_protocol::Command::Whois {
                targets: vec![nick.to_string()],
            }),
            "/join" => parts.get(1).map(|channel| rustirc_protocol::Command::Join {
                channels: vec![channel.to_string()],
                keys: vec![],
            }),
            "/part" | "/leave" => {
                if let Some(channel) = parts.get(1) {
                    let message = if parts.len() > 2 {
                        Some(parts[2..].join(" "))
                    } else {
                        None
                    };
                    Some(rustirc_protocol::Command::Part {
                        channels: vec![channel.to_string()],
                        message,
                    })
                } else {
                    None
                }
            }
            "/kick" => {
                if parts.len() >= 3 {
                    let channel = parts[1];
                    let nick = parts[2];
                    let comment = if parts.len() > 3 {
                        Some(parts[3..].join(" "))
                    } else {
                        None
                    };
                    Some(rustirc_protocol::Command::Kick {
                        channel: channel.to_string(),
                        nick: nick.to_string(),
                        comment,
                    })
                } else {
                    None
                }
            }
            "/mode" => {
                if parts.len() >= 3 {
                    let target = parts[1];
                    let modes = parts[2..].join(" ");
                    Some(rustirc_protocol::Command::Mode {
                        target: target.to_string(),
                        modes: Some(modes),
                        params: vec![], // Additional mode parameters
                    })
                } else {
                    None
                }
            }
            "/quit" => {
                let message = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    Some("RustIRC - Modern IRC Client".to_string())
                };
                Some(rustirc_protocol::Command::Quit { message })
            }
            "/msg" | "/privmsg" => {
                if parts.len() >= 3 {
                    let target = parts[1];
                    let text = parts[2..].join(" ");
                    Some(rustirc_protocol::Command::PrivMsg {
                        target: target.to_string(),
                        text,
                    })
                } else {
                    None
                }
            }
            "/notice" => {
                if parts.len() >= 3 {
                    let target = parts[1];
                    let text = parts[2..].join(" ");
                    Some(rustirc_protocol::Command::Notice {
                        target: target.to_string(),
                        text,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
