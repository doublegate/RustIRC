//! Application state management for RustIRC GUI
//!
//! Manages the overall application state including servers, channels,
//! private messages, tabs, and user interface state.

use rustirc_core::connection::ConnectionState as CoreConnectionState;
use std::collections::{HashMap, VecDeque};
use std::time::SystemTime;

/// Application-wide state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Connected servers
    pub servers: HashMap<String, ServerInfo>,
    /// Open tabs (channels and private messages)
    pub tabs: HashMap<String, Tab>,
    /// Currently active tab
    pub current_tab_id: Option<String>,
    /// Tab order for navigation
    pub tab_order: Vec<String>,
    /// Global settings
    pub settings: AppSettings,
    /// UI state
    pub ui_state: UiState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            tabs: HashMap::new(),
            current_tab_id: None,
            tab_order: Vec::new(),
            settings: AppSettings::default(),
            ui_state: UiState::default(),
        }
    }

    /// Get the current tab
    pub fn current_tab(&self) -> Option<&Tab> {
        if let Some(tab_id) = &self.current_tab_id {
            self.tabs.get(tab_id)
        } else {
            None
        }
    }

    /// Get application settings
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Add a new server
    pub fn add_server(&mut self, server_id: String, name: String) {
        let server_info = ServerInfo::new(name);
        self.servers.insert(server_id.clone(), server_info);
        
        // Create server tab
        let tab = Tab::server(server_id.clone());
        let tab_id = format!("server:{}", server_id);
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());
        
        // Set as current tab if it's the first one
        if self.current_tab_id.is_none() {
            self.current_tab_id = Some(tab_id);
        }
    }

    /// Add a channel tab
    pub fn add_channel_tab(&mut self, server_id: String, channel: String) {
        let tab = Tab::channel(server_id.clone(), channel.clone());
        let tab_id = format!("{}:{}", server_id, channel);
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());
        
        // Set as current tab
        self.current_tab_id = Some(tab_id);
        
        // Add channel to server if server exists
        if let Some(server) = self.servers.get_mut(&server_id) {
            server.channels.insert(channel.clone(), ChannelInfo::new(channel));
        }
    }

    /// Add a private message tab
    pub fn add_private_tab(&mut self, server_id: String, nick: String) {
        let tab = Tab::private(server_id.clone(), nick.clone());
        let tab_id = format!("{}:@{}", server_id, nick);
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());
        
        // Set as current tab
        self.current_tab_id = Some(tab_id);
    }

    /// Remove a tab
    pub fn remove_tab(&mut self, tab_id: &str) {
        self.tabs.remove(tab_id);
        self.tab_order.retain(|id| id != tab_id);
    }

    /// Get current tab
    pub fn get_current_tab(&self) -> Option<&Tab> {
        if let Some(tab_id) = &self.current_tab_id {
            self.tabs.get(tab_id)
        } else {
            None
        }
    }

    /// Set current tab
    pub fn set_current_tab(&mut self, tab_id: String) {
        if self.tabs.contains_key(&tab_id) {
            self.current_tab_id = Some(tab_id);
        }
    }

    /// Switch to a specific tab
    pub fn switch_to_tab(&mut self, tab_id: &str) {
        if self.tabs.contains_key(tab_id) {
            self.current_tab_id = Some(tab_id.to_string());
        }
    }

    /// Get mutable reference to current tab
    pub fn current_tab_mut(&mut self) -> Option<&mut Tab> {
        if let Some(tab_id) = &self.current_tab_id.clone() {
            self.tabs.get_mut(tab_id)
        } else {
            None
        }
    }

    /// Close a tab
    pub fn close_tab(&mut self, tab_id: &str) {
        self.remove_tab(tab_id);
    }

    /// Get mutable reference to settings
    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    /// Get mutable reference to UI state
    pub fn ui_state_mut(&mut self) -> &mut UiState {
        &mut self.ui_state
    }
}

/// Tab information
#[derive(Debug, Clone)]
pub struct Tab {
    pub name: String,
    pub tab_type: TabType,
    pub server_id: Option<String>,
    pub messages: VecDeque<DisplayMessage>,
    pub activity: ActivityLevel,
    pub last_read_time: Option<SystemTime>,
    pub users: std::collections::HashMap<String, UserInfo>,
}

impl Tab {
    pub fn server(server_id: String) -> Self {
        Self {
            name: server_id.clone(),
            tab_type: TabType::Server,
            server_id: Some(server_id),
            messages: VecDeque::new(),
            activity: ActivityLevel::None,
            last_read_time: None,
            users: HashMap::new(),
        }
    }

