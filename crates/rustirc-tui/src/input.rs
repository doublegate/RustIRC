//! Input handling for TUI
//!
//! Handles keyboard input and vi-like navigation for the terminal interface.
//! Features:
//! - Vi-like keybindings for navigation
//! - Input modes (normal, insert, command)
//! - Command completion and history
//! - Focus management between panes

use crate::state::{TuiState, FocusArea};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use tracing::info;

/// Input modes for vi-like interface
#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,   // Navigation mode
    Insert,   // Text input mode
    Command,  // Command mode (started with :)
}

/// TUI actions matching GUI functionality
#[derive(Debug, Clone)]
pub enum TuiAction {
    // Input and navigation
    TabComplete,
    HistoryPrevious,
    HistoryNext,
    ScrollUp,
    ScrollDown,
    ShowHelp,
    
    // Tab management
    NextTab,
    PreviousTab,
    CloseTab,
    
    // Theme management
    NextTheme,
    PreviousTheme,
    
    // Connection management
    Connect,
    Disconnect,
    JoinChannel,
    PartChannel,
    
    // Message actions
    SendMessage(String),
    PrivateMessage(String),
    
    // UI toggles
    ToggleUserList,
    ToggleServerTree,
    ToggleTimestamps,
    ToggleJoinPart,
    
    // Other
    Quit,
    None,
}

/// Represents a key event
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

