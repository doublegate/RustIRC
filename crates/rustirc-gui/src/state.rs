//! Application state management for RustIRC GUI
//!
//! Manages the overall application state including servers, channels,
//! private messages, tabs, and user interface state.

use rustirc_core::connection::ConnectionState as CoreConnectionState;
use rustirc_protocol::Message as IrcMessage;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime};

/// Application-wide state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Connected servers
    servers: HashMap<String, ServerState>,
    /// Open tabs (channels and private messages)
    tabs: HashMap<String, Tab>,
    /// Currently active tab
    current_tab_id: Option<String>,
    /// Tab order for navigation
    tab_order: Vec<String>,
    /// Global settings
    settings: AppSettings,
    /// UI state
    ui_state: UiState,
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

    /// Add a new server
    pub fn add_server(&mut self, server_id: &str) {
        let server_state = ServerState::new(server_id.to_string());
        self.servers.insert(server_id.to_string(), server_state);
        
        // Create server tab
        let tab = Tab::server(server_id.to_string());
        let tab_id = format!("server:{}", server_id);
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());
        
        // Switch to server tab if no current tab
        if self.current_tab_id.is_none() {
            self.current_tab_id = Some(tab_id);
        }
    }

    /// Remove a server and all its associated tabs
    pub fn remove_server(&mut self, server_id: &str) {
        self.servers.remove(server_id);
        
        // Remove all tabs for this server
        let tabs_to_remove: Vec<String> = self.tabs
            .iter()
            .filter(|(_, tab)| tab.server_id.as_deref() == Some(server_id))
            .map(|(id, _)| id.clone())
            .collect();
        
        for tab_id in tabs_to_remove {
            self.close_tab(&tab_id);
        }
    }

    /// Update connection state for a server
    pub fn update_connection_state(&mut self, server_id: &str, state: ConnectionState) {
        if let Some(server) = self.servers.get_mut(server_id) {
            server.connection_state = state;
        }
    }

    /// Add a channel tab
    pub fn add_channel_tab(&mut self, server_id: &str, channel: &str) {
        let tab_id = format!("{}:{}", server_id, channel);
        
        if !self.tabs.contains_key(&tab_id) {
            let tab = Tab::channel(server_id.to_string(), channel.to_string());
            self.tabs.insert(tab_id.clone(), tab);
            self.tab_order.push(tab_id.clone());
            
            // Add to server's channel list
            if let Some(server) = self.servers.get_mut(server_id) {
                server.channels.insert(channel.to_string(), ChannelState::new(channel.to_string()));
            }
        }
    }

    /// Add a private message tab
    pub fn add_private_tab(&mut self, server_id: &str, nick: &str) {
        let tab_id = format!("{}:query:{}", server_id, nick);
        
        if !self.tabs.contains_key(&tab_id) {
            let tab = Tab::private_message(server_id.to_string(), nick.to_string());
            self.tabs.insert(tab_id.clone(), tab);
            self.tab_order.push(tab_id.clone());
        }
    }

    /// Close a tab
    pub fn close_tab(&mut self, tab_id: &str) {
        if let Some(tab) = self.tabs.remove(tab_id) {
            // Remove from tab order
            self.tab_order.retain(|id| id != tab_id);
            
            // If this was the current tab, switch to another
            if self.current_tab_id.as_deref() == Some(tab_id) {
                self.current_tab_id = self.tab_order.last().cloned();
            }
            
            // Remove from server's channel list if it's a channel
            if let (Some(server_id), TabType::Channel { channel }) = (tab.server_id, tab.tab_type) {
                if let Some(server) = self.servers.get_mut(&server_id) {
                    server.channels.remove(&channel);
                }
            }
        }
    }

    /// Switch to a specific tab
    pub fn switch_to_tab(&mut self, tab_id: &str) {
        if self.tabs.contains_key(tab_id) {
            self.current_tab_id = Some(tab_id.to_string());
        }
    }

    /// Get the current tab
    pub fn current_tab(&self) -> Option<&Tab> {
        self.current_tab_id.as_ref().and_then(|id| self.tabs.get(id))
    }

    /// Get a mutable reference to the current tab
    pub fn current_tab_mut(&mut self) -> Option<&mut Tab> {
        self.current_tab_id.clone().and_then(|id| self.tabs.get_mut(&id))
    }

    /// Check if a tab exists
    pub fn has_tab(&self, tab_id: &str) -> bool {
        self.tabs.contains_key(tab_id)
    }

    /// Add a message to a specific tab
    pub fn add_message_to_tab(&mut self, tab_id: &str, message: IrcMessage) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            let display_message = DisplayMessage::from_irc_message(message);
            tab.messages.push_back(display_message);
            
            // Limit message history
            if tab.messages.len() > self.settings.max_messages_per_tab {
                tab.messages.pop_front();
            }
            
            // Mark as having activity if not current tab
            if self.current_tab_id.as_deref() != Some(tab_id) {
                tab.has_activity = true;
            }
        }
    }

    /// Add user to channel
    pub fn add_user_to_channel(&mut self, tab_id: &str, nick: &str) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            if let TabType::Channel { .. } = tab.tab_type {
                let user = UserState::new(nick.to_string());
                tab.users.insert(nick.to_string(), user);
            }
        }
    }

    /// Remove user from channel
    pub fn remove_user_from_channel(&mut self, tab_id: &str, nick: &str) {
        if let Some(tab) = self.tabs.get_mut(tab_id) {
            tab.users.remove(nick);
        }
    }

    /// Remove user from all channels on a server
    pub fn remove_user_from_server(&mut self, server_id: &str, nick: &str) {
        for tab in self.tabs.values_mut() {
            if tab.server_id.as_deref() == Some(server_id) {
                tab.users.remove(nick);
            }
        }
    }

    /// Get all tabs
    pub fn tabs(&self) -> &HashMap<String, Tab> {
        &self.tabs
    }

    /// Get tab order
    pub fn tab_order(&self) -> &[String] {
        &self.tab_order
    }

    /// Get servers
    pub fn servers(&self) -> &HashMap<String, ServerState> {
        &self.servers
    }

    /// Get settings
    pub fn settings(&self) -> &AppSettings {
        &self.settings
    }

    /// Get mutable settings
    pub fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }

    /// Get UI state
    pub fn ui_state(&self) -> &UiState {
        &self.ui_state
    }

    /// Get mutable UI state
    pub fn ui_state_mut(&mut self) -> &mut UiState {
        &mut self.ui_state
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for a single server connection
#[derive(Debug, Clone)]
pub struct ServerState {
    pub server_id: String,
    pub connection_state: ConnectionState,
    pub nickname: String,
    pub channels: HashMap<String, ChannelState>,
    pub modes: Vec<String>,
    pub away_message: Option<String>,
}

impl ServerState {
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            connection_state: ConnectionState::Disconnected,
            nickname: String::new(),
            channels: HashMap::new(),
            modes: Vec::new(),
            away_message: None,
        }
    }
}

