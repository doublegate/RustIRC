//! Input area widget for RustIRC GUI
//!
//! Handles message input with command history, auto-completion, and multi-line support.
//! Features command processing, nick completion, and input validation.

use crate::state::{AppState, TabType};
use crate::theme::Theme;
use iced::{
    widget::{container, text_input, button, row, column, text, Space},
    Element, Length, Task, Color, Alignment,
    keyboard::{Key, Modifiers},
};
use std::collections::VecDeque;

/// Messages for input area interactions
#[derive(Debug, Clone)]
pub enum InputAreaMessage {
    InputChanged(String),
    SendMessage(String),
    InputSubmitted(String),
    TabCompleted(String),
    ToggleMultiline,
    HistoryUp,
    HistoryDown,
    TabCompletion,
    ClearInput,
    PasteText(String),
    KeyPressed(Key, Modifiers),
}

/// Input area widget state
#[derive(Debug, Clone)]
pub struct InputArea {
    current_input: String,
    input_history: VecDeque<String>,
    history_index: Option<usize>,
    completion_candidates: Vec<String>,
    completion_index: Option<usize>,
    completion_prefix: String,
    multiline_mode: bool,
    max_history_size: usize,
}

impl InputArea {
    pub fn new() -> Self {
        Self {
            current_input: String::new(),
            input_history: VecDeque::new(),
            history_index: None,
            completion_candidates: Vec::new(),
            completion_index: None,
            completion_prefix: String::new(),
            multiline_mode: false,
            max_history_size: 100,
        }
    }

    /// Update the input area state
    pub fn update(&mut self, message: InputAreaMessage, app_state: &mut AppState) -> Task<InputAreaMessage> {
        match message {
            InputAreaMessage::InputChanged(value) => {
                self.current_input = value;
                self.reset_completion();
                self.history_index = None;
                Task::none()
            }
            InputAreaMessage::SendMessage(text) => {
                if !text.trim().is_empty() {
                    self.add_to_history(text.clone());
                    self.current_input.clear();
                    self.reset_completion();
                    self.history_index = None;
                }
                Task::none()
            }
            InputAreaMessage::InputSubmitted(text) => {
                // Handle multiline input submission
                if !text.trim().is_empty() {
                    self.add_to_history(text.clone());
                    self.current_input.clear();
                    self.reset_completion();
                    self.history_index = None;
                }
                Task::none()
            }
            InputAreaMessage::TabCompleted(text) => {
                // Handle tab completion result
                self.current_input = text;
                self.reset_completion();
                Task::none()
            }
            InputAreaMessage::ToggleMultiline => {
                self.toggle_multiline();
                Task::none()
            }
            InputAreaMessage::HistoryUp => {
                if !self.input_history.is_empty() {
                    match self.history_index {
                        None => {
                            self.history_index = Some(self.input_history.len() - 1);
                            if let Some(entry) = self.input_history.back() {
                                self.current_input = entry.clone();
                            }
                        }
                        Some(index) if index > 0 => {
                            self.history_index = Some(index - 1);
                            if let Some(entry) = self.input_history.get(index - 1) {
                                self.current_input = entry.clone();
                            }
                        }
                        _ => {} // Already at the beginning
                    }
                }
                Task::none()
            }
            InputAreaMessage::HistoryDown => {
                match self.history_index {
                    Some(index) if index < self.input_history.len() - 1 => {
                        self.history_index = Some(index + 1);
                        if let Some(entry) = self.input_history.get(index + 1) {
                            self.current_input = entry.clone();
                        }
                    }
                    Some(_) => {
                        self.history_index = None;
                        self.current_input.clear();
                    }
                    None => {} // Not in history mode
                }
                Task::none()
            }
            InputAreaMessage::TabCompletion => {
                self.handle_tab_completion(app_state);
                Task::none()
            }
            InputAreaMessage::ClearInput => {
                self.current_input.clear();
                self.reset_completion();
                self.history_index = None;
                Task::none()
            }
            InputAreaMessage::PasteText(text) => {
                self.current_input.push_str(&text);
                self.reset_completion();
                Task::none()
            }
            InputAreaMessage::KeyPressed(key, modifiers) => {
                self.handle_key_press(key, modifiers, app_state)
            }
        }
    }

