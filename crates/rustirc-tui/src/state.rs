//! TUI state management
//!
//! Manages the state of the TUI interface including:
//! - Server connections and channel lists
//! - Current focus and navigation state
//! - Message history and scrolling
//! - Input buffer and command history

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of messages to keep per channel
const MAX_MESSAGES_PER_CHANNEL: usize = 1000;

/// Maximum command history size
const MAX_COMMAND_HISTORY: usize = 100;

/// A message in a channel
#[derive(Debug, Clone)]
pub struct TuiMessage {
    pub nick: String,
    pub content: String,
    pub timestamp: SystemTime,
    pub is_own_message: bool,
    pub is_highlight: bool,
    pub message_type: MessageType,
}

/// Types of messages
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Message,
    Action,
    Join,
    Part,
    Quit,
    Nick,
    Topic,
    Notice,
    System,
}

/// A channel's state
#[derive(Debug, Clone)]
pub struct ChannelState {
    pub name: String,
    pub messages: VecDeque<TuiMessage>,
    pub users: Vec<String>,
    pub topic: Option<String>,
    pub unread_count: usize,
    pub has_highlight: bool,
    pub scroll_position: usize,
}

impl ChannelState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            messages: VecDeque::new(),
            users: Vec::new(),
            topic: None,
            unread_count: 0,
            has_highlight: false,
            scroll_position: 0,
        }
    }

    pub fn add_message(&mut self, message: TuiMessage) {
        // Remove old messages if at capacity
        if self.messages.len() >= MAX_MESSAGES_PER_CHANNEL {
            self.messages.pop_front();
        }

        // Update unread count
        if !message.is_own_message {
            self.unread_count += 1;
            if message.is_highlight {
                self.has_highlight = true;
            }
        }

        self.messages.push_back(message);

        // Auto-scroll to bottom if at the end
        if self.scroll_position == 0 {
            self.scroll_position = 0; // Stay at bottom
        }
    }

    pub fn mark_as_read(&mut self) {
        self.unread_count = 0;
        self.has_highlight = false;
    }
}

/// A server's state
#[derive(Debug, Clone)]
pub struct ServerState {
    pub name: String,
    pub channels: HashMap<String, ChannelState>,
    pub nickname: String,
    pub connected: bool,
    pub current_channel: Option<String>,
}

impl ServerState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            channels: HashMap::new(),
            nickname: "RustIRC".to_string(),
            connected: false,
            current_channel: None,
        }
    }

    pub fn add_channel(&mut self, channel_name: String) {
        let channel = ChannelState::new(channel_name.clone());
        self.channels.insert(channel_name.clone(), channel);

        // Switch to this channel if it's the first one
        if self.current_channel.is_none() {
            self.current_channel = Some(channel_name);
        }
    }

    pub fn remove_channel(&mut self, channel_name: &str) {
        self.channels.remove(channel_name);

        // Switch to another channel if current was removed
        if self.current_channel.as_ref() == Some(&channel_name.to_string()) {
            self.current_channel = self.channels.keys().next().cloned();
        }
    }
}

/// Current focus area in the TUI
#[derive(Debug, Clone, PartialEq)]
pub enum FocusArea {
    ChannelList,
    MessageArea,
    UserList,
    Input,
}

/// Application settings for TUI (matching GUI AppSettings)
#[derive(Debug, Clone)]
pub struct TuiSettings {
    pub theme: String,
    pub font_size: f32,
    pub show_timestamps: bool,
    pub show_join_part: bool,
    pub highlight_words: Vec<String>,
    pub notification_sound: bool,
    pub auto_reconnect: bool,
    pub nick_colors: bool,
    pub timestamp_format: String,
    pub last_message_id: usize,
    pub notification_popup: bool,
    pub compact_mode: bool,
}

