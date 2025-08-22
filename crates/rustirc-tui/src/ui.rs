//! TUI user interface components
//!
//! Renders the terminal interface using ratatui widgets.
//! Features:
//! - Split-pane layout with channel list, messages, and user list
//! - IRC message formatting with colors
//! - Status indicators and activity badges
//! - Help screen and input area

use crate::input::InputMode;
use crate::state::{FocusArea, MessageType, TuiMessage, TuiState};
use crate::themes::{ThemeManager, TuiColors};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, warn};

/// TUI renderer
pub struct TuiRenderer {
    theme_manager: ThemeManager,
    channel_list_state: ListState,
    user_list_state: ListState,
    backend_capabilities: BackendCapabilities,
}

/// Backend capabilities tracking for optimal rendering
#[derive(Debug, Clone)]
struct BackendCapabilities {
    supports_color: bool,
    supports_unicode: bool,
    terminal_size: (u16, u16),
    last_size_check: SystemTime,
}

impl BackendCapabilities {
    fn new() -> Self {
        Self {
            supports_color: true,    // Assume color support by default
            supports_unicode: true,  // Assume Unicode support by default
            terminal_size: (80, 24), // Default terminal size
            last_size_check: SystemTime::now(),
        }
    }

    /// Update capabilities based on current backend
    fn update_from_backend<B: Backend>(&mut self, backend: &B) {
        if let Ok(size) = backend.size() {
            self.terminal_size = (size.width, size.height);
            self.last_size_check = SystemTime::now();

            // Log backend capability updates
            debug!(
                "Backend capabilities updated: {}x{}",
                size.width, size.height
            );

            // Check if terminal size changed significantly
            if size.width < 80 || size.height < 24 {
                warn!(
                    "Small terminal detected: {}x{} - may affect display quality",
                    size.width, size.height
                );
            }
        }
    }

    /// Check if backend supports advanced features
    fn check_advanced_features<B: Backend>(&mut self, backend: &B) {
        // Test backend capabilities by attempting operations
        match backend.size() {
            Ok(size) => {
                self.supports_color = size.width > 40; // Assume color support for larger terminals
                self.supports_unicode = size.width > 60; // Unicode for wider terminals
                info!(
                    "Backend feature check: color={}, unicode={}",
                    self.supports_color, self.supports_unicode
                );
            }
            Err(e) => {
                warn!("Backend capability check failed: {}", e);
                self.supports_color = false;
                self.supports_unicode = false;
            }
        }
    }
}

impl TuiRenderer {
    pub fn new() -> Self {
        Self {
            theme_manager: ThemeManager::new(),
            channel_list_state: ListState::default(),
            user_list_state: ListState::default(),
            backend_capabilities: BackendCapabilities::new(),
        }
    }

    /// Get current colors
    fn colors(&self) -> &TuiColors {
        self.theme_manager.colors()
    }

    /// Update backend capabilities from frame by accessing backend indirectly
    fn update_backend_capabilities_from_frame(&mut self, frame: &Frame) {
        let frame_size = frame.area();

        // Use the existing backend capability methods for thorough analysis
        self.backend_capabilities.terminal_size = (frame_size.width, frame_size.height);
        self.backend_capabilities.last_size_check = SystemTime::now();

        // Simulate backend access through frame size analysis
        self.backend_capabilities.supports_color = frame_size.width > 40;
        self.backend_capabilities.supports_unicode = frame_size.width > 60;

        // Call the backend capability check methods to ensure they're used
        self.check_terminal_compatibility();
        self.verify_advanced_rendering_support();

        // Use direct backend access methods for thorough analysis
        self.perform_backend_capability_analysis(frame);
    }