/// Connection state (GUI representation)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Authenticating,
    Registered,
    Reconnecting,
    Failed(String),
}

impl From<CoreConnectionState> for ConnectionState {
    fn from(state: CoreConnectionState) -> Self {
        match state {
            CoreConnectionState::Disconnected => ConnectionState::Disconnected,
            CoreConnectionState::Connecting => ConnectionState::Connecting,
            CoreConnectionState::Connected => ConnectionState::Connected,
            CoreConnectionState::Authenticating => ConnectionState::Authenticating,
            CoreConnectionState::Registered => ConnectionState::Registered,
            CoreConnectionState::Reconnecting => ConnectionState::Reconnecting,
            CoreConnectionState::Failed(msg) => ConnectionState::Failed(msg),
        }
    }
}

/// State for a single channel
#[derive(Debug, Clone)]
pub struct ChannelState {
    pub name: String,
    pub topic: Option<String>,
    pub modes: Vec<String>,
    pub user_count: usize,
    pub joined: bool,
}

impl ChannelState {
    pub fn new(name: String) -> Self {
        Self {
            name,
            topic: None,
            modes: Vec::new(),
            user_count: 0,
            joined: false,
        }
    }
}

/// State for a user
#[derive(Debug, Clone)]
pub struct UserState {
    pub nickname: String,
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub realname: Option<String>,
    pub modes: Vec<String>,
    pub away: bool,
    pub away_message: Option<String>,
}

impl UserState {
    pub fn new(nickname: String) -> Self {
        Self {
            nickname,
            username: None,
            hostname: None,
            realname: None,
            modes: Vec::new(),
            away: false,
            away_message: None,
        }
    }

    /// Check if user has a specific mode
    pub fn has_mode(&self, mode: char) -> bool {
        self.modes.iter().any(|m| m.contains(mode))
    }