    /// Render the input area
    pub fn view(&self, app_state: &AppState) -> Element<InputAreaMessage> {
        // Create theme instance for theming support
        let theme = Theme::default();
        
        let current_tab = app_state.current_tab();
        
        // Determine input placeholder
        let placeholder = if let Some(tab) = current_tab {
            match &tab.tab_type {
                TabType::Channel { channel } => format!("Message {}...", channel),
                TabType::PrivateMessage { nick } => format!("Message {}...", nick),
                TabType::Server => "Enter IRC command...".to_string(),
                TabType::Private => format!("Message {}...", tab.name),
            }
        } else {
            "No active tab".to_string()
        };

        // Input field
        let input_field = text_input(&placeholder, &self.current_input)
            .on_input(InputAreaMessage::InputChanged)
            .on_submit(InputAreaMessage::SendMessage(self.current_input.clone()))
            .size(14.0)
            .padding(8)
            .width(Length::Fill);

        // Send button
        let send_button = button(
            text("Send")
                .size(12.0)
                .color(theme.get_text_color())
        )
        .on_press(InputAreaMessage::SendMessage(self.current_input.clone()))
        .padding([6, 12]);

        // Format indicators (if needed)
        let format_info = if self.current_input.starts_with('/') {
            text("Command")
                .size(10.0)
                .color(Color::from_rgb(0.0, 0.6, 1.0))
        } else {
            text("")
        };

        // Completion hint
        let completion_hint = if !self.completion_candidates.is_empty() {
            let hint_text = if let Some(index) = self.completion_index {
                format!("Tab: {} ({}/{})", 
                    self.completion_candidates.get(index).unwrap_or(&String::new()),
                    index + 1,
                    self.completion_candidates.len()
                )
            } else {
                format!("Tab: {} candidates", self.completion_candidates.len())
            };
            
            text(hint_text)
                .size(10.0)
                .color(Color::from_rgb(0.6, 0.6, 0.6))
        } else {
            text("")
        };

        // Main input row
        let input_row = row![
            input_field,
            Space::with_width(Length::Fixed(8.0)),
            send_button
        ]
        .align_y(Alignment::Center);

        // Info row (format and completion hints)
        let info_row = row![
            format_info,
            Space::with_width(Length::Fill),
            completion_hint
        ]
        .align_y(Alignment::Center);

        let content = if self.multiline_mode {
            // Implement multiline input mode
            let multiline_input = text_input("Type multiple lines... (Ctrl+Enter to send)", &self.current_input)
                .on_input(InputAreaMessage::InputChanged)
                .on_submit(InputAreaMessage::InputSubmitted(self.current_input.clone()))
                .padding(8)
                .width(Length::Fill);
                
            let send_button = button("Send")
                .on_press(InputAreaMessage::InputSubmitted(self.current_input.clone()))
                .padding([4, 8]);
                
            let toggle_button = button("Single Line")
                .on_press(InputAreaMessage::ToggleMultiline)
                .padding([4, 8]);
                
            column![
                multiline_input,
                row![
                    send_button,
                    toggle_button,
                ].spacing(5).align_y(Alignment::Center),
                Space::with_height(Length::Fixed(4.0)),
                info_row
            ]
        } else {
            column![
                input_row,
                Space::with_height(Length::Fixed(4.0)),
                info_row
            ]
        };

        container(content)
            .padding(8)
            .width(Length::Fill)
            .into()
    }

    /// Handle key press events
    fn handle_key_press(&mut self, key: Key, modifiers: Modifiers, app_state: &AppState) -> Task<InputAreaMessage> {
        match key {
            Key::Named(iced::keyboard::key::Named::ArrowUp) if modifiers.control() => {
                Task::done(InputAreaMessage::HistoryUp)
            }
            Key::Named(iced::keyboard::key::Named::ArrowDown) if modifiers.control() => {
                Task::done(InputAreaMessage::HistoryDown)
            }
            Key::Named(iced::keyboard::key::Named::Tab) => {
                Task::done(InputAreaMessage::TabCompletion)
            }
            Key::Named(iced::keyboard::key::Named::Escape) => {
                self.reset_completion();
                Task::none()
            }
            Key::Named(iced::keyboard::key::Named::Enter) if modifiers.control() => {
                // Ctrl+Enter for multiline (if implemented)
                Task::none()
            }
            _ => Task::none()
        }
    }