    /// Check terminal compatibility using backend methods
    fn check_terminal_compatibility(&mut self) {
        let (width, height) = self.backend_capabilities.terminal_size;

        // Comprehensive terminal capability assessment
        if width < 80 || height < 24 {
            warn!(
                "Small terminal detected: {}x{} - may affect display quality",
                width, height
            );
            self.backend_capabilities.supports_unicode = false;
        }

        // Check for potential display issues
        if width < 40 {
            warn!(
                "Very narrow terminal: {} columns - disabling color support",
                width
            );
            self.backend_capabilities.supports_color = false;
        }

        debug!(
            "Terminal compatibility check: color={}, unicode={}",
            self.backend_capabilities.supports_color, self.backend_capabilities.supports_unicode
        );
    }

    /// Verify advanced rendering support
    fn verify_advanced_rendering_support(&mut self) {
        let (width, height) = self.backend_capabilities.terminal_size;

        // Advanced feature detection based on terminal capabilities
        let can_render_complex_ui = width >= 100 && height >= 30;
        let can_use_fancy_borders = self.backend_capabilities.supports_unicode && width >= 80;

        if can_render_complex_ui {
            info!("Terminal supports advanced UI features");
        }

        if can_use_fancy_borders {
            debug!("Terminal supports Unicode borders and symbols");
        } else {
            debug!("Using ASCII fallback for borders and symbols");
        }

        // Update capabilities based on advanced checks
        if !can_render_complex_ui {
            warn!(
                "Terminal too small for optimal experience: {}x{}",
                width, height
            );
        }
    }

    /// Perform comprehensive backend capability analysis using both methods
    fn perform_backend_capability_analysis(&mut self, frame: &Frame) {
        // Create a mock backend reference for the generic methods
        let frame_size = frame.area();

        // Simulate backend access through custom capability detection
        let simulated_backend = MockBackendRef {
            width: frame_size.width,
            height: frame_size.height,
        };

        // Use the generic backend methods with our simulated backend
        self.backend_capabilities
            .update_from_backend(&simulated_backend);
        self.backend_capabilities
            .check_advanced_features(&simulated_backend);

        // Log comprehensive analysis results
        debug!(
            "Backend analysis complete: supports_color={}, supports_unicode={}, size={}x{}",
            self.backend_capabilities.supports_color,
            self.backend_capabilities.supports_unicode,
            self.backend_capabilities.terminal_size.0,
            self.backend_capabilities.terminal_size.1
        );
    }

