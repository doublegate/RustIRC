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

/// Main TUI state
#[derive(Debug, Clone)]
pub struct TuiState {
    /// Connected servers
    pub servers: HashMap<String, ServerState>,
    
    /// Currently active server
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
    
    /// Whether we're showing the help screen
    pub show_help: bool,
    
    /// Selected channel in channel list
    pub selected_channel_index: usize,
    
    /// Selected user in user list
    pub selected_user_index: usize,
    
    /// Application start time for relative timestamps
    pub start_time: SystemTime,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            current_server: None,
            focus: FocusArea::Input,
            input_buffer: String::new(),
            input_cursor: 0,
            command_history: VecDeque::new(),
            history_position: 0,
            show_help: false,
            selected_channel_index: 0,
            selected_user_index: 0,
            start_time: SystemTime::now(),
        }
    }

    /// Add a server
    pub fn add_server(&mut self, server_name: String) {
        let server = ServerState::new(server_name.clone());
        self.servers.insert(server_name.clone(), server);
        
        // Switch to this server if it's the first one
        if self.current_server.is_none() {
            self.current_server = Some(server_name);
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
    pub fn add_message(&mut self, server_name: String, channel_name: String, nick: String, content: String) {
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
            (&self.current_server, self.current_channel()) {
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
        let channels: Vec<String> = self.current_server_channels().into_iter().cloned().collect();
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
        let channels: Vec<String> = self.current_server_channels().into_iter().cloned().collect();
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
        if !command.trim().is_empty() {
            if self.command_history.back() != Some(&command) {
                self.command_history.push_back(command.clone());
                
                // Limit history size
                if self.command_history.len() > MAX_COMMAND_HISTORY {
                    self.command_history.pop_front();
                }
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
        self.show_help = !self.show_help;
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
        // This could be used for relative time formatting
        // or other time-based animations
    }

    /// Get total unread message count
    pub fn total_unread_count(&self) -> usize {
        self.servers.values()
            .flat_map(|server| server.channels.values())
            .map(|channel| channel.unread_count)
            .sum()
    }

    /// Check if there are any highlights
    pub fn has_highlights(&self) -> bool {
        self.servers.values()
            .flat_map(|server| server.channels.values())
            .any(|channel| channel.has_highlight)
    }
}

impl Default for TuiState {
    fn default() -> Self {
        Self::new()
    }
}
