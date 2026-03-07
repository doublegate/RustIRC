//! Application state management for RustIRC GUI
//!
//! Manages the overall application state including servers, channels,
//! private messages, tabs, and user interface state.
//! Designed for use with Dioxus Signal-based reactivity.

use rustirc_core::connection::ConnectionState as CoreConnectionState;
use std::collections::{HashMap, VecDeque};
use std::time::SystemTime;

/// Application-wide state (wrapped in Signal<AppState> at the provider level)
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

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
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
        self.current_tab_id
            .as_ref()
            .and_then(|id| self.tabs.get(id))
    }

    /// Get mutable reference to current tab
    pub fn current_tab_mut(&mut self) -> Option<&mut Tab> {
        let tab_id = self.current_tab_id.clone();
        tab_id.and_then(move |id| self.tabs.get_mut(&id))
    }

    /// Get application settings
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Get mutable reference to settings
    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    /// Get mutable reference to UI state
    pub fn ui_state_mut(&mut self) -> &mut UiState {
        &mut self.ui_state
    }

    /// Add a new server
    pub fn add_server(&mut self, server_id: String, name: String) {
        let server_info = ServerInfo::new(name);
        self.servers.insert(server_id.clone(), server_info);

        let tab = Tab::server(server_id.clone());
        let tab_id = format!("server:{server_id}");
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());

        if self.current_tab_id.is_none() {
            self.current_tab_id = Some(tab_id);
        }
    }

    /// Add a channel tab
    pub fn add_channel_tab(&mut self, server_id: String, channel: String) {
        let tab = Tab::channel(server_id.clone(), channel.clone());
        let tab_id = format!("{server_id}:{channel}");
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());
        self.current_tab_id = Some(tab_id);

        if let Some(server) = self.servers.get_mut(&server_id) {
            server
                .channels
                .insert(channel.clone(), ChannelInfo::new(channel));
        }
    }

    /// Remove a tab
    pub fn remove_tab(&mut self, tab_id: &str) {
        self.tabs.remove(tab_id);
        self.tab_order.retain(|id| id != tab_id);
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

    /// Close a tab
    pub fn close_tab(&mut self, tab_id: &str) {
        self.remove_tab(tab_id);
    }

    /// Add a private message tab
    pub fn add_private_tab(&mut self, server_id: &str, nick: String) {
        let tab = Tab::private_message(server_id.to_string(), nick.clone());
        let tab_id = format!("{server_id}:pm:{nick}");
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());
        self.current_tab_id = Some(tab_id);
    }

    /// Add a message to a tab
    pub fn add_message(&mut self, server_id: &str, target: &str, message: &str, sender: &str) {
        let tab_id = if target.starts_with('#') || target.starts_with('&') {
            format!("{server_id}:{target}")
        } else if target == server_id {
            format!("server:{server_id}")
        } else {
            format!(
                "{}:pm:{}",
                server_id,
                if sender == "self" { target } else { sender }
            )
        };

        let message_id = self.next_message_id();

        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            let display_msg = DisplayMessage {
                id: message_id,
                timestamp: SystemTime::now(),
                sender: sender.to_string(),
                content: message.to_string(),
                message_type: MessageType::Message,
                is_highlight: false,
                is_own_message: sender == "self",
            };

            tab.messages.push_back(display_msg);
            tab.has_activity = true;

            if tab.messages.len() > 1000 {
                tab.messages.pop_front();
            }
        }
    }

    /// Remove a server and all associated tabs
    pub fn remove_server(&mut self, server_id: &str) {
        self.servers.remove(server_id);

        let tabs_to_remove: Vec<String> = self
            .tabs
            .iter()
            .filter(|(_, tab)| tab.server_id.as_ref() == Some(&server_id.to_string()))
            .map(|(id, _)| id.clone())
            .collect();

        for tab_id in tabs_to_remove {
            self.tabs.remove(&tab_id);
            self.tab_order.retain(|id| id != &tab_id);
        }

        if let Some(current_id) = &self.current_tab_id {
            if !self.tabs.contains_key(current_id) {
                self.current_tab_id = self.tab_order.first().cloned();
            }
        }
    }

    fn next_message_id(&mut self) -> usize {
        self.settings.last_message_id += 1;
        self.settings.last_message_id
    }

    /// Add a user to a channel
    pub fn add_user_to_channel(&mut self, server_id: &str, channel: &str, nick: &str) {
        if let Some(server) = self.servers.get_mut(server_id) {
            if let Some(channel_info) = server.channels.get_mut(channel) {
                if !channel_info.users.contains(&nick.to_string()) {
                    channel_info.users.push(nick.to_string());
                    channel_info.user_count = channel_info.users.len();
                }
            }
        }

        let tab_id = format!("{server_id}:{channel}");
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.users
                .insert(nick.to_string(), UserInfo::new(nick.to_string()));
        }
    }

    /// Remove a user from a channel
    pub fn remove_user_from_channel(&mut self, server_id: &str, channel: &str, nick: &str) {
        if let Some(server) = self.servers.get_mut(server_id) {
            if let Some(channel_info) = server.channels.get_mut(channel) {
                channel_info.users.retain(|user| user != nick);
                channel_info.user_count = channel_info.users.len();
            }
        }

        let tab_id = format!("{server_id}:{channel}");
        if let Some(tab) = self.tabs.get_mut(&tab_id) {
            tab.users.remove(nick);
        }
    }

    /// Remove a user from all channels (when user quits)
    pub fn remove_user_from_all_channels(&mut self, server_id: &str, nick: &str) {
        if let Some(server) = self.servers.get_mut(server_id) {
            for channel_info in server.channels.values_mut() {
                channel_info.users.retain(|user| user != nick);
                channel_info.user_count = channel_info.users.len();
            }
        }

        for (tab_id, tab) in self.tabs.iter_mut() {
            if tab_id.starts_with(&format!("{server_id}:")) && tab_id.contains('#') {
                tab.users.remove(nick);
            }
        }
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
    pub users: HashMap<String, UserInfo>,
    pub has_highlight: bool,
    pub has_activity: bool,
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
            has_highlight: false,
            has_activity: false,
        }
    }

    pub fn channel(server_id: String, channel: String) -> Self {
        Self {
            name: channel.clone(),
            tab_type: TabType::Channel { channel },
            server_id: Some(server_id),
            messages: VecDeque::new(),
            activity: ActivityLevel::None,
            last_read_time: None,
            users: HashMap::new(),
            has_highlight: false,
            has_activity: false,
        }
    }

    pub fn private_message(server_id: String, nick: String) -> Self {
        Self {
            name: nick.clone(),
            tab_type: TabType::PrivateMessage { nick },
            server_id: Some(server_id),
            messages: VecDeque::new(),
            activity: ActivityLevel::None,
            last_read_time: None,
            users: HashMap::new(),
            has_highlight: false,
            has_activity: false,
        }
    }

    pub fn mark_as_read(&mut self) {
        self.last_read_time = Some(SystemTime::now());
        self.activity = ActivityLevel::None;
        self.has_highlight = false;
        self.has_activity = false;
    }
}