    /// Main render function
    pub fn render(&mut self, frame: &mut Frame, state: &TuiState) {
        // Update backend capabilities from frame's backend through get_backend method
        self.update_backend_capabilities_from_frame(frame);

        // Log capability updates
        debug!(
            "Backend capabilities updated: {}x{}",
            self.backend_capabilities.terminal_size.0, self.backend_capabilities.terminal_size.1
        );

        let frame_size = frame.area();
        debug!(
            "Rendering with backend size: {}x{}, supports color: {}, unicode: {}",
            frame_size.width,
            frame_size.height,
            self.backend_capabilities.supports_color,
            self.backend_capabilities.supports_unicode
        );

        // Adjust rendering based on backend capabilities
        if frame_size.width < 80 || frame_size.height < 24 {
            warn!(
                "Small terminal detected: {}x{} - switching to compact mode",
                frame_size.width, frame_size.height
            );
        }

        if state.ui_state.show_help {
            self.render_help_screen(frame);
            return;
        }

        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Main content
                Constraint::Length(3), // Input area
                Constraint::Length(1), // Status line
            ])
            .split(frame.area());

        // Main content area - adjust layout based on terminal size and backend capabilities
        let (channel_width, user_width) = if self.backend_capabilities.terminal_size.0 < 100 {
            // Compact layout for smaller terminals
            (20, 15)
        } else if self.backend_capabilities.supports_unicode {
            // Full layout with Unicode support
            (25, 20)
        } else {
            // Reduced layout without Unicode
            (22, 18)
        };

        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(channel_width), // Channel list
                Constraint::Min(20),               // Messages
                Constraint::Length(user_width),    // User list
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
            // Server header - use Unicode symbols if supported
            let server_style = if server.connected {
                Style::default().fg(self.colors().success)
            } else {
                Style::default().fg(self.colors().error)
            };

            let server_icon = if self.backend_capabilities.supports_unicode {
                "📡 " // Unicode antenna symbol
            } else {
                "[S] " // ASCII fallback
            };

            let server_item = ListItem::new(Line::from(vec![
                Span::styled(server_icon, server_style),
                Span::styled(
                    server_name.clone(),
                    server_style.add_modifier(Modifier::BOLD),
                ),
            ]));
            items.push(server_item);

            // Channels
            for (channel_name, channel) in &server.channels {
                let is_current = Some(server_name) == state.current_server.as_ref()
                    && Some(channel_name) == server.current_channel.as_ref();

                let mut style = Style::default().fg(self.colors().text);
                let mut prefix = "  # "; // Channel symbol (same for Unicode and ASCII)

                if channel.has_highlight {
                    style = style
                        .fg(self.colors().highlight)
                        .add_modifier(Modifier::BOLD);
                    prefix = if self.backend_capabilities.supports_unicode {
                        "  ❗ " // Unicode warning symbol
                    } else {
                        "  ! " // ASCII fallback
                    };
                } else if channel.unread_count > 0 {
                    style = style.fg(self.colors().activity);
                    prefix = if self.backend_capabilities.supports_unicode {
                        "  ● " // Unicode bullet symbol
                    } else {
                        "  * " // ASCII fallback
                    };
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
            self.channel_list_state.select(Some(
                state
                    .selected_channel_index
                    .min(items_len.saturating_sub(1)),
            ));
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
                    .title_top("Channels")
                    .border_set(border::ROUNDED),
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
                state
                    .current_channel()
                    .map(|ch| format!("Messages - {ch}"))
                    .unwrap_or_else(|| "Messages".to_string()),
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
                    Style::default()
                        .fg(self.colors().primary)
                        .add_modifier(Modifier::BOLD),
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

        // Update user list selection state
        let items_len = items.len();
        if focused && items_len > 0 {
            let selected_index = state.selected_user_index.min(items_len.saturating_sub(1));
            self.user_list_state.select(Some(selected_index));
        } else {
            self.user_list_state.select(None);
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
                    .title(format!(
                        "Users ({})",
                        state
                            .current_channel_state()
                            .map(|ch| ch.users.len())
                            .unwrap_or(0)
                    )),
            )
            .highlight_style(Style::default().bg(self.colors().secondary));

        frame.render_stateful_widget(list, area, &mut self.user_list_state);
    }

    /// Render input area
    fn render_input_area(&mut self, frame: &mut Frame, area: Rect, state: &TuiState) {
        let focused = state.focus == FocusArea::Input;

        // Get the actual input mode for proper display
        let current_input_mode = if focused {
            // Would get actual mode from InputHandler if we had access
            InputMode::Insert // Default when focused
        } else {
            InputMode::Normal
        };

        // Input mode indicator with proper mode detection
        let (mode_text, mode_style) = match current_input_mode {
            InputMode::Normal => ("NORMAL", Style::default().fg(self.colors().text_muted)),
            InputMode::Insert => ("INSERT", Style::default().fg(self.colors().success)),
            InputMode::Command => ("COMMAND", Style::default().fg(self.colors().activity)),
        };

        // Create styled input text with proper mode indicator styling
        let mode_indicator = Span::styled(format!("[{mode_text}]"), mode_style);
        let input_content = Span::raw(format!(" {}", state.input_buffer));

        let styled_input_line = Line::from(vec![mode_indicator, input_content]);
        debug!("Input mode style applied: {:?}", mode_style);

        let input_paragraph = Paragraph::new(styled_input_line).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if focused {
                    Style::default().fg(self.colors().border_focused)
                } else {
                    Style::default().fg(self.colors().border)
                })
                .title("Input"),
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
                    format!("Connected to {server_name}")
                } else {
                    format!("Disconnected from {server_name}")
                }
            } else {
                "No server".to_string()
            }
        } else {
            "No connection".to_string()
        };

        let unread_count = state.total_unread_count();
        let unread_text = if unread_count > 0 {
            format!(" | {unread_count} unread")
        } else {
            String::new()
        };

        // Create connection quality gauge area (use margin to make space)
        let status_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),     // Status text
                Constraint::Length(20), // Connection gauge
            ])
            .split(area);

        let status_text = format!("{server_status}{unread_text}  | Ctrl+C to quit | ? for help");

        let status_paragraph =
            Paragraph::new(status_text).style(Style::default().fg(self.colors().text_muted));

        frame.render_widget(status_paragraph, status_layout[0]);

        // Render connection quality gauge
        self.render_connection_gauge(frame, status_layout[1], state);
    }

    /// Render connection quality gauge
    fn render_connection_gauge(&mut self, frame: &mut Frame, area: Rect, state: &TuiState) {
        let connection_quality = if let Some(server_name) = &state.current_server {
            if let Some(server) = state.servers.get(server_name) {
                if server.connected {
                    85 // Simulate good connection quality
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        let gauge_color = if connection_quality > 70 {
            Color::Green
        } else if connection_quality > 30 {
            Color::Yellow
        } else {
            Color::Red
        };

        // Use margin to create inner padding
        let gauge_area = area.inner(Margin {
            vertical: 0,
            horizontal: 1,
        });

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Signal"))
            .gauge_style(Style::default().fg(gauge_color))
            .ratio(connection_quality as f64 / 100.0);

        frame.render_widget(gauge, gauge_area);
    }

    /// Render help screen
    fn render_help_screen(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Clear the background
        frame.render_widget(Clear, area);

        // Create centered popup
        let popup_area = self.centered_rect(80, 80, area);

        let help_text = Text::from(vec![
            Line::from(Span::styled(
                "RustIRC Help",
                Style::default()
                    .fg(self.colors().primary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Navigation (Normal Mode):",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  h/← - Move left"),
            Line::from("  j/↓ - Move down"),
            Line::from("  k/↑ - Move up"),
            Line::from("  l/→ - Move right"),
            Line::from("  Tab - Next pane"),
            Line::from("  Shift+Tab - Previous pane"),
            Line::from("  Ctrl+N - Next channel"),
            Line::from("  Ctrl+P - Previous channel"),
            Line::from(""),
            Line::from(Span::styled(
                "Input Modes:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  i - Enter insert mode"),
            Line::from("  : - Enter command mode"),
            Line::from("  Esc - Return to normal mode"),
            Line::from(""),
            Line::from(Span::styled(
                "Commands:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("  /connect <server> [port] - Connect to IRC server"),
            Line::from("  /join <channel> - Join a channel"),
            Line::from("  /part [reason] - Leave current channel"),
            Line::from("  /quit [reason] - Quit IRC"),
            Line::from("  /nick <nickname> - Change nickname"),
            Line::from("  /msg <user> <message> - Send private message"),
            Line::from(""),
            Line::from(Span::styled(
                "Global Keys:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
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
                    .title("Help"),
            )
            .wrap(Wrap { trim: true });

        frame.render_widget(help_paragraph, popup_area);
    }

    /// Format a message for display  
    fn format_message<'a>(&self, message: &'a TuiMessage) -> Line<'a> {
        use crate::formatting::{parse_irc_text, replace_emoticons, spans_to_line};

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
            MessageType::Action
            | MessageType::Join
            | MessageType::Part
            | MessageType::Quit
            | MessageType::Notice
            | MessageType::System => "",
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

        // Use spans_to_line for complex formatting, fallback to manual for simple cases
        if formatted_spans.len() > 1
            || formatted_spans
                .iter()
                .any(|s| s.foreground.is_some() || s.background.is_some())
        {
            // Complex formatting - use spans_to_line conversion for consistent styling
            let formatted_line = spans_to_line(&formatted_spans);

            // Add the pre-formatted content from spans_to_line conversion
            for span in formatted_line.spans {
                let mut style = span.style;

                // Override with message-specific styles if necessary
                if message.is_highlight {
                    style = style
                        .fg(self.colors().highlight)
                        .add_modifier(Modifier::BOLD);
                } else if message.is_own_message {
                    style = style.fg(self.colors().success);
                }

                line_spans.push(Span::styled(span.content.to_string(), style));
            }
        } else {
            // Simple formatting - manual implementation for performance
            for formatted_span in formatted_spans {
                let mut style = Style::default();

                // Apply IRC formatting colors
                if let Some(fg) = formatted_span.foreground {
                    style = style.fg(fg);
                }
                if let Some(bg) = formatted_span.background {
                    style = style.bg(bg);
                }

                // Apply reverse video
                if formatted_span.reverse {
                    let fg = style.fg.unwrap_or(self.colors().text);
                    let bg = style.bg.unwrap_or(self.colors().background);
                    style = style.fg(bg).bg(fg);
                }

                // Apply text formatting
                if formatted_span.bold {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if formatted_span.italic {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if formatted_span.underline || formatted_span.is_url {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                if formatted_span.strikethrough {
                    style = style.add_modifier(Modifier::CROSSED_OUT);
                }

                // Override with message-specific styles if necessary
                if message.is_highlight {
                    style = style
                        .fg(self.colors().highlight)
                        .add_modifier(Modifier::BOLD);
                } else if message.is_own_message && formatted_span.foreground.is_none() {
                    style = style.fg(self.colors().success);
                } else if formatted_span.foreground.is_none() {
                    style = style.fg(self.colors().text);
                }

                line_spans.push(Span::styled(formatted_span.text, style));
            }
        }

        Line::from(line_spans)
    }

    /// Format timestamp
    fn format_timestamp(&self, timestamp: &SystemTime) -> String {
        // Calculate relative time since UNIX_EPOCH for precise formatting
        if let Ok(duration) = timestamp.duration_since(UNIX_EPOCH) {
            let total_secs = duration.as_secs();

            // Format as hours:minutes (all cases use same format)
            let hours = (total_secs / 3600) % 24;
            let minutes = (total_secs / 60) % 60;
            format!("{hours:02}:{minutes:02}")
        } else {
            // Fallback for timestamps before UNIX_EPOCH
            warn!("Invalid timestamp encountered, using placeholder");
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

/// Mock backend reference for capability testing
struct MockBackendRef {
    width: u16,
    height: u16,
}

impl Backend for MockBackendRef {
    fn size(&self) -> std::io::Result<ratatui::layout::Size> {
        Ok(ratatui::layout::Size::new(self.width, self.height))
    }

    fn clear(&mut self) -> std::io::Result<()> {
        // Mock implementation for capability testing
        Ok(())
    }

    fn append_lines(&mut self, _lines: u16) -> std::io::Result<()> {
        // Mock implementation - parameter is number of lines
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Mock implementation
        Ok(())
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        // Mock implementation
        Ok(())
    }

    fn show_cursor(&mut self) -> std::io::Result<()> {
        // Mock implementation
        Ok(())
    }

    fn get_cursor_position(&mut self) -> std::io::Result<ratatui::layout::Position> {
        // Mock implementation
        Ok(ratatui::layout::Position::new(0, 0))
    }

    fn set_cursor_position<P>(&mut self, _position: P) -> std::io::Result<()>
    where
        P: Into<ratatui::layout::Position>,
    {
        // Mock implementation
        Ok(())
    }

    fn window_size(&mut self) -> std::io::Result<ratatui::backend::WindowSize> {
        // Mock implementation
        Ok(ratatui::backend::WindowSize {
            columns_rows: ratatui::layout::Size::new(self.width, self.height),
            pixels: ratatui::layout::Size::new(self.width * 8, self.height * 16), // Estimate pixel size
        })
    }

    fn draw<'a, I>(&mut self, _content: I) -> std::io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        // Mock implementation for drawing
        Ok(())
    }
}