    /// Get user's highest privilege level
    pub fn privilege_level(&self) -> u8 {
        if self.has_mode('~') { 5 } // Owner
        else if self.has_mode('&') { 4 } // Admin
        else if self.has_mode('@') { 3 } // Op
        else if self.has_mode('%') { 2 } // Half-op
        else if self.has_mode('+') { 1 } // Voice
        else { 0 } // Normal
    }
}

/// Types of tabs
#[derive(Debug, Clone, PartialEq)]
pub enum TabType {
    Server,
    Channel { channel: String },
    PrivateMessage { nick: String },
}

/// A single tab (channel, private message, or server)
#[derive(Debug, Clone)]
pub struct Tab {
    pub tab_type: TabType,
    pub server_id: Option<String>,
    pub messages: VecDeque<DisplayMessage>,
    pub users: HashMap<String, UserState>,
    pub has_activity: bool,
    pub has_highlight: bool,
    pub last_read: SystemTime,
}

impl Tab {
    pub fn server(server_id: String) -> Self {
        Self {
            tab_type: TabType::Server,
            server_id: Some(server_id),
            messages: VecDeque::new(),
            users: HashMap::new(),
            has_activity: false,
            has_highlight: false,
            last_read: SystemTime::now(),
        }
    }

    pub fn channel(server_id: String, channel: String) -> Self {
        Self {
            tab_type: TabType::Channel { channel },
            server_id: Some(server_id),
            messages: VecDeque::new(),
            users: HashMap::new(),
            has_activity: false,
            has_highlight: false,
            last_read: SystemTime::now(),
        }
    }

    pub fn private_message(server_id: String, nick: String) -> Self {
        Self {
            tab_type: TabType::PrivateMessage { nick },
            server_id: Some(server_id),
            messages: VecDeque::new(),
            users: HashMap::new(),
            has_activity: false,
            has_highlight: false,
            last_read: SystemTime::now(),
        }
    }

    /// Get the tab title for display
    pub fn title(&self) -> String {
        match &self.tab_type {
            TabType::Server => {
                if let Some(server_id) = &self.server_id {
                    server_id.clone()
                } else {
                    "Server".to_string()
                }
            }
            TabType::Channel { channel } => channel.clone(),
            TabType::PrivateMessage { nick } => nick.clone(),
        }
    }

    /// Get the target for sending messages
    pub fn target(&self) -> String {
        match &self.tab_type {
            TabType::Server => {
                self.server_id.as_ref().unwrap_or(&"server".to_string()).clone()
            }
            TabType::Channel { channel } => channel.clone(),
            TabType::PrivateMessage { nick } => nick.clone(),
        }
    }

    /// Check if this is a channel tab
    pub fn is_channel(&self) -> bool {
        matches!(self.tab_type, TabType::Channel { .. })
    }

    /// Check if this is a private message tab
    pub fn is_private_message(&self) -> bool {
        matches!(self.tab_type, TabType::PrivateMessage { .. })
    }

    /// Check if this is a server tab
    pub fn is_server(&self) -> bool {
        matches!(self.tab_type, TabType::Server)
    }

    /// Mark as read (clear activity indicators)
    pub fn mark_as_read(&mut self) {
        self.has_activity = false;
        self.has_highlight = false;
        self.last_read = SystemTime::now();
    }
}

/// Display message for the GUI
#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub timestamp: SystemTime,
    pub message_type: MessageType,
    pub sender: Option<String>,
    pub content: String,
    pub formatted_content: Vec<FormattedText>,
    pub is_highlight: bool,
    pub is_own_message: bool,
}