impl Default for TuiSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 12.0,
            show_timestamps: true,
            show_join_part: false,
            highlight_words: vec!["RustIRC_User".to_string()],
            notification_sound: true,
            auto_reconnect: true,
            nick_colors: true,
            timestamp_format: "%H:%M:%S".to_string(),
            last_message_id: 0,
            notification_popup: true,
            compact_mode: false,
        }
    }
}

/// TUI UI state (matching GUI UiState)
#[derive(Debug, Clone)]
pub struct TuiUiState {
    pub show_server_tree: bool,
    pub show_user_list: bool,
    pub show_sidebar: bool,
    pub show_userlist: bool,
    pub show_help: bool,
    pub show_status_bar: bool,
}

impl Default for TuiUiState {
    fn default() -> Self {
        Self {
            show_server_tree: true,
            show_user_list: true,
            show_sidebar: true,
            show_userlist: true,
            show_help: false,
            show_status_bar: true,
        }
    }
}

/// Tab information for TUI (matching GUI Tab)
#[derive(Debug, Clone)]
pub struct TuiTab {
    pub id: String,
    pub name: String,
    pub tab_type: TuiTabType,
    pub server_id: Option<String>,
    pub messages: VecDeque<TuiMessage>,
    pub unread_count: usize,
    pub has_highlight: bool,
    pub activity_level: ActivityLevel,
}

/// Tab types for TUI
#[derive(Debug, Clone, PartialEq)]
pub enum TuiTabType {
    Server,
    Channel,
    PrivateMessage,
}

/// Activity levels for tabs
#[derive(Debug, Clone, PartialEq)]
pub enum ActivityLevel {
    None,
    Activity,
    Highlight,
}

impl TuiTab {
    pub fn server(server_id: String) -> Self {
        Self {
            id: format!("server:{server_id}"),
            name: server_id.clone(),
            tab_type: TuiTabType::Server,
            server_id: Some(server_id),
            messages: VecDeque::new(),
            unread_count: 0,
            has_highlight: false,
            activity_level: ActivityLevel::None,
        }
    }

    pub fn channel(server_id: String, channel: String) -> Self {
        Self {
            id: format!("{server_id}:{channel}"),
            name: channel,
            tab_type: TuiTabType::Channel,
            server_id: Some(server_id),
            messages: VecDeque::new(),
            unread_count: 0,
            has_highlight: false,
            activity_level: ActivityLevel::None,
        }
    }

    pub fn private_message(server_id: String, nick: String) -> Self {
        Self {
            id: format!("{server_id}:pm:{nick}"),
            name: nick,
            tab_type: TuiTabType::PrivateMessage,
            server_id: Some(server_id),
            messages: VecDeque::new(),
            unread_count: 0,
            has_highlight: false,
            activity_level: ActivityLevel::None,
        }
    }
}

/// Main TUI state
#[derive(Debug, Clone)]
pub struct TuiState {
    /// Connected servers
    pub servers: HashMap<String, ServerState>,

    /// Open tabs (channels and private messages) - matching GUI
    pub tabs: HashMap<String, TuiTab>,

    /// Currently active tab
    pub current_tab_id: Option<String>,

    /// Tab order for navigation
    pub tab_order: Vec<String>,

    /// Currently active server (legacy, replaced by current_tab_id)
    pub current_server: Option<String>,

    /// Current focus area
    pub focus: FocusArea,

    /// Input buffer
    pub input_buffer: String,

    /// Input cursor position
    pub input_cursor: usize,

    /// Command history
    pub command_history: VecDeque<String>,

    /// Current position in command history
    pub history_position: usize,

    /// Selected channel in channel list
    pub selected_channel_index: usize,

    /// Selected user in user list
    pub selected_user_index: usize,

    /// Application start time for relative timestamps
    pub start_time: SystemTime,

    /// Global settings
    pub settings: TuiSettings,

