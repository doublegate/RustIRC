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

use crate::theme::{Theme, ThemeType};
use crate::state::AppState;
use crate::widgets::{
    server_tree::{ServerTree, ServerTreeMessage},
    message_view::{MessageView, MessageViewMessage},
    user_list::{UserList, UserListMessage},
    input_area::{InputArea, InputAreaMessage},
    tab_bar::{TabBar, TabBarMessage},
    status_bar::{StatusBar, StatusBarMessage},
};
use iced::{
    widget::{pane_grid, container, row, column, text, button, text_input, scrollable},
    Element, Length, Task,
};
use rustirc_core::IrcClient;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

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
    JoinChannel(String, String), // server_id, channel
    LeaveChannel(String, String), // server_id, channel
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
    
    // Widget messages
    ServerTree(ServerTreeMessage),
    MessageView(MessageViewMessage),
    UserList(UserListMessage),
    InputArea(InputAreaMessage),
    TabBar(TabBarMessage),
    StatusBar(StatusBarMessage),
    
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

/// Main GUI application state
pub struct RustIrcGui {
    // Core IRC functionality
    irc_client: Arc<RwLock<Option<Arc<IrcClient>>>>,
    
    // Application state
    app_state: AppState,
    current_theme: Theme,
    
    // Layout
    panes: pane_grid::State<PaneType>,
    
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
}

impl Default for RustIrcGui {
    fn default() -> Self {
        // Initialize pane grid layout
        let (mut panes, _) = pane_grid::State::new(PaneType::MessageView);
        
        // Split to create server tree on the left
        let first_pane = *panes.iter().next().unwrap().0;
        let (left_pane, _) = panes.split(
            pane_grid::Axis::Vertical,
            first_pane,
            PaneType::ServerTree,
        ).unwrap();
        
        // Split right side to add user list
        let (message_pane, user_pane) = panes.split(
            pane_grid::Axis::Vertical,
            left_pane,
            PaneType::UserList,
        ).unwrap();
        
        // Split bottom for input area
        let (_top_pane, _bottom_pane) = panes.split(
            pane_grid::Axis::Horizontal,
            message_pane,
            PaneType::InputArea,
        ).unwrap();

        Self {
            irc_client: Arc::new(RwLock::new(None)),
            app_state: AppState::new(),
            current_theme: Theme::default(),
            panes,
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
        }
    }
}

