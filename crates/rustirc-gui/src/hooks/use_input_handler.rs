//! Hook for input handling and IRC command processing

use crate::context::{IrcState, MessageType};
use crate::hooks::use_irc_connection::use_irc_connection;
use dioxus::prelude::*;

/// Input handling hook for IRC commands and messages
#[allow(non_snake_case)]
pub fn use_input_handler() -> InputHandlerHook {
    let mut input_text = use_signal(|| String::new());
    let mut command_history = use_signal(|| Vec::<String>::new());
    let mut history_index = use_signal(|| 0usize);
    let mut suggestions = use_signal(|| Vec::<String>::new());
    let mut show_suggestions = use_signal(|| false);
    
    let irc_connection = use_irc_connection();
    
    InputHandlerHook {
        input_text,
        command_history,
        history_index,
        suggestions,
        show_suggestions,
        irc_connection,
    }
}

/// Input handler hook interface
pub struct InputHandlerHook {
    pub input_text: Signal<String>,
    pub command_history: Signal<Vec<String>>,
    pub history_index: Signal<usize>,
    pub suggestions: Signal<Vec<String>>,
    pub show_suggestions: Signal<bool>,
    pub irc_connection: crate::hooks::use_irc_connection::IrcConnectionHook,
}

impl InputHandlerHook {
    /// Get current input text
    pub fn get_text(&self) -> String {
        self.input_text.read().clone()
    }

    /// Set input text
    pub fn set_text(&self, text: String) {
        self.input_text.set(text);
        self.update_suggestions();
    }

    /// Clear input text
    pub fn clear(&self) {
        self.input_text.set(String::new());
        self.hide_suggestions();
    }

    /// Handle key press events
    pub fn handle_key_press(&self, key: &str, ctrl: bool, shift: bool, alt: bool) {
        match key {
            "Enter" => {
                if !shift {
                    self.submit();
                } else {
                    // Shift+Enter adds newline (for multiline input)
                    let current = self.get_text();
                    self.set_text(format!("{}\n", current));
                }
            }
            "ArrowUp" => {
                if ctrl {
                    self.scroll_message_area(-1);
                } else {
                    self.navigate_history_up();
                }
            }
            "ArrowDown" => {
                if ctrl {
                    self.scroll_message_area(1);
                } else {
                    self.navigate_history_down();
                }
            }
            "Tab" => {
                self.handle_tab_completion();
            }
            "Escape" => {
                self.hide_suggestions();
            }
            "Home" => {
                if ctrl {
                    self.scroll_to_top();
                }
            }
            "End" => {
                if ctrl {
                    self.scroll_to_bottom();
                }
            }
            "PageUp" => {
                self.scroll_message_area(-10);
            }
            "PageDown" => {
                self.scroll_message_area(10);
            }
            _ => {
                // Handle IRC formatting shortcuts
                if ctrl {
                    self.handle_formatting_shortcut(key);
                }
            }
        }
    }

    /// Submit current input
    pub fn submit(&self) {
        let text = self.get_text().trim().to_string();
        if text.is_empty() {
            return;
        }

        // Add to command history
        self.add_to_history(text.clone());

        // Process the input
        spawn({
            let hook = self.clone();
            let text = text.clone();
            async move {
                hook.process_input(text).await;
            }
        });

        // Clear input
        self.clear();
    }

    /// Process input (command or message)
    pub async fn process_input(&self, text: String) {
        if text.starts_with('/') {
            self.execute_command(text).await;
        } else {
            self.send_message(text).await;
        }
    }