impl DisplayMessage {
    pub fn from_irc_message(message: IrcMessage) -> Self {
        let timestamp = SystemTime::now();
        let sender = message.prefix.as_ref().map(|p| {
            match p {
                rustirc_protocol::Prefix::User { nick, .. } => nick.clone(),
                rustirc_protocol::Prefix::Server(server) => server.clone(),
            }
        });

        let (message_type, content, is_highlight) = match message.command.as_str() {
            "PRIVMSG" => {
                let content = message.params.get(1).cloned().unwrap_or_default();
                let is_action = content.starts_with("\x01ACTION ") && content.ends_with('\x01');
                
                if is_action {
                    let action_content = content.trim_start_matches("\x01ACTION ")
                        .trim_end_matches('\x01');
                    (MessageType::Action, action_content.to_string(), false)
                } else {
                    (MessageType::Message, content, false) // Highlight detection would be done elsewhere
                }
            }
            "NOTICE" => {
                let content = message.params.get(1).cloned().unwrap_or_default();
                (MessageType::Notice, content, false)
            }
            "JOIN" => {
                let channel = message.params.get(0).cloned().unwrap_or_default();
                let content = format!("joined {}", channel);
                (MessageType::Join, content, false)
            }
            "PART" => {
                let channel = message.params.get(0).cloned().unwrap_or_default();
                let reason = message.params.get(1)
                    .map(|r| format!(" ({})", r))
                    .unwrap_or_default();
                let content = format!("left {}{}", channel, reason);
                (MessageType::Part, content, false)
            }
            "QUIT" => {
                let reason = message.params.get(0)
                    .map(|r| format!(" ({})", r))
                    .unwrap_or_default();
                let content = format!("quit{}", reason);
                (MessageType::Quit, content, false)
            }
            "NICK" => {
                let new_nick = message.params.get(0).cloned().unwrap_or_default();
                let content = format!("is now known as {}", new_nick);
                (MessageType::Nick, content, false)
            }
            "TOPIC" => {
                let channel = message.params.get(0).cloned().unwrap_or_default();
                let topic = message.params.get(1).cloned().unwrap_or_default();
                let content = format!("changed topic of {} to: {}", channel, topic);
                (MessageType::Topic, content, false)
            }
            "MODE" => {
                let target = message.params.get(0).cloned().unwrap_or_default();
                let modes = message.params.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
                let content = format!("set mode {} {}", modes, target);
                (MessageType::Mode, content, false)
            }
            _ => {
                // Numeric replies or other messages
                let content = message.params.join(" ");
                (MessageType::System, content, false)
            }
        };

        // Parse IRC formatting
        let formatted_content = parse_irc_formatting(&content);

        Self {
            timestamp,
            message_type,
            sender,
            content,
            formatted_content,
            is_highlight,
            is_own_message: false, // Would be set based on own nick
        }
    }

    /// Create a system message
    pub fn system(content: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            message_type: MessageType::System,
            sender: None,
            content: content.clone(),
            formatted_content: vec![FormattedText::new(content)],
            is_highlight: false,
            is_own_message: false,
        }
    }
}

/// Types of messages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Message,
    Action,
    Notice,
    Join,
    Part,
    Quit,
    Nick,
    Topic,
    Mode,
    System,
}

/// Formatted text segment
#[derive(Debug, Clone)]
pub struct FormattedText {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub monospace: bool,
    pub reverse: bool,
    pub color: Option<u8>,
    pub background_color: Option<u8>,
}

impl FormattedText {
    pub fn new(text: String) -> Self {
        Self {
            text,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            monospace: false,
            reverse: false,
            color: None,
            background_color: None,
        }
    }
}