    /// Handle tab completion
    fn handle_tab_completion(&mut self, app_state: &AppState) {
        if self.current_input.is_empty() {
            return;
        }

        // If we're already in completion mode, cycle through candidates
        if !self.completion_candidates.is_empty() {
            self.cycle_completion();
            return;
        }

        // Extract the word to complete (last word before cursor)
        let words: Vec<&str> = self.current_input.split_whitespace().collect();
        if let Some(&last_word) = words.last() {
            self.completion_prefix = last_word.to_string();
            
            // Get completion candidates
            self.completion_candidates = self.get_completion_candidates(last_word, app_state);
            
            if !self.completion_candidates.is_empty() {
                self.completion_index = Some(0);
                self.apply_completion();
            }
        }
    }

    /// Get completion candidates for a prefix
    fn get_completion_candidates(&self, prefix: &str, app_state: &AppState) -> Vec<String> {
        let mut candidates = Vec::new();
        let lower_prefix = prefix.to_lowercase();

        // Task completion
        if prefix.starts_with('/') {
            let commands = [
                "/join", "/part", "/quit", "/nick", "/msg", "/notice", "/me", "/topic",
                "/kick", "/ban", "/unban", "/mode", "/whois", "/away", "/back",
                "/connect", "/disconnect", "/server", "/clear", "/help"
            ];
            
            for &command in &commands {
                if command.to_lowercase().starts_with(&lower_prefix) {
                    candidates.push(command.to_string());
                }
            }
        } else {
            // Nick completion
            if let Some(current_tab) = app_state.current_tab() {
                for (nick, _user) in &current_tab.users {
                    if nick.to_lowercase().starts_with(&lower_prefix) {
                        // Add colon if it's the first word (mention)
                        let completion = if self.current_input.split_whitespace().count() <= 1 {
                            format!("{}: ", nick)
                        } else {
                            nick.clone()
                        };
                        candidates.push(completion);
                    }
                }
            }

            // Channel completion (if applicable)
            if prefix.starts_with('#') || prefix.starts_with('&') {
                for (server_id, server_state) in &app_state.servers {
                    for (channel_name, _channel_state) in &server_state.channels {
                        if channel_name.to_lowercase().starts_with(&lower_prefix) {
                            candidates.push(channel_name.clone());
                        }
                    }
                }
            }
        }

        // Sort candidates
        candidates.sort();
        candidates
    }

    /// Cycle through completion candidates
    fn cycle_completion(&mut self) {
        if let Some(current_index) = self.completion_index {
            let next_index = (current_index + 1) % self.completion_candidates.len();
            self.completion_index = Some(next_index);
            self.apply_completion();
        }
    }

    /// Apply the current completion
    fn apply_completion(&mut self) {
        if let (Some(index), Some(candidate)) = (
            self.completion_index,
            self.completion_candidates.get(self.completion_index.unwrap_or(0))
        ) {
            // Replace the last word with the completion
            let mut words: Vec<&str> = self.current_input.split_whitespace().collect();
            if !words.is_empty() {
                words.pop(); // Remove the incomplete word
                words.push(candidate);
                self.current_input = words.join(" ");
            }
        }
    }

    /// Reset completion state
    fn reset_completion(&mut self) {
        self.completion_candidates.clear();
        self.completion_index = None;
        self.completion_prefix.clear();
    }

    /// Add input to history
    fn add_to_history(&mut self, input: String) {
        // Don't add empty or duplicate entries
        if input.trim().is_empty() || self.input_history.back() == Some(&input) {
            return;
        }

        self.input_history.push_back(input);

        // Limit history size
        if self.input_history.len() > self.max_history_size {
            self.input_history.pop_front();
        }
    }

    /// Get current input
    pub fn current_input(&self) -> &str {
        &self.current_input
    }

    /// Set input text
    pub fn set_input(&mut self, text: String) {
        self.current_input = text;
        self.reset_completion();
    }

    /// Clear input
    pub fn clear(&mut self) {
        self.current_input.clear();
        self.reset_completion();
        self.history_index = None;
    }

    /// Toggle multiline mode
    pub fn toggle_multiline(&mut self) {
        self.multiline_mode = !self.multiline_mode;
    }

    /// Get input history
    pub fn history(&self) -> &VecDeque<String> {
        &self.input_history
    }
}

impl Default for InputArea {
    fn default() -> Self {
        Self::new()
    }
}