/// Tab types
#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
    Server,
    Channel { channel: String },
    PrivateMessage { nick: String },
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
    pub modes: Vec<String>,
    pub last_ping: Option<SystemTime>,
}

impl ServerInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            connection_state: CoreConnectionState::Disconnected,
            nickname: String::new(),
            channels: HashMap::new(),
            users: HashMap::new(),
            modes: Vec::new(),
            last_ping: None,
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
    pub is_op: bool,
    pub is_voice: bool,
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
            is_op: false,
            is_voice: false,
        }
    }

    pub fn has_mode(&self, mode: char) -> bool {
        self.modes.contains(&mode)
    }

    pub fn privilege_level(&self) -> u8 {
        if self.has_mode('o') {
            4
        } else if self.has_mode('h') {
            3
        } else if self.has_mode('v') {
            2
        } else {
            1
        }
    }
}

/// Application settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
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
    pub last_message_id: usize,
    pub notification_popup: bool,
    pub compact_mode: bool,
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
            last_message_id: 0,
            notification_popup: true,
            compact_mode: false,
        }
    }
}

impl AppSettings {
    pub fn settings_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("rustirc")
            .join("settings.toml")
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load() -> Self {
        let path = Self::settings_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(settings) => return settings,
                    Err(e) => tracing::warn!("Failed to parse settings: {}", e),
                },
                Err(e) => tracing::warn!("Failed to read settings: {}", e),
            }
        }
        Self::default()
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
        }
    }
}

