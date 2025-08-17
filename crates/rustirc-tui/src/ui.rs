//! TUI user interface components
//!
//! Renders the terminal interface using ratatui widgets.
//! Features:
//! - Split-pane layout with channel list, messages, and user list
//! - IRC message formatting with colors
//! - Status indicators and activity badges
//! - Help screen and input area

use crate::state::{TuiState, FocusArea, TuiMessage, MessageType};
use crate::input::InputMode;
use crate::themes::{ThemeManager, TuiColors};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::Title, Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};


/// TUI renderer
pub struct TuiRenderer {
    theme_manager: ThemeManager,
    channel_list_state: ListState,
    user_list_state: ListState,
}

impl TuiRenderer {
    pub fn new() -> Self {
        Self {
            theme_manager: ThemeManager::new(),
            channel_list_state: ListState::default(),
            user_list_state: ListState::default(),
        }
    }
    
    /// Get current colors
    fn colors(&self) -> &TuiColors {
        self.theme_manager.colors()
    }

    /// Main render function
    pub fn render(&mut self, frame: &mut Frame, state: &TuiState) {
        if state.show_help {
            self.render_help_screen(frame);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),      // Main content
                Constraint::Length(3),   // Input area
                Constraint::Length(1),   // Status line
            ])
            .split(frame.area());

        // Main content area
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(25),  // Channel list
                Constraint::Min(20),     // Messages
                Constraint::Length(20),  // User list
            ])
            .split(main_layout[0]);

        // Render main components
        self.render_channel_list(frame, content_layout[0], state);
        self.render_message_area(frame, content_layout[1], state);
        self.render_user_list(frame, content_layout[2], state);
        self.render_input_area(frame, main_layout[1], state);
        self.render_status_line(frame, main_layout[2], state);
    }

    /// Render channel list
    fn render_channel_list(&mut self, frame: &mut Frame, area: Rect, state: &TuiState) {
        let focused = state.focus == FocusArea::ChannelList;
        
        let mut items = Vec::new();
        
        for (server_name, server) in &state.servers {
            // Server header
            let server_style = if server.connected {
                Style::default().fg(self.colors().success)
            } else {
                Style::default().fg(self.colors().error)
            };
            
            let server_item = ListItem::new(Line::from(vec![
                Span::styled("📡 ", server_style),
                Span::styled(server_name.clone(), server_style.add_modifier(Modifier::BOLD)),
            ]));
            items.push(server_item);

            // Channels
            for (channel_name, channel) in &server.channels {
                let is_current = Some(server_name) == state.current_server.as_ref() 
                    && Some(channel_name) == server.current_channel.as_ref();
                
                let mut style = Style::default().fg(self.colors().text);
                let mut prefix = "  # ";
                
                if channel.has_highlight {
                    style = style.fg(self.colors().highlight).add_modifier(Modifier::BOLD);
                    prefix = "  ❗ ";
                } else if channel.unread_count > 0 {
                    style = style.fg(self.colors().activity);
                    prefix = "  ● ";
                }
                
                if is_current {
                    style = style.bg(self.colors().primary).fg(self.colors().background);
                }

                let channel_text = if channel.unread_count > 0 {
                    format!("{} ({})", channel_name, channel.unread_count)
                } else {
                    channel_name.clone()
                };

                let channel_item = ListItem::new(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(channel_text, style),
                ]));
                items.push(channel_item);
            }
        }

        // Update selection state first
        let items_len = items.len();
        if focused {
            self.channel_list_state.select(Some(state.selected_channel_index.min(items_len.saturating_sub(1))));
        } else {
            self.channel_list_state.select(None);
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if focused {
                        Style::default().fg(self.colors().border_focused)
                    } else {
                        Style::default().fg(self.colors().border)
                    })
                    .title("Channels")
            )
            .highlight_style(Style::default().bg(self.colors().secondary));

        frame.render_stateful_widget(list, area, &mut self.channel_list_state);
    }

    /// Render message area
    fn render_message_area(&mut self, frame: &mut Frame, area: Rect, state: &TuiState) {
        let focused = state.focus == FocusArea::MessageArea;
        
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if focused {
                Style::default().fg(self.colors().border_focused)
            } else {
                Style::default().fg(self.colors().border)
            })
            .title(
                state.current_channel()
                    .map(|ch| format!("Messages - {}", ch))
                    .unwrap_or_else(|| "Messages".to_string())
            );

        if let Some(channel) = state.current_channel_state() {
            let mut lines = Vec::new();
            
            // Show recent messages (with scrolling consideration)
            let message_count = channel.messages.len();
            let visible_lines = (area.height as usize).saturating_sub(2); // Account for borders
            let start_index = if message_count > visible_lines {
                message_count - visible_lines + channel.scroll_position
            } else {
                0
            };

            for (i, message) in channel.messages.iter().enumerate() {
                if i < start_index {
                    continue;
                }
                if lines.len() >= visible_lines {
                    break;
                }

                let formatted_line = self.format_message(message);
                lines.push(formatted_line);
            }

            let text = Text::from(lines);
            let paragraph = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: true })
                .scroll((0, 0));

            frame.render_widget(paragraph, area);
        } else {
            // No channel selected
            let welcome_text = Text::from(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Welcome to RustIRC!",
                    Style::default().fg(self.colors().primary).add_modifier(Modifier::BOLD)
                )),
                Line::from(""),
                Line::from("Connect to a server to start chatting."),
                Line::from(""),
                Line::from("Commands:"),
                Line::from("  /connect <server> [port]"),
                Line::from("  /join <channel>"),
                Line::from("  /help - Show help"),
                Line::from(""),
                Line::from("Navigation:"),
                Line::from("  Tab - Switch focus areas"),
                Line::from("  j/k - Navigate up/down"),
                Line::from("  h/l - Navigate left/right"),
                Line::from("  ? - Toggle help"),
            ]);

            let paragraph = Paragraph::new(welcome_text)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, area);
        }
    }

    /// Render user list
    fn render_user_list(&mut self, frame: &mut Frame, area: Rect, state: &TuiState) {
        let focused = state.focus == FocusArea::UserList;
        
        let mut items = Vec::new();
        
        if let Some(channel) = state.current_channel_state() {
            for (i, user) in channel.users.iter().enumerate() {
                let is_selected = focused && i == state.selected_user_index;
                
                let style = if is_selected {
                    Style::default().bg(self.colors().secondary)
                } else {
                    Style::default().fg(self.colors().text)
                };

                let user_item = ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(user.clone(), style),
                ]));
                items.push(user_item);
            }
        }

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if focused {
                        Style::default().fg(self.colors().border_focused)
                    } else {
                        Style::default().fg(self.colors().border)
                    })
                    .title(format!("Users ({})", 
                        state.current_channel_state()
                            .map(|ch| ch.users.len())
                            .unwrap_or(0)
                    ))
            );

        frame.render_widget(list, area);
    }

    /// Render input area
    fn render_input_area(&mut self, frame: &mut Frame, area: Rect, state: &TuiState) {
        let focused = state.focus == FocusArea::Input;
        
        // Input mode indicator
        let mode_text = match state.focus {
            FocusArea::Input => "INSERT",
            _ => "NORMAL",
        };
        
        let mode_style = match state.focus {
            FocusArea::Input => Style::default().fg(self.colors().success),
            _ => Style::default().fg(self.colors().text_muted),
        };

        let input_text = format!("[{}] {}", mode_text, state.input_buffer);
        
        let input_paragraph = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if focused {
                        Style::default().fg(self.colors().border_focused)
                    } else {
                        Style::default().fg(self.colors().border)
                    })
                    .title("Input")
            );

        frame.render_widget(input_paragraph, area);

        // Show cursor if focused
        if focused {
            let cursor_x = area.x + 1 + mode_text.len() as u16 + 3 + state.input_cursor as u16;
            let cursor_y = area.y + 1;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }

    /// Render status line
    fn render_status_line(&mut self, frame: &mut Frame, area: Rect, state: &TuiState) {
        let server_status = if let Some(server_name) = &state.current_server {
            if let Some(server) = state.servers.get(server_name) {
                if server.connected {
                    format!("Connected to {}", server_name)
                } else {
                    format!("Disconnected from {}", server_name)
                }
            } else {
                "No server".to_string()
            }
        } else {
            "No connection".to_string()
        };

        let unread_count = state.total_unread_count();
        let unread_text = if unread_count > 0 {
            format!(" | {} unread", unread_count)
        } else {
            String::new()
        };

        let status_text = format!("{}{}  | Ctrl+C to quit | ? for help", server_status, unread_text);
        
        let status_paragraph = Paragraph::new(status_text)
            .style(Style::default().fg(self.colors().text_muted));

        frame.render_widget(status_paragraph, area);
    }

    /// Render help screen
    fn render_help_screen(&mut self, frame: &mut Frame) {
        let area = frame.area();
        
        // Clear the background
        frame.render_widget(Clear, area);
        
        // Create centered popup
        let popup_area = self.centered_rect(80, 80, area);
        
        let help_text = Text::from(vec![
            Line::from(Span::styled("RustIRC Help", Style::default().fg(self.colors().primary).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("Navigation (Normal Mode):", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("  h/← - Move left"),
            Line::from("  j/↓ - Move down"),
            Line::from("  k/↑ - Move up"),
            Line::from("  l/→ - Move right"),
            Line::from("  Tab - Next pane"),
            Line::from("  Shift+Tab - Previous pane"),
            Line::from("  Ctrl+N - Next channel"),
            Line::from("  Ctrl+P - Previous channel"),
            Line::from(""),
            Line::from(Span::styled("Input Modes:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("  i - Enter insert mode"),
            Line::from("  : - Enter command mode"),
            Line::from("  Esc - Return to normal mode"),
            Line::from(""),
            Line::from(Span::styled("Commands:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("  /connect <server> [port] - Connect to IRC server"),
            Line::from("  /join <channel> - Join a channel"),
            Line::from("  /part [reason] - Leave current channel"),
            Line::from("  /quit [reason] - Quit IRC"),
            Line::from("  /nick <nickname> - Change nickname"),
            Line::from("  /msg <user> <message> - Send private message"),
            Line::from(""),
            Line::from(Span::styled("Global Keys:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("  Ctrl+C/Ctrl+Q - Quit application"),
            Line::from("  ? - Toggle this help screen"),
            Line::from(""),
            Line::from("Press ? or Esc to close this help screen"),
        ]);

        let help_paragraph = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.colors().border_focused))
                    .title("Help")
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(help_paragraph, popup_area);
    }

    /// Format a message for display
    fn format_message<'a>(&self, message: &'a TuiMessage) -> Line<'a> {
        use crate::formatting::{parse_irc_text, spans_to_line, replace_emoticons};
        
        let timestamp = self.format_timestamp(&message.timestamp);
        
        let nick_style = match message.message_type {
            MessageType::Message => Style::default().fg(self.colors().primary),
            MessageType::Action => Style::default().fg(self.colors().accent),
            MessageType::Join => Style::default().fg(self.colors().success),
            MessageType::Part | MessageType::Quit => Style::default().fg(self.colors().error),
            MessageType::Notice => Style::default().fg(self.colors().activity),
            MessageType::System => Style::default().fg(self.colors().text_muted),
            _ => Style::default().fg(self.colors().text),
        };

        let prefix = match message.message_type {
            MessageType::Action => "* ",
            MessageType::Join => "→ ",
            MessageType::Part => "← ",
            MessageType::Quit => "⚠ ",
            MessageType::Notice => "! ",
            MessageType::System => "*** ",
            _ => "<",
        };

        let suffix = match message.message_type {
            MessageType::Action | MessageType::Join | MessageType::Part | 
            MessageType::Quit | MessageType::Notice | MessageType::System => "",
            _ => ">",
        };

        // Parse IRC formatting in message content
        let content_with_emotes = replace_emoticons(&message.content);
        let formatted_spans = parse_irc_text(&content_with_emotes);
        
        // Build the complete message line
        let mut line_spans = vec![
            Span::styled(timestamp, Style::default().fg(self.colors().text_muted)),
            Span::raw(" "),
            Span::styled(prefix, nick_style),
            Span::styled(&message.nick, nick_style),
            Span::styled(suffix, nick_style),
            Span::raw(" "),
        ];
        
        // Add formatted content spans
        for span in formatted_spans {
            let mut style = Style::default();
            
            // Apply IRC formatting colors
            if let Some(fg) = span.foreground {
                style = style.fg(fg);
            }
            if let Some(bg) = span.background {
                style = style.bg(bg);
            }
            
            // Apply reverse video
            if span.reverse {
                let fg = style.fg.unwrap_or(self.colors().text);
                let bg = style.bg.unwrap_or(self.colors().background);
                style = style.fg(bg).bg(fg);
            }
            
            // Apply text formatting
            if span.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if span.italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            if span.underline || span.is_url {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            if span.strikethrough {
                style = style.add_modifier(Modifier::CROSSED_OUT);
            }
            
            // Override with highlight style if necessary
            if message.is_highlight {
                style = style.fg(self.colors().highlight).add_modifier(Modifier::BOLD);
            } else if message.is_own_message && span.foreground.is_none() {
                style = style.fg(self.colors().success);
            } else if span.foreground.is_none() {
                style = style.fg(self.colors().text);
            }
            
            line_spans.push(Span::styled(span.text, style));
        }

        Line::from(line_spans)
    }

    /// Format timestamp
    fn format_timestamp(&self, timestamp: &SystemTime) -> String {
        if let Ok(duration) = timestamp.duration_since(UNIX_EPOCH) {
            let secs = duration.as_secs();
            let hours = (secs / 3600) % 24;
            let minutes = (secs / 60) % 60;
            format!("{:02}:{:02}", hours, minutes)
        } else {
            "??:??".to_string()
        }
    }

    /// Helper to create centered rectangle
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    /// Set color theme
    pub fn set_theme(&mut self, theme: crate::themes::ThemeName) {
        self.theme_manager.set_theme(theme);
    }
    
    /// Switch to next theme
    pub fn next_theme(&mut self) {
        self.theme_manager.next_theme();
    }
    
    /// Switch to previous theme
    pub fn previous_theme(&mut self) {
        self.theme_manager.previous_theme();
    }
    
    /// Get current theme name
    pub fn current_theme(&self) -> crate::themes::ThemeName {
        self.theme_manager.current_theme()
    }
}

impl Default for TuiRenderer {
    fn default() -> Self {
        Self::new()
    }
}