    /// UI state
    pub ui_state: TuiUiState,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            tabs: HashMap::new(),
            current_tab_id: None,
            tab_order: Vec::new(),
            current_server: None,
            focus: FocusArea::Input,
            input_buffer: String::new(),
            input_cursor: 0,
            command_history: VecDeque::new(),
            history_position: 0,
            selected_channel_index: 0,
            selected_user_index: 0,
            start_time: SystemTime::now(),
            settings: TuiSettings::default(),
            ui_state: TuiUiState::default(),
        }
    }

    /// Get the current tab
    pub fn current_tab(&self) -> Option<&TuiTab> {
        if let Some(tab_id) = &self.current_tab_id {
            self.tabs.get(tab_id)
        } else {
            None
        }
    }

    /// Get application settings
    pub fn settings(&self) -> &TuiSettings {
        &self.settings
    }

    /// Add a server (with tab management)
    pub fn add_server(&mut self, server_name: String) {
        let server = ServerState::new(server_name.clone());
        self.servers.insert(server_name.clone(), server);

        // Create server tab
        let tab = TuiTab::server(server_name.clone());
        let tab_id = format!("server:{server_name}");
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());

        // Set as current tab if it's the first one
        if self.current_tab_id.is_none() {
            self.current_tab_id = Some(tab_id);
        }

        // Legacy support
        if self.current_server.is_none() {
            self.current_server = Some(server_name);
        }
    }

    /// Add a channel tab
    pub fn add_channel_tab(&mut self, server_name: String, channel: String) {
        let tab = TuiTab::channel(server_name.clone(), channel.clone());
        let tab_id = format!("{server_name}:{channel}");
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());

        // Set as current tab
        self.current_tab_id = Some(tab_id);

        // Add channel to server if server exists
        if let Some(server) = self.servers.get_mut(&server_name) {
            server.add_channel(channel);
        }
    }

    /// Add a private message tab
    pub fn add_private_message_tab(&mut self, server_name: String, nick: String) {
        let tab = TuiTab::private_message(server_name.clone(), nick.clone());
        let tab_id = format!("{server_name}:pm:{nick}");
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());

        // Set as current tab
        self.current_tab_id = Some(tab_id);
    }

    /// Remove a tab
    pub fn remove_tab(&mut self, tab_id: &str) {
        self.tabs.remove(tab_id);
        self.tab_order.retain(|id| id != tab_id);

        // If this was current tab, switch to next available
        if self.current_tab_id.as_ref() == Some(&tab_id.to_string()) {
            self.current_tab_id = self.tab_order.first().cloned();
        }
    }

    /// Select a tab
    pub fn select_tab(&mut self, tab_id: String) {
        if self.tabs.contains_key(&tab_id) {
            self.current_tab_id = Some(tab_id);

            // Mark as read
            if let Some(current_id) = &self.current_tab_id {
                if let Some(tab) = self.tabs.get_mut(current_id) {
                    tab.unread_count = 0;
                    tab.has_highlight = false;
                    tab.activity_level = ActivityLevel::None;
                }
            }
        }
    }

    /// Get current tab (mutable)
    pub fn current_tab_mut(&mut self) -> Option<&mut TuiTab> {
        if let Some(tab_id) = &self.current_tab_id {
            let tab_id = tab_id.clone();
            self.tabs.get_mut(&tab_id)
        } else {
            None
        }
    }

    /// Remove a server
    pub fn remove_server(&mut self, server_name: &str) {
        self.servers.remove(server_name);

        // Switch to another server if current was removed
        if self.current_server.as_ref() == Some(&server_name.to_string()) {
            self.current_server = self.servers.keys().next().cloned();
        }
    }

    /// Add a channel to a server
    pub fn add_channel(&mut self, server_name: String, channel_name: String) {
        if let Some(server) = self.servers.get_mut(&server_name) {
            server.add_channel(channel_name);
        }
    }

    /// Remove a channel from a server
    pub fn remove_channel(&mut self, server_name: &str, channel_name: &str) {
        if let Some(server) = self.servers.get_mut(server_name) {
            server.remove_channel(channel_name);
        }
    }

    /// Add a message to a channel
    pub fn add_message(
        &mut self,
        server_name: String,
        channel_name: String,
        nick: String,
        content: String,
    ) {
        if let Some(server) = self.servers.get_mut(&server_name) {
            if let Some(channel) = server.channels.get_mut(&channel_name) {
                let message = TuiMessage {
                    nick,
                    content,
                    timestamp: SystemTime::now(),
                    is_own_message: false,
                    is_highlight: false,
                    message_type: MessageType::Message,
                };
                channel.add_message(message);
            }
        }
    }

    /// Get current server
    pub fn current_server(&self) -> Option<&String> {
        self.current_server.as_ref()
    }

    /// Get current channel
    pub fn current_channel(&self) -> Option<&String> {
        if let Some(server_name) = &self.current_server {
            if let Some(server) = self.servers.get(server_name) {
                return server.current_channel.as_ref();
            }
        }
        None
    }

    /// Get current channel state
    pub fn current_channel_state(&self) -> Option<&ChannelState> {
        if let (Some(server_name), Some(channel_name)) =
            (&self.current_server, self.current_channel())
        {
            if let Some(server) = self.servers.get(server_name) {
                return server.channels.get(channel_name);
            }
        }
        None
    }

    /// Get mutable current channel state
    pub fn current_channel_state_mut(&mut self) -> Option<&mut ChannelState> {
        if let Some(server_name) = self.current_server.clone() {
            if let Some(server) = self.servers.get_mut(&server_name) {
                if let Some(channel_name) = server.current_channel.clone() {
                    return server.channels.get_mut(&channel_name);
                }
            }
        }
        None
    }

    /// Switch to a different channel
    pub fn switch_to_channel(&mut self, server_name: &str, channel_name: &str) {
        if let Some(server) = self.servers.get_mut(server_name) {
            if server.channels.contains_key(channel_name) {
                server.current_channel = Some(channel_name.to_string());
                self.current_server = Some(server_name.to_string());

                // Mark channel as read
                if let Some(channel) = server.channels.get_mut(channel_name) {
                    channel.mark_as_read();
                }
            }
        }
    }

    /// Get all channels for current server
    pub fn current_server_channels(&self) -> Vec<&String> {
        if let Some(server_name) = &self.current_server {
            if let Some(server) = self.servers.get(server_name) {
                return server.channels.keys().collect();
            }
        }
        Vec::new()
    }

    /// Navigate channels
    pub fn next_channel(&mut self) {
        let channels: Vec<String> = self
            .current_server_channels()
            .into_iter()
            .cloned()
            .collect();
        let channels_len = channels.len();
        if !channels.is_empty() {
            self.selected_channel_index = (self.selected_channel_index + 1) % channels_len;
            if let Some(channel_name) = channels.get(self.selected_channel_index) {
                if let Some(server_name) = self.current_server.clone() {
                    self.switch_to_channel(&server_name, channel_name);
                }
            }
        }
    }

    pub fn previous_channel(&mut self) {
        let channels: Vec<String> = self
            .current_server_channels()
            .into_iter()
            .cloned()
            .collect();
        let channels_len = channels.len();
        if !channels.is_empty() {
            self.selected_channel_index = if self.selected_channel_index == 0 {
                channels_len - 1
            } else {
                self.selected_channel_index - 1
            };
            if let Some(channel_name) = channels.get(self.selected_channel_index) {
                if let Some(server_name) = self.current_server.clone() {
                    self.switch_to_channel(&server_name, channel_name);
                }
            }
        }
    }

    /// Handle input
    pub fn insert_char(&mut self, c: char) {
        self.input_buffer.insert(self.input_cursor, c);
        self.input_cursor += 1;
    }

    pub fn delete_char(&mut self) {
        if self.input_cursor > 0 && !self.input_buffer.is_empty() {
            self.input_cursor -= 1;
            self.input_buffer.remove(self.input_cursor);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.input_cursor < self.input_buffer.len() {
            self.input_cursor += 1;
        }
    }

    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.input_cursor = 0;
    }

    /// Submit input and return the command
    pub fn submit_input(&mut self) -> String {
        let command = self.input_buffer.clone();

        // Add to history if not empty and different from last command
        if !command.trim().is_empty() && self.command_history.back() != Some(&command) {
            self.command_history.push_back(command.clone());

            // Limit history size
            if self.command_history.len() > MAX_COMMAND_HISTORY {
                self.command_history.pop_front();
            }
        }

        self.clear_input();
        self.history_position = self.command_history.len();

        command
    }

    /// Navigate command history
    pub fn history_up(&mut self) {
        if !self.command_history.is_empty() && self.history_position > 0 {
            self.history_position -= 1;
            if let Some(command) = self.command_history.get(self.history_position) {
                self.input_buffer = command.clone();
                self.input_cursor = self.input_buffer.len();
            }
        }
    }

    pub fn history_down(&mut self) {
        if self.history_position < self.command_history.len() {
            self.history_position += 1;
            if self.history_position == self.command_history.len() {
                self.clear_input();
            } else if let Some(command) = self.command_history.get(self.history_position) {
                self.input_buffer = command.clone();
                self.input_cursor = self.input_buffer.len();
            }
        }
    }

    /// Focus management
    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            FocusArea::ChannelList => FocusArea::MessageArea,
            FocusArea::MessageArea => FocusArea::UserList,
            FocusArea::UserList => FocusArea::Input,
            FocusArea::Input => FocusArea::ChannelList,
        };
    }

    pub fn set_focus(&mut self, focus: FocusArea) {
        self.focus = focus;
    }

    /// Toggle help screen
    pub fn toggle_help(&mut self) {
        self.ui_state.show_help = !self.ui_state.show_help;
    }

    /// Toggle channel list visibility (placeholder for UI state)
    pub fn toggle_channel_list(&mut self) {
        // This would toggle channel list visibility in a more complex UI
        // For now, we'll just focus on the channel list
        self.set_focus(FocusArea::ChannelList);
    }

    /// Toggle user list visibility (placeholder for UI state)
    pub fn toggle_user_list(&mut self) {
        // This would toggle user list visibility in a more complex UI
        // For now, we'll just focus on the user list
        self.set_focus(FocusArea::UserList);
    }

    /// Update timestamps and other time-based state
    pub fn update_timestamps(&mut self) {
        // Calculate elapsed time since UNIX_EPOCH for relative timestamps
        let current_time = SystemTime::now();
        let epoch_duration = current_time.duration_since(UNIX_EPOCH).unwrap_or_default();

        // Update any time-sensitive UI elements based on elapsed time
        for server in self.servers.values_mut() {
            for channel in server.channels.values_mut() {
                // Check for old messages and mark them as such
                let cutoff_time = current_time - std::time::Duration::from_secs(300); // 5 minutes

                for message in &mut channel.messages {
                    if message.timestamp < cutoff_time {
                        // Could add aging logic here for old messages
                    }
                }
            }
        }

        // Log timestamp update for debugging
        let total_seconds = epoch_duration.as_secs();
        if total_seconds % 60 == 0 {
            // Every minute
            println!("Timestamp update: {total_seconds} seconds since UNIX_EPOCH");
        }
    }

    /// Get total unread message count
    pub fn total_unread_count(&self) -> usize {
        self.servers
            .values()
            .flat_map(|server| server.channels.values())
            .map(|channel| channel.unread_count)
            .sum()
    }

    /// Check if there are any highlights
    pub fn has_highlights(&self) -> bool {
        self.servers
            .values()
            .flat_map(|server| server.channels.values())
            .any(|channel| channel.has_highlight)
    }
}

impl Default for TuiState {
    fn default() -> Self {
        Self::new()
    }
}