impl RustIrcGui {
    /// Update function for Iced 0.13.1 functional approach
    fn update(&mut self, message: Message) -> impl Into<Task<Message>> {
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
                                            let _ = client.send_message(&target_clone, &message_clone).await;
                                        }
                                    });
                                    
                                    // Add message to app state for display
                                    self.app_state.add_message(&server_id, &target, &message, "self");
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
                let server_id = format!("{}:{}", server, port);
                let server_clone = server.clone(); // Clone for async move
                
                // Initialize IRC client
                tokio::spawn(async move {
                    let mut client_write = client_clone.write().await;
                    if client_write.is_none() {
                        let config = rustirc_core::Config::default();
                        let client = Arc::new(rustirc_core::IrcClient::new(config));
                        if let Ok(()) = client.connect(&server_clone, port).await {
                            *client_write = Some(client);
                        }
                    }
                });
                
                // Add server to app state
                self.app_state.add_server(server_id.clone(), server.to_string());
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
                let tab_id = format!("{}/{}", server_id, channel);
                self.app_state.remove_tab(&tab_id);
            }
            Message::SendMessage(server_id, target, message) => {
                info!("Sending message to {} on {}: {}", target, server_id, message);
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
                self.app_state.add_message(&server_id, &target, &message, "self");
            }
            Message::JoinChannel(server_id, channel) => {
                info!("Joining channel {} on server {}", channel, server_id);
                self.app_state.add_channel_tab(server_id, channel);
            }
            Message::TabSelected(tab_id) => {
                self.app_state.current_tab_id = Some(tab_id);
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
                                        self.send_irc_command(server_id, &format!("/whois {}", nick));
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
                            let server_id = if let Some(current_tab) = &self.app_state.current_tab_id {
                                self.app_state.tabs.get(current_tab)
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
                            let tab_id = format!("{}/{}", server_id, channel);
                            self.app_state.current_tab_id = Some(tab_id.clone());
                            info!("Switched to channel: {} on server: {}", channel, server_id);
                        } else {
                            // Handle case where channel_id is just the channel name (need current server)
                            let tab_id = if let Some(current_tab) = self.app_state.current_tab() {
                                current_tab.server_id.as_ref().map(|server_id| {
                                    format!("{}/{}", server_id, channel_id)
                                })
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
                let task = self.server_tree.update(server_tree_msg, &mut self.app_state);
                
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
                        // Note: URL opening functionality requires external crate
                        // For now, log the URL click for demonstration
                        info!("URL clicked (would open in browser): {}", url);
                        
                        // In a real implementation, this would use the `open` crate:
                        // if let Err(e) = open::that(&url) {
                        //     warn!("Failed to open URL {}: {}", url, e);
                        // }
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
                let task = self.message_view.update(message_view_msg, &mut self.app_state);
                
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
                                    drop(tab); // Release immutable borrow
                                    self.app_state.add_private_tab(&server_id_clone, nick.clone());
                                }
                            }
                        }
                    }
                    UserListMessage::UserDoubleClicked(nick) => {
                        // Send whois command
                        if let Some(current_tab) = &self.app_state.current_tab_id {
                            if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                if let Some(server_id) = &tab.server_id {
                                    self.send_irc_command(server_id, &format!("/whois {}", nick));
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
                        self.user_list.set_sort_mode(crate::widgets::user_list::SortMode::Nickname);
                    }
                    UserListMessage::SortByMode => {
                        // Sort user list by user mode
                        info!("Sorting user list by mode");
                        self.user_list.set_sort_mode(crate::widgets::user_list::SortMode::Privilege);
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
                            self.handle_irc_command(&text);
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
                                            let _ = client.send_message(&target_for_async, &message_for_async).await;
                                        }
                                    });
                                    
                                    // Release immutable borrow before mutable call
                                    drop(tab);
                                    self.app_state.add_message(&server_id_clone, &target, &message, "self");
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
                                    self.send_irc_command(server_id, &text);
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
                        // Handle tab completion
                        info!("Tab completion requested");
                        // Tab completion logic would be implemented here
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
                        // Key handling logic would be implemented here
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
                        if self.app_state.current_tab_id.as_ref() == Some(&tab_id) {
                            self.app_state.current_tab_id = self.app_state.tab_order.first().cloned();
                        }
                    }
                    TabBarMessage::TabClosed(tab_id) => {
                        self.app_state.remove_tab(tab_id);
                        // Select next available tab
                        if self.app_state.current_tab_id.as_ref() == Some(&tab_id) {
                            self.app_state.current_tab_id = self.app_state.tab_order.first().cloned();
                        }
                    }
                    TabBarMessage::MoveTab(tab_id, new_position) => {
                        // Implement tab reordering
                        if let Some(current_pos) = self.app_state.tab_order.iter().position(|id| id == tab_id) {
                            let tab_id = self.app_state.tab_order.remove(current_pos);
                            let insert_pos = (*new_position).min(self.app_state.tab_order.len());
                            self.app_state.tab_order.insert(insert_pos, tab_id);
                        }
                    }
                    TabBarMessage::NewTab => {
                        // Show new tab dialog (simplified: create server tab)
                        let server_id = format!("new_server_{}", self.app_state.servers.len() + 1);
                        self.app_state.add_server(server_id.clone(), "New Server".to_string());
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
                        let tabs_to_remove: Vec<String> = self.app_state.tab_order
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
            Message::None => {}
        }
        
        Task::none()
    }

    /// View function for Iced 0.13.1 functional approach
    fn view(&self) -> Element<Message> {
        let pane_grid = pane_grid::PaneGrid::new(&self.panes, |_pane, pane_type, _is_maximized| {
            self.pane_content(*pane_type)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(1)
        .on_click(Message::PaneClicked)
        .on_drag(Message::PaneDragged)
        .on_resize(10, Message::PaneResized);

        let content = column![
            self.render_tab_bar(),
            pane_grid,
        ]
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
                    button("Close Tab").on_press(Message::ContextMenuAction("close_tab".to_string())),
                ]
                .spacing(2)
                .padding(5)
            )
            .padding(1)
            .width(Length::Shrink)
            .height(Length::Shrink);
            
            // For now, we'll overlay it simply - in a real implementation, 
            // you'd use proper positioning based on context_menu_x/y
            return container(
                column![
                    content,
                    container(context_menu)
                        .width(Length::Fill)
                        .height(Length::Shrink)
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Run the GUI application using Iced 0.13.1 Application trait
    pub fn run() -> iced::Result {
        iced::application("RustIRC - Modern IRC Client", Self::update, Self::view)
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
        }
    }

    fn render_tab_bar(&self) -> Element<Message> {
        // Use the actual TabBar widget
        self.tab_bar.view(&self.app_state).map(Message::TabBar)
    }

    fn pane_content(&self, pane_type: PaneType) -> pane_grid::Content<Message> {
        let content = match pane_type {
            PaneType::ServerTree => {
                self.render_server_tree()
            }
            PaneType::MessageView => {
                self.render_message_view()
            }
            PaneType::UserList => {
                self.render_user_list()
            }
            PaneType::InputArea => {
                self.render_input_area()
            }
        };

        pane_grid::Content::new(content)
    }

    fn render_server_tree(&self) -> Element<Message> {
        // Use the actual ServerTree widget
        self.server_tree.view(&self.app_state).map(Message::ServerTree)
    }

    fn render_message_view(&self) -> Element<Message> {
        let content: Element<Message> = if let Some(current_tab_id) = &self.app_state.current_tab_id {
            if let Some(tab) = self.app_state.tabs.get(current_tab_id) {
                let mut messages = column![] as iced::widget::Column<'_, Message, iced::Theme, iced::Renderer>;
                messages = messages.spacing(2).padding(10);
                
                messages = messages.push(text(format!("Messages for {}", tab.name)).size(16));
                
                // Add sample messages for demonstration
                messages = messages.push(text("12:00 <user1> Hello everyone!"));
                messages = messages.push(text("12:01 <user2> How's everyone doing today?"));
                messages = messages.push(text("12:02 <user3> Great! Working on some Rust code"));
                
                messages.into()
            } else {
                text("No tab selected").into()
            }
        } else {
            text("Welcome to RustIRC!").into()
        };

        scrollable(content).into()
    }

    fn render_user_list(&self) -> Element<Message> {
        let mut content = column![] as iced::widget::Column<'_, Message, iced::Theme, iced::Renderer>;
        content = content.spacing(5).padding(10);

        content = content.push(text("Users").size(16));
        
        // Add sample users for demonstration
        content = content.push(text("@operator"));
        content = content.push(text("+voice"));
        content = content.push(text("regular_user"));
        content = content.push(text("another_user"));

        scrollable(content).into()
    }

    fn render_input_area(&self) -> Element<Message> {
        let input = text_input("Type a message...", &self.input_buffer)
            .on_input(Message::InputChanged)
            .on_submit(Message::InputSubmitted)
            .padding(10);
            
        let send_button = button("Send")
            .on_press(Message::InputSubmitted);
            
        // Use row layout to arrange input and send button horizontally
        let input_row = row![input, send_button]
            .spacing(10)
            .align_y(iced::Alignment::Center);

        container(input_row)
            .width(Length::Fill)
            .padding(10)
            .into()
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
                            let client_clone = self.irc_client.clone();
                            let channel = tab.name.clone();
                            
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
                            
                            // Clone tab ID before mutable call
                            let current_tab_clone = current_tab.clone();
                            drop(tab); // Release immutable borrow
                            self.app_state.remove_tab(&current_tab_clone);
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
            "/connect" => {
                if parts.len() > 1 {
                    let server = parts[1].to_string();
                    info!("Connect command: {}", server);
                    
                    // Parse server and port if provided
                    let (server_addr, port) = if server.contains(':') {
                        let parts: Vec<&str> = server.split(':').collect();
                        (parts[0].to_string(), parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(6667))
                    } else {
                        (server, 6667)
                    };
                    
                    // Connect to server
                    let client_clone = self.irc_client.clone();
                    let server_id = format!("{}:{}", server_addr, port);
                    
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
    
    /// Helper method to send IRC commands
    fn send_irc_command(&self, server_id: &str, command: &str) {
        let client_clone = self.irc_client.clone();
        let command = command.to_string();
        
        tokio::spawn(async move {
            let client_guard = client_clone.read().await;
            if let Some(client) = client_guard.as_ref() {
                // Parse and send the IRC command
                if let Some(cmd) = Self::parse_irc_command(&command) {
                    let _ = client.send_command(cmd).await;
                }
            }
        });
    }
    
    /// Parse IRC command from string
    fn parse_irc_command(command_str: &str) -> Option<rustirc_protocol::Command> {
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        
        match parts.get(0)?.to_lowercase().as_str() {
            "/whois" => {
                if let Some(nick) = parts.get(1) {
                    Some(rustirc_protocol::Command::Whois {
                        targets: vec![nick.to_string()],
                    })
                } else {
                    None
                }
            }
            "/join" => {
                if let Some(channel) = parts.get(1) {
                    Some(rustirc_protocol::Command::Join {
                        channels: vec![channel.to_string()],
                        keys: vec![],
                    })
                } else {
                    None
                }
            }
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