/// Input handler for TUI
pub struct InputHandler {
    /// Current input mode
    mode: InputMode,
    /// Last command for repeat functionality
    last_command: Option<String>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Insert, // Start in insert mode for user-friendly experience
            last_command: None,
        }
    }

    /// Get current input mode
    pub fn current_mode(&self) -> InputMode {
        self.mode.clone()
    }

    /// Handle a key event and return optional action
    pub fn handle_key(&mut self, key: KeyEvent, state: &mut TuiState) -> Result<TuiAction> {
        match self.mode {
            InputMode::Normal => self.handle_normal_mode(key, state),
            InputMode::Insert => self.handle_insert_mode(key, state),
            InputMode::Command => self.handle_command_mode(key, state),
        }
    }

    /// Handle keys in normal (navigation) mode
    fn handle_normal_mode(&mut self, key: KeyEvent, state: &mut TuiState) -> Result<TuiAction> {
        match key.code {
            // Mode switches
            KeyCode::Char('i') => {
                self.mode = InputMode::Insert;
                state.set_focus(FocusArea::Input);
            }
            KeyCode::Char(':') => {
                self.mode = InputMode::Command;
                state.set_focus(FocusArea::Input);
                state.insert_char(':');
            }
            
            // Navigation
            KeyCode::Char('h') | KeyCode::Left => {
                match state.focus {
                    FocusArea::ChannelList => {
                        // Stay in channel list
                    }
                    FocusArea::MessageArea => {
                        state.set_focus(FocusArea::ChannelList);
                    }
                    FocusArea::UserList => {
                        state.set_focus(FocusArea::MessageArea);
                    }
                    FocusArea::Input => {
                        state.set_focus(FocusArea::MessageArea);
                    }
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                match state.focus {
                    FocusArea::ChannelList => {
                        state.set_focus(FocusArea::MessageArea);
                    }
                    FocusArea::MessageArea => {
                        state.set_focus(FocusArea::UserList);
                    }
                    FocusArea::UserList => {
                        // Stay in user list
                    }
                    FocusArea::Input => {
                        state.set_focus(FocusArea::UserList);
                    }
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match state.focus {
                    FocusArea::ChannelList => {
                        state.next_channel();
                    }
                    FocusArea::MessageArea => {
                        // Scroll down in messages
                        if let Some(channel) = state.current_channel_state_mut() {
                            if channel.scroll_position > 0 {
                                channel.scroll_position -= 1;
                            }
                        }
                    }
                    FocusArea::UserList => {
                        state.selected_user_index = state.selected_user_index.saturating_add(1);
                    }
                    FocusArea::Input => {
                        state.set_focus(FocusArea::MessageArea);
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match state.focus {
                    FocusArea::ChannelList => {
                        state.previous_channel();
                    }
                    FocusArea::MessageArea => {
                        // Scroll up in messages
                        if let Some(channel) = state.current_channel_state_mut() {
                            channel.scroll_position += 1;
                        }
                    }
                    FocusArea::UserList => {
                        state.selected_user_index = state.selected_user_index.saturating_sub(1);
                    }
                    FocusArea::Input => {
                        state.set_focus(FocusArea::MessageArea);
                    }
                }
            }
            
            // Tab navigation
            KeyCode::Tab => {
                state.next_focus();
            }
            KeyCode::BackTab => {
                // Previous focus (Shift+Tab)
                match state.focus {
                    FocusArea::ChannelList => state.set_focus(FocusArea::Input),
                    FocusArea::MessageArea => state.set_focus(FocusArea::ChannelList),
                    FocusArea::UserList => state.set_focus(FocusArea::MessageArea),
                    FocusArea::Input => state.set_focus(FocusArea::UserList),
                }
            }
            
            // Channel navigation
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.next_channel();
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.previous_channel();
            }
            
            // Function keys
            KeyCode::F(1) => {
                state.toggle_help();
            }
            KeyCode::F(2) => {
                // Toggle channel list
                state.toggle_channel_list();
            }
            KeyCode::F(3) => {
                // Toggle user list
                state.toggle_user_list();
            }
            KeyCode::F(12) => {
                // Switch theme (F12)
                return Ok(TuiAction::NextTheme);
            }
            
            // Page navigation
            KeyCode::PageUp => {
                match state.focus {
                    FocusArea::MessageArea => {
                        // Scroll up by page
                        if let Some(channel) = state.current_channel_state_mut() {
                            channel.scroll_position += 10;
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::PageDown => {
                match state.focus {
                    FocusArea::MessageArea => {
                        // Scroll down by page
                        if let Some(channel) = state.current_channel_state_mut() {
                            if channel.scroll_position >= 10 {
                                channel.scroll_position -= 10;
                            } else {
                                channel.scroll_position = 0;
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // Home/End keys
            KeyCode::Home => {
                match state.focus {
                    FocusArea::MessageArea => {
                        // Scroll to top
                        if let Some(channel) = state.current_channel_state_mut() {
                            channel.scroll_position = channel.messages.len().saturating_sub(1);
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::End => {
                match state.focus {
                    FocusArea::MessageArea => {
                        // Scroll to bottom
                        if let Some(channel) = state.current_channel_state_mut() {
                            channel.scroll_position = 0;
                        }
                    }
                    _ => {}
                }
            }
            
            // Enter channel or activate
            KeyCode::Enter => {
                match state.focus {
                    FocusArea::ChannelList => {
                        // Channel already selected by navigation
                    }
                    FocusArea::Input => {
                        self.mode = InputMode::Insert;
                    }
                    _ => {}
                }
            }
            
            // Help
            KeyCode::Char('?') => {
                state.toggle_help();
            }
            
            // Quick commands
            KeyCode::Char('o') => {
                // Open - could open connection dialog
                return Ok(TuiAction::Connect);
            }
            
            _ => {}
        }
        
        Ok(TuiAction::None)
    }

    /// Handle keys in insert (text input) mode
    fn handle_insert_mode(&mut self, key: KeyEvent, state: &mut TuiState) -> Result<TuiAction> {
        match key.code {
            // Exit insert mode
            KeyCode::Esc => {
                self.mode = InputMode::Normal;
                state.set_focus(FocusArea::MessageArea);
            }
            
            // Submit input
            KeyCode::Enter => {
                let command = state.submit_input();
                return Ok(if command.is_empty() { 
                    TuiAction::None 
                } else { 
                    TuiAction::SendMessage(command) 
                });
            }
            
            // Clear input (must come before general Char pattern)
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                state.clear_input();
            }
            
            // Text editing
            KeyCode::Char(c) => {
                state.insert_char(c);
            }
            KeyCode::Backspace => {
                state.delete_char();
            }
            
            // Cursor movement
            KeyCode::Left => {
                state.move_cursor_left();
            }
            KeyCode::Right => {
                state.move_cursor_right();
            }
            KeyCode::Home => {
                state.input_cursor = 0;
            }
            KeyCode::End => {
                state.input_cursor = state.input_buffer.len();
            }
            
            // History navigation
            KeyCode::Up => {
                state.history_up();
            }
            KeyCode::Down => {
                state.history_down();
            }
            
            // Tab completion (basic)
            KeyCode::Tab => {
                self.handle_tab_completion(state);
            }
            
            _ => {}
        }
        
        Ok(TuiAction::None)
    }

    /// Handle keys in command mode
    fn handle_command_mode(&mut self, key: KeyEvent, state: &mut TuiState) -> Result<TuiAction> {
        match key.code {
            // Exit command mode
            KeyCode::Esc => {
                self.mode = InputMode::Normal;
                state.clear_input();
                state.set_focus(FocusArea::MessageArea);
            }
            
            // Execute command
            KeyCode::Enter => {
                let command = state.submit_input();
                self.mode = InputMode::Normal;
                state.set_focus(FocusArea::MessageArea);
                
                if !command.is_empty() {
                    self.last_command = Some(command.clone());
                    return Ok(TuiAction::SendMessage(command));
                }
            }
            
            // Text editing (similar to insert mode but for commands)
            KeyCode::Char(c) => {
                state.insert_char(c);
            }
            KeyCode::Backspace => {
                state.delete_char();
                // If we delete the ':' at the beginning, exit command mode
                if state.input_buffer.is_empty() {
                    self.mode = InputMode::Insert;
                }
            }
            
            // Cursor movement
            KeyCode::Left => {
                state.move_cursor_left();
            }
            KeyCode::Right => {
                state.move_cursor_right();
            }
            KeyCode::Home => {
                state.input_cursor = 1; // After the ':'
            }
            KeyCode::End => {
                state.input_cursor = state.input_buffer.len();
            }
            
            // Command history
            KeyCode::Up => {
                state.history_up();
            }
            KeyCode::Down => {
                state.history_down();
            }
            
            // Tab completion for commands
            KeyCode::Tab => {
                self.handle_command_completion(state);
            }
            
            _ => {}
        }
        
        Ok(TuiAction::None)
    }

    /// Handle tab completion for nicknames and channels
    fn handle_tab_completion(&mut self, state: &mut TuiState) {
        // Simple tab completion - in a real implementation this would be more sophisticated
        let input = &state.input_buffer;
        let cursor = state.input_cursor;
        
        // Use cursor position for more precise completion
        info!("Tab completion at cursor position: {}", cursor);
        
        // Find the word at cursor
        let words: Vec<&str> = input.split_whitespace().collect();
        if let Some(last_word) = words.last() {
            if last_word.starts_with('#') {
                // Channel completion
                let channels = state.current_server_channels();
                for channel in channels {
                    if channel.starts_with(last_word) {
                        // Replace the partial channel name
                        let new_input = input.replace(last_word, channel);
                        state.input_buffer = new_input;
                        state.input_cursor = state.input_buffer.len();
                        break;
                    }
                }
            }
            // Could add nickname completion here
        }
    }

    /// Handle command completion
    fn handle_command_completion(&mut self, state: &mut TuiState) {
        let input = &state.input_buffer;
        if input.starts_with(':') {
            let command_part = &input[1..];
            
            let commands = vec![
                "connect", "join", "part", "quit", "msg", "nick", "topic", 
                "kick", "ban", "unban", "mode", "whois", "help", "clear"
            ];
            
            for command in commands {
                if command.starts_with(command_part) {
                    state.input_buffer = format!(":{}", command);
                    state.input_cursor = state.input_buffer.len();
                    break;
                }
            }
        }
    }

    /// Set input mode
    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    /// Repeat last command
    pub fn repeat_last_command(&self) -> Option<String> {
        self.last_command.clone()
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