    pub fn channel(server_id: String, channel: String) -> Self {
        Self {
            name: channel,
            tab_type: TabType::Channel,
            server_id: Some(server_id),
            messages: VecDeque::new(),
            activity: ActivityLevel::None,
            last_read_time: None,
            users: HashMap::new(),
        }
    }

    pub fn private(server_id: String, nick: String) -> Self {
        Self {
            name: nick,
            tab_type: TabType::Private,
            server_id: Some(server_id),
            messages: VecDeque::new(),
            activity: ActivityLevel::None,
            last_read_time: None,
            users: HashMap::new(),
        }
    }
}

/// Tab types
#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
    Server,
    Channel,
    Private,
}

/// Activity level indicators
#[derive(Debug, Clone, PartialEq)]
pub enum ActivityLevel {
    None,
    Activity,
    Highlight,
    Mention,
}

/// Server information
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub connection_state: CoreConnectionState,
    pub nickname: String,
    pub channels: HashMap<String, ChannelInfo>,
    pub users: HashMap<String, UserInfo>,
}

impl ServerInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            connection_state: CoreConnectionState::Disconnected,
            nickname: String::new(),
            channels: HashMap::new(),
            users: HashMap::new(),
        }
    }
}

/// Channel information
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub name: String,
    pub topic: Option<String>,
    pub modes: Vec<String>,
    pub user_count: usize,
    pub users: Vec<String>,
}

impl ChannelInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            topic: None,
            modes: Vec::new(),
            user_count: 0,
            users: Vec::new(),
        }
    }
}

/// User information
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub nickname: String,
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub realname: Option<String>,
    pub is_away: bool,
    pub away_message: Option<String>,
    pub modes: Vec<char>,
    pub away: bool,
}

impl UserInfo {
    pub fn new(nickname: String) -> Self {
        Self {
            nickname,
            username: None,
            hostname: None,
            realname: None,
            is_away: false,
            away_message: None,
            modes: Vec::new(),
            away: false,
        }
    }

    /// Check if user has a specific mode
    pub fn has_mode(&self, mode: char) -> bool {
        self.modes.contains(&mode)
    }
}

/// Application settings
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub theme: String,
    pub font_size: f32,
    pub show_timestamps: bool,
    pub show_join_part: bool,
    pub highlight_words: Vec<String>,
    pub notification_sound: bool,
    pub auto_reconnect: bool,
    pub nick_colors: bool,
    pub timestamp_format: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            font_size: 13.0,
            show_timestamps: true,
            show_join_part: true,
            highlight_words: Vec::new(),
            notification_sound: true,
            auto_reconnect: true,
            nick_colors: true,
            timestamp_format: "%H:%M:%S".to_string(),
        }
    }
}

/// UI state information
#[derive(Debug, Clone)]
pub struct UiState {
    pub window_maximized: bool,
    pub window_width: f32,
    pub window_height: f32,
    pub server_tree_width: f32,
    pub user_list_width: f32,
    pub show_server_tree: bool,
    pub show_user_list: bool,
    pub show_sidebar: bool,
    pub show_userlist: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            window_maximized: false,
            window_width: 1200.0,
            window_height: 800.0,
            server_tree_width: 200.0,
            user_list_width: 150.0,
            show_server_tree: true,
            show_user_list: true,
            show_sidebar: true,
            show_userlist: true,
        }
    }
}

/// Display message for GUI rendering
#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub content: String,
    pub sender: Option<String>,
    pub timestamp: SystemTime,
    pub message_type: MessageType,
    pub is_highlight: bool,
    pub is_own_message: bool,
}

/// Message types for display
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Message,
    Notice,
    Action,
    Join,
    Part,
    Quit,
    Nick,
    Topic,
    Mode,
    System,
}

/// Formatted text for IRC messages
#[derive(Debug, Clone)]
pub struct FormattedText {
    pub spans: Vec<FormattedSpan>,
}

/// Formatted text span for IRC message formatting
#[derive(Debug, Clone)]
pub struct FormattedSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub monospace: bool,
    pub color: Option<(u8, u8, u8)>,
    pub background_color: Option<(u8, u8, u8)>,
}

impl FormattedSpan {
    pub fn new(text: String) -> Self {
        Self {
            text,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            monospace: false,
            color: None,
            background_color: None,
        }
    }
}