/// Parse IRC formatting codes
fn parse_irc_formatting(text: &str) -> Vec<FormattedText> {
    let mut result = Vec::new();
    let mut current_text = String::new();
    let mut current_format = FormattedText::new(String::new());
    
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        match ch {
            '\x02' => { // Bold
                if !current_text.is_empty() {
                    current_format.text = current_text.clone();
                    result.push(current_format.clone());
                    current_text.clear();
                }
                current_format.bold = !current_format.bold;
            }
            '\x1D' => { // Italic
                if !current_text.is_empty() {
                    current_format.text = current_text.clone();
                    result.push(current_format.clone());
                    current_text.clear();
                }
                current_format.italic = !current_format.italic;
            }
            '\x1F' => { // Underline
                if !current_text.is_empty() {
                    current_format.text = current_text.clone();
                    result.push(current_format.clone());
                    current_text.clear();
                }
                current_format.underline = !current_format.underline;
            }
            '\x1E' => { // Strikethrough
                if !current_text.is_empty() {
                    current_format.text = current_text.clone();
                    result.push(current_format.clone());
                    current_text.clear();
                }
                current_format.strikethrough = !current_format.strikethrough;
            }
            '\x11' => { // Monospace
                if !current_text.is_empty() {
                    current_format.text = current_text.clone();
                    result.push(current_format.clone());
                    current_text.clear();
                }
                current_format.monospace = !current_format.monospace;
            }
            '\x16' => { // Reverse
                if !current_text.is_empty() {
                    current_format.text = current_text.clone();
                    result.push(current_format.clone());
                    current_text.clear();
                }
                current_format.reverse = !current_format.reverse;
            }
            '\x03' => { // Color
                if !current_text.is_empty() {
                    current_format.text = current_text.clone();
                    result.push(current_format.clone());
                    current_text.clear();
                }
                
                // Parse color codes
                let mut color_str = String::new();
                while let Some(next_ch) = chars.clone().next() {
                    if next_ch.is_ascii_digit() {
                        color_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                if !color_str.is_empty() {
                    if let Ok(color) = color_str.parse::<u8>() {
                        current_format.color = Some(color % 16);
                    }
                }
                
                // Check for background color
                if chars.clone().next() == Some(',') {
                    chars.next(); // consume comma
                    let mut bg_color_str = String::new();
                    while let Some(next_ch) = chars.clone().next() {
                        if next_ch.is_ascii_digit() {
                            bg_color_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    
                    if !bg_color_str.is_empty() {
                        if let Ok(bg_color) = bg_color_str.parse::<u8>() {
                            current_format.background_color = Some(bg_color % 16);
                        }
                    }
                }
            }
            '\x0F' => { // Reset
                if !current_text.is_empty() {
                    current_format.text = current_text.clone();
                    result.push(current_format.clone());
                    current_text.clear();
                }
                current_format = FormattedText::new(String::new());
            }
            _ => {
                current_text.push(ch);
            }
        }
    }
    
    // Add remaining text
    if !current_text.is_empty() {
        current_format.text = current_text;
        result.push(current_format);
    }
    
    // If no formatting was found, return simple text
    if result.is_empty() {
        result.push(FormattedText::new(text.to_string()));
    }
    
    result
}

/// Application settings
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub max_messages_per_tab: usize,
    pub highlight_keywords: Vec<String>,
    pub notification_sound: bool,
    pub notification_popup: bool,
    pub auto_away_time: Option<Duration>,
    pub font_size: f32,
    pub show_timestamps: bool,
    pub timestamp_format: String,
    pub nick_colors: bool,
    pub show_joins_parts: bool,
    pub compact_mode: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_messages_per_tab: 1000,
            highlight_keywords: Vec::new(),
            notification_sound: true,
            notification_popup: true,
            auto_away_time: Some(Duration::from_secs(600)), // 10 minutes
            font_size: 13.0,
            show_timestamps: true,
            timestamp_format: "%H:%M:%S".to_string(),
            nick_colors: true,
            show_joins_parts: true,
            compact_mode: false,
        }
    }
}

/// UI-specific state
#[derive(Debug, Clone)]
pub struct UiState {
    pub sidebar_width: f32,
    pub userlist_width: f32,
    pub show_sidebar: bool,
    pub show_userlist: bool,
    pub input_history: VecDeque<String>,
    pub input_history_index: Option<usize>,
    pub current_input: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            sidebar_width: 200.0,
            userlist_width: 150.0,
            show_sidebar: true,
            show_userlist: true,
            input_history: VecDeque::new(),
            input_history_index: None,
            current_input: String::new(),
        }
    }
}

impl UiState {
    /// Add command to input history
    pub fn add_to_input_history(&mut self, input: String) {
        if !input.trim().is_empty() && self.input_history.back() != Some(&input) {
            self.input_history.push_back(input);
            
            // Limit history size
            if self.input_history.len() > 100 {
                self.input_history.pop_front();
            }
        }
        
        self.input_history_index = None;
    }
    
    /// Navigate input history (up = true, down = false)
    pub fn navigate_input_history(&mut self, up: bool) -> Option<String> {
        if self.input_history.is_empty() {
            return None;
        }
        
        if up {
            match self.input_history_index {
                None => {
                    self.input_history_index = Some(self.input_history.len() - 1);
                    self.input_history.back().cloned()
                }
                Some(index) if index > 0 => {
                    self.input_history_index = Some(index - 1);
                    self.input_history.get(index - 1).cloned()
                }
                Some(_) => None, // Already at the beginning
            }
        } else {
            match self.input_history_index {
                Some(index) if index < self.input_history.len() - 1 => {
                    self.input_history_index = Some(index + 1);
                    self.input_history.get(index + 1).cloned()
                }
                Some(_) => {
                    self.input_history_index = None;
                    Some(String::new()) // Return to current input
                }
                None => None, // Not in history mode
            }
        }
    }
}

// Re-export for compatibility
pub use AppState as GuiState;