    /// Execute IRC command
    async fn execute_command(&self, command: String) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "/join" | "/j" => {
                if parts.len() >= 2 {
                    let channel = parts[1].to_string();
                    if let Some(server_id) = self.irc_connection.get_current_server() {
                        if let Err(e) = self.irc_connection.join_channel(server_id, channel.clone()).await {
                            self.add_error_message(format!("Failed to join {}: {}", channel, e));
                        }
                    } else {
                        self.add_error_message("Not connected to any server");
                    }
                } else {
                    self.add_error_message("Usage: /join <channel>");
                }
            }
            "/part" | "/leave" => {
                if let Some(channel) = self.irc_connection.get_current_channel() {
                    // TODO: Implement part command
                    self.add_system_message(format!("Leaving {}", channel));
                } else {
                    self.add_error_message("Not in a channel");
                }
            }
            "/quit" => {
                let quit_message = if parts.len() > 1 {
                    parts[1..].join(" ")
                } else {
                    "RustIRC".to_string()
                };
                
                if let Some(server_id) = self.irc_connection.get_current_server() {
                    if let Err(e) = self.irc_connection.disconnect(server_id).await {
                        self.add_error_message(format!("Error disconnecting: {}", e));
                    }
                }
            }
            "/nick" => {
                if parts.len() >= 2 {
                    let new_nick = parts[1];
                    // TODO: Implement nick change
                    self.add_system_message(format!("Changing nickname to {}", new_nick));
                } else {
                    self.add_error_message("Usage: /nick <nickname>");
                }
            }
            "/msg" | "/query" => {
                if parts.len() >= 3 {
                    let target = parts[1].to_string();
                    let message = parts[2..].join(" ");
                    
                    if let Some(server_id) = self.irc_connection.get_current_server() {
                        if let Err(e) = self.irc_connection.send_message(server_id, target.clone(), message.clone()).await {
                            self.add_error_message(format!("Failed to send message to {}: {}", target, e));
                        }
                    }
                } else {
                    self.add_error_message("Usage: /msg <nick> <message>");
                }
            }
            "/me" => {
                if parts.len() >= 2 {
                    let action = parts[1..].join(" ");
                    self.send_action_message(action).await;
                } else {
                    self.add_error_message("Usage: /me <action>");
                }
            }
            "/clear" => {
                // TODO: Clear message history in current channel
                self.add_system_message("Message history cleared");
            }
            "/help" => {
                self.show_help();
            }
            _ => {
                self.add_error_message(format!("Unknown command: {}", cmd));
            }
        }
    }

    /// Send regular chat message
    async fn send_message(&self, text: String) {
        if let (Some(server_id), Some(channel)) = (
            self.irc_connection.get_current_server(),
            self.irc_connection.get_current_channel(),
        ) {
            if let Err(e) = self.irc_connection.send_message(server_id, channel, text).await {
                self.add_error_message(format!("Failed to send message: {}", e));
            }
        } else {
            self.add_error_message("Not connected to a channel. Use /join <channel> to join a channel.");
        }
    }

    /// Send action message (/me)
    async fn send_action_message(&self, action: String) {
        if let (Some(server_id), Some(channel)) = (
            self.irc_connection.get_current_server(),
            self.irc_connection.get_current_channel(),
        ) {
            // Add action message locally (TODO: send CTCP ACTION to server)
            self.irc_connection.irc_state.add_message(
                server_id,
                channel,
                Some("YourNick".to_string()), // TODO: Get actual nickname
                action,
                MessageType::Action,
            );
        }
    }

    /// Handle tab completion
    fn handle_tab_completion(&self) {
        let text = self.get_text();
        let words: Vec<&str> = text.split_whitespace().collect();
        
        if let Some(last_word) = words.last() {
            if last_word.starts_with('/') {
                // Command completion
                self.complete_command(last_word);
            } else if last_word.starts_with('#') || last_word.starts_with('&') {
                // Channel completion
                self.complete_channel(last_word);
            } else {
                // Nick completion
                self.complete_nickname(last_word);
            }
        }
    }

    /// Complete IRC commands
    fn complete_command(&self, partial: &str) {
        let commands = vec![
            "/join", "/part", "/quit", "/nick", "/msg", "/query", "/me", 
            "/clear", "/help", "/list", "/whois", "/kick", "/ban", "/topic"
        ];
        
        let matches: Vec<String> = commands
            .iter()
            .filter(|cmd| cmd.starts_with(partial))
            .map(|s| s.to_string())
            .collect();
        
        self.show_completions(matches);
    }

    /// Complete channel names
    fn complete_channel(&self, partial: &str) {
        let connections = self.irc_connection.get_connections();
        let mut channels = Vec::new();
        
        for connection in connections.values() {
            for channel_name in connection.channels.keys() {
                if channel_name.starts_with(partial) {
                    channels.push(channel_name.clone());
                }
            }
        }
        
        self.show_completions(channels);
    }

    /// Complete nicknames
    fn complete_nickname(&self, partial: &str) {
        let connections = self.irc_connection.get_connections();
        let mut nicknames = Vec::new();
        
        if let Some(current_server) = self.irc_connection.get_current_server() {
            if let Some(connection) = connections.get(&current_server) {
                if let Some(current_channel) = self.irc_connection.get_current_channel() {
                    if let Some(channel_info) = connection.channels.get(&current_channel) {
                        for user in channel_info.users.keys() {
                            if user.starts_with(partial) {
                                nicknames.push(format!("{}: ", user)); // Add colon for mentions
                            }
                        }
                    }
                }
            }
        }
        
        self.show_completions(nicknames);
    }

    /// Show completion suggestions
    fn show_completions(&self, completions: Vec<String>) {
        if !completions.is_empty() {
            self.suggestions.set(completions);
            self.show_suggestions.set(true);
        }
    }

    /// Hide suggestions
    fn hide_suggestions(&self) {
        self.show_suggestions.set(false);
    }

    /// Update suggestions based on current input
    fn update_suggestions(&self) {
        // TODO: Implement real-time suggestions as user types
    }

    /// Navigate command history up
    fn navigate_history_up(&self) {
        let history = self.command_history.read();
        if !history.is_empty() && self.history_index() > 0 {
            let new_index = self.history_index() - 1;
            self.history_index.set(new_index);
            self.set_text(history[new_index].clone());
        }
    }

    /// Navigate command history down  
    fn navigate_history_down(&self) {
        let history = self.command_history.read();
        if !history.is_empty() {
            if self.history_index() < history.len() - 1 {
                let new_index = self.history_index() + 1;
                self.history_index.set(new_index);
                self.set_text(history[new_index].clone());
            } else {
                self.history_index.set(history.len());
                self.clear();
            }
        }
    }

    /// Add text to command history
    fn add_to_history(&self, text: String) {
        self.command_history.write().push(text);
        
        // Limit history size
        let mut history = self.command_history.write();
        if history.len() > 100 {
            history.remove(0);
        }
        
        self.history_index.set(history.len());
    }

    /// Handle IRC formatting shortcuts
    fn handle_formatting_shortcut(&self, key: &str) {
        match key {
            "b" => self.insert_formatting("\u{0002}", "\u{0002}"), // Bold
            "i" => self.insert_formatting("\u{001d}", "\u{001d}"), // Italic  
            "u" => self.insert_formatting("\u{001f}", "\u{001f}"), // Underline
            "k" => self.insert_formatting("\u{0003}", ""), // Color
            _ => {}
        }
    }

    /// Insert IRC formatting codes
    fn insert_formatting(&self, start_code: &str, end_code: &str) {
        let current = self.get_text();
        // TODO: Handle text selection properly
        let new_text = format!("{}{}{}", current, start_code, end_code);
        self.set_text(new_text);
    }

    /// Add system message
    fn add_system_message(&self, message: String) {
        if let Some(server_id) = self.irc_connection.get_current_server() {
            let target = self.irc_connection.get_current_channel()
                .unwrap_or_else(|| server_id.clone());
            
            self.irc_connection.irc_state.add_message(
                server_id,
                target,
                None,
                message,
                MessageType::System,
            );
        }
    }

    /// Add error message
    fn add_error_message(&self, message: String) {
        if let Some(server_id) = self.irc_connection.get_current_server() {
            let target = self.irc_connection.get_current_channel()
                .unwrap_or_else(|| server_id.clone());
            
            self.irc_connection.irc_state.add_message(
                server_id,
                target,
                None,
                message,
                MessageType::Error,
            );
        }
    }

    /// Show help message
    fn show_help(&self) {
        let help_text = r#"Available IRC commands:
/join <channel> - Join a channel
/part - Leave current channel
/quit [message] - Disconnect from server
/nick <nickname> - Change nickname
/msg <nick> <message> - Send private message
/me <action> - Send action message
/clear - Clear message history
/help - Show this help

Keyboard shortcuts:
Ctrl+B - Bold text
Ctrl+I - Italic text
Ctrl+U - Underline text
Ctrl+K - Color codes
Tab - Auto-complete
↑/↓ - Command history
Ctrl+↑/↓ - Scroll messages
Page Up/Down - Scroll messages"#;

        self.add_system_message(help_text.to_string());
    }

    // Scroll management methods
    fn scroll_message_area(&self, lines: i32) {
        // TODO: Implement message area scrolling
    }

    fn scroll_to_top(&self) {
        // TODO: Implement scroll to top
    }

    fn scroll_to_bottom(&self) {
        // TODO: Implement scroll to bottom
    }
}

impl Clone for InputHandlerHook {
    fn clone(&self) -> Self {
        Self {
            input_text: self.input_text.clone(),
            command_history: self.command_history.clone(),
            history_index: self.history_index.clone(),
            suggestions: self.suggestions.clone(),
            show_suggestions: self.show_suggestions.clone(),
            irc_connection: self.irc_connection.clone(),
        }
    }
}