/// Display message for GUI rendering
#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub id: usize,
    pub content: String,
    pub sender: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert!(state.servers.is_empty());
        assert!(state.tabs.is_empty());
        assert!(state.current_tab_id.is_none());
        assert!(state.tab_order.is_empty());
    }

    #[test]
    fn test_add_server() {
        let mut state = AppState::new();
        state.add_server("irc.example.com:6697".to_string(), "Example".to_string());
        assert_eq!(state.servers.len(), 1);
        assert_eq!(state.tabs.len(), 1);
        assert!(state.current_tab_id.is_some());
    }

    #[test]
    fn test_add_channel_tab() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_channel_tab("server1".to_string(), "#test".to_string());
        assert_eq!(state.tabs.len(), 2); // server + channel
        assert_eq!(state.current_tab_id, Some("server1:#test".to_string()));
    }

    #[test]
    fn test_add_message() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_channel_tab("server1".to_string(), "#test".to_string());
        state.add_message("server1", "#test", "Hello world", "nick1");
        let tab = state.tabs.get("server1:#test").unwrap();
        assert_eq!(tab.messages.len(), 1);
        assert_eq!(tab.messages[0].content, "Hello world");
        assert_eq!(tab.messages[0].sender, "nick1");
    }

    #[test]
    fn test_remove_server() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_channel_tab("server1".to_string(), "#test".to_string());
        assert_eq!(state.tabs.len(), 2);
        state.remove_server("server1");
        assert!(state.servers.is_empty());
        assert!(state.tabs.is_empty());
    }

    #[test]
    fn test_switch_tab() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_channel_tab("server1".to_string(), "#a".to_string());
        state.add_channel_tab("server1".to_string(), "#b".to_string());
        state.switch_to_tab("server1:#a");
        assert_eq!(state.current_tab_id, Some("server1:#a".to_string()));
    }

    #[test]
    fn test_close_tab() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_channel_tab("server1".to_string(), "#test".to_string());
        assert_eq!(state.tabs.len(), 2);
        state.close_tab("server1:#test");
        assert_eq!(state.tabs.len(), 1);
    }

    #[test]
    fn test_add_user_to_channel() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_channel_tab("server1".to_string(), "#test".to_string());
        state.add_user_to_channel("server1", "#test", "user1");
        let server = state.servers.get("server1").unwrap();
        let channel = server.channels.get("#test").unwrap();
        assert_eq!(channel.users.len(), 1);
        assert!(channel.users.contains(&"user1".to_string()));
    }

    #[test]
    fn test_remove_user_from_channel() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_channel_tab("server1".to_string(), "#test".to_string());
        state.add_user_to_channel("server1", "#test", "user1");
        state.remove_user_from_channel("server1", "#test", "user1");
        let server = state.servers.get("server1").unwrap();
        let channel = server.channels.get("#test").unwrap();
        assert!(channel.users.is_empty());
    }

    #[test]
    fn test_private_message_tab() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_private_tab("server1", "friend".to_string());
        assert!(state.tabs.contains_key("server1:pm:friend"));
        assert_eq!(state.current_tab_id, Some("server1:pm:friend".to_string()));
    }

    #[test]
    fn test_message_history_limit() {
        let mut state = AppState::new();
        state.add_server("server1".to_string(), "Server 1".to_string());
        state.add_channel_tab("server1".to_string(), "#test".to_string());
        for i in 0..1010 {
            state.add_message("server1", "#test", &format!("msg {i}"), "user");
        }
        let tab = state.tabs.get("server1:#test").unwrap();
        assert!(tab.messages.len() <= 1000);
    }

    #[test]
    fn test_user_privilege_level() {
        let mut user = UserInfo::new("nick".to_string());
        assert_eq!(user.privilege_level(), 1);
        user.modes.push('v');
        assert_eq!(user.privilege_level(), 2);
        user.modes.push('o');
        assert_eq!(user.privilege_level(), 4);
    }

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert_eq!(settings.theme, "Dark");
        assert!(settings.show_timestamps);
        assert!(settings.auto_reconnect);
        assert_eq!(settings.font_size, 13.0);
    }

    #[test]
    fn test_tab_mark_as_read() {
        let mut tab = Tab::channel("server1".to_string(), "#test".to_string());
        tab.has_activity = true;
        tab.has_highlight = true;
        tab.activity = ActivityLevel::Highlight;
        tab.mark_as_read();
        assert!(!tab.has_activity);
        assert!(!tab.has_highlight);
        assert_eq!(tab.activity, ActivityLevel::None);
    }
}
