//! Message view widget for RustIRC GUI
//!
//! Displays IRC messages with formatting, timestamps, and scrolling.
//! Features message rendering, auto-scroll, search, and selection.

use crate::state::{AppState, DisplayMessage, MessageType, FormattedText};
use crate::theme::Theme;
use crate::formatting::{parse_irc_text, spans_to_elements, replace_emoticons};
use iced::{
    widget::{container, scrollable, text, column, row, Space},
    Element, Length, Task, Color, Alignment, Background,
    font::{Weight, Style as FontStyle},
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Messages for message view interactions
#[derive(Debug, Clone)]
pub enum MessageViewMessage {
    ScrollToBottom,
    ScrollToTop,
    MessageClicked(usize),
    MessageSelected(usize),
    SearchRequested(String),
    ClearSelection,
    CopySelected,
    UrlClicked(String),
}

/// Message view widget state
#[derive(Debug, Clone)]
pub struct MessageView {
    auto_scroll: bool,
    scroll_position: f32,
    selected_messages: Vec<usize>,
    search_query: Option<String>,
    font_size: f32,
    show_timestamps: bool,
    show_joins_parts: bool,
    compact_mode: bool,
}

impl MessageView {
    pub fn new() -> Self {
        Self {
            auto_scroll: true,
            scroll_position: 0.0,
            selected_messages: Vec::new(),
            search_query: None,
            font_size: 13.0,
            show_timestamps: true,
            show_joins_parts: true,
            compact_mode: false,
        }
    }

    /// Update the message view state
    pub fn update(&mut self, message: MessageViewMessage, app_state: &mut AppState) -> Task<MessageViewMessage> {
        match message {
            MessageViewMessage::ScrollToBottom => {
                self.auto_scroll = true;
                self.scroll_position = 1.0;
                Task::none()
            }
            MessageViewMessage::ScrollToTop => {
                self.auto_scroll = false;
                self.scroll_position = 0.0;
                Task::none()
            }
            MessageViewMessage::MessageClicked(index) => {
                if !self.selected_messages.contains(&index) {
                    self.selected_messages.clear();
                    self.selected_messages.push(index);
                }
                Task::none()
            }
            MessageViewMessage::MessageSelected(index) => {
                if self.selected_messages.contains(&index) {
                    self.selected_messages.retain(|&i| i != index);
                } else {
                    self.selected_messages.push(index);
                }
                Task::none()
            }
            MessageViewMessage::SearchRequested(query) => {
                self.search_query = if query.is_empty() { None } else { Some(query) };
                Task::none()
            }
            MessageViewMessage::ClearSelection => {
                self.selected_messages.clear();
                Task::none()
            }
            MessageViewMessage::CopySelected => {
                // TODO: Copy selected messages to clipboard
                Task::none()
            }
            MessageViewMessage::UrlClicked(url) => {
                // TODO: Open URL in default browser
                Task::none()
            }
        }
    }

    /// Render the message view
    pub fn view<'a>(&self, app_state: &'a AppState) -> Element<'a, MessageViewMessage> {
        let current_tab = app_state.current_tab();
        
        if let Some(tab) = current_tab {
            let mut content = column![];
            
            for (index, message) in tab.messages.iter().enumerate() {
                // Filter messages based on settings
                if !self.should_show_message(message) {
                    continue;
                }
                
                // Check if message matches search
                if let Some(ref query) = self.search_query {
                    if !message.content.to_lowercase().contains(&query.to_lowercase()) {
                        continue;
                    }
                }
                
                let message_element = self.render_message(message, index, app_state);
                content = content.push(message_element);
            }
            
            let scrollable_content = scrollable(
                container(content)
                    .padding(8)
                    .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill);
            
            container(scrollable_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            // No active tab - show welcome message
            container(
                container(
                    column![
                        text("Welcome to RustIRC")
                            .size(24)
                            .color(Color::from_rgb(0.4, 0.6, 1.0)),
                        Space::with_height(Length::Fixed(16.0)),
                        text("Connect to a server to start chatting")
                            .size(14)
                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                    ]
                    .align_x(Alignment::Center)
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }

    /// Check if a message should be shown based on current settings
    fn should_show_message(&self, message: &DisplayMessage) -> bool {
        if !self.show_joins_parts {
            match message.message_type {
                MessageType::Join | MessageType::Part | MessageType::Quit => return false,
                _ => {}
            }
        }
        true
    }

    /// Render a single message
    fn render_message<'a>(&self, message: &'a DisplayMessage, index: usize, app_state: &AppState) -> Element<'a, MessageViewMessage> {
        let is_selected = self.selected_messages.contains(&index);
        let is_highlight = message.is_highlight;
        let is_own_message = message.is_own_message;

        // Build timestamp
        let timestamp_element: Element<MessageViewMessage> = if self.show_timestamps {
            let timestamp = format_timestamp(&message.timestamp, &app_state.settings().timestamp_format);
            text(timestamp)
                .size(self.font_size - 1.0)
                .color(Color::from_rgb(0.5, 0.5, 0.5))
                .into()
        } else {
            Space::with_width(Length::Fixed(0.0)).into()
        };

        // Build sender
        let sender_element: Element<MessageViewMessage> = if let Some(ref sender) = message.sender {
            let sender_color = if app_state.settings().nick_colors {
                get_nick_color(sender)
            } else {
                Color::from_rgb(0.7, 0.7, 0.7)
            };

            let sender_text = match message.message_type {
                MessageType::Message | MessageType::Notice => format!("<{}>", sender),
                MessageType::Action => format!("* {}", sender),
                MessageType::Join => format!("→ {}", sender),
                MessageType::Part => format!("← {}", sender),
                MessageType::Quit => format!("⚠ {}", sender),
                MessageType::Nick => format!("~ {}", sender),
                MessageType::Topic => format!("ⓘ {}", sender),
                MessageType::Mode => format!("⚙ {}", sender),
                MessageType::System => "***".to_string(),
            };

            text(sender_text)
                .size(self.font_size)
                .font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::default() })
                .color(sender_color)
                .into()
        } else {
            Space::with_width(Length::Fixed(0.0)).into()
        };

        // Build message content
        let content_element = self.render_formatted_content(&message.content);

        // Build the complete message row
        let message_row = if self.compact_mode {
            row![
                timestamp_element,
                sender_element,
                content_element
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        } else {
            row![
                column![
                    timestamp_element,
                    Space::with_height(Length::Fixed(2.0))
                ]
                .width(Length::Fixed(80.0))
                .align_x(Alignment::End),
                column![
                    sender_element,
                    Space::with_height(Length::Fixed(2.0))
                ]
                .width(Length::Fixed(120.0)),
                column![
                    content_element,
                    Space::with_height(Length::Fixed(2.0))
                ]
                .width(Length::Fill)
            ]
            .spacing(8)
            .align_y(Alignment::Start)
        };

        // Determine background color
        let background_color = if is_selected {
            Color::from_rgb(0.2, 0.4, 0.8)
        } else if is_highlight {
            Color::from_rgb(0.8, 0.2, 0.2)
        } else if is_own_message {
            Color::from_rgb(0.1, 0.3, 0.1)
        } else {
            Color::TRANSPARENT
        };

        container(message_row)
            .padding(if self.compact_mode { 2 } else { 4 })
            .width(Length::Fill)
            .into()
    }

    /// Render formatted text content with IRC formatting
    fn render_formatted_content<'a>(&self, content: &'a str) -> Element<'a, MessageViewMessage> {
        // Process emoticons first
        let content_with_emotes = replace_emoticons(content);
        
        // Parse IRC formatting
        let spans = parse_irc_text(&content_with_emotes);
        
        // Convert spans to Iced elements
        let elements = spans_to_elements(&spans, |url| MessageViewMessage::UrlClicked(url));
        
        // Arrange elements in a row with proper wrapping
        if elements.is_empty() {
            text("")
                .size(self.font_size)
                .into()
        } else {
            let mut content_row = row![];
            for element in elements {
                content_row = content_row.push(element);
            }
            content_row
                .spacing(0)
                .into()
        }
    }

    /// Set font size for messages
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    /// Toggle timestamp display
    pub fn toggle_timestamps(&mut self) {
        self.show_timestamps = !self.show_timestamps;
    }

    /// Toggle join/part message display
    pub fn toggle_joins_parts(&mut self) {
        self.show_joins_parts = !self.show_joins_parts;
    }

    /// Toggle compact mode
    pub fn toggle_compact_mode(&mut self) {
        self.compact_mode = !self.compact_mode;
    }

    /// Set auto-scroll behavior
    pub fn set_auto_scroll(&mut self, enabled: bool) {
        self.auto_scroll = enabled;
    }
}

impl Default for MessageView {
    fn default() -> Self {
        Self::new()
    }
}

/// Format timestamp for display
fn format_timestamp(timestamp: &SystemTime, format: &str) -> String {
    let duration = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    
    // Simple time formatting (hours:minutes:seconds)
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;
    
    match format {
        "%H:%M:%S" => format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
        "%H:%M" => format!("{:02}:{:02}", hours, minutes),
        _ => format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
    }
}

/// Generate a consistent color for a nickname
fn get_nick_color(nick: &str) -> Color {
    // Simple hash-based color generation
    let mut hash = 0u32;
    for byte in nick.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    
    let hue = (hash % 360) as f32;
    let saturation = 0.7;
    let lightness = 0.6;
    
    hsl_to_rgb(hue, saturation, lightness)
}

/// Convert HSL to RGB color
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let h = h / 360.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r, g, b) = if h < 1.0/6.0 {
        (c, x, 0.0)
    } else if h < 2.0/6.0 {
        (x, c, 0.0)
    } else if h < 3.0/6.0 {
        (0.0, c, x)
    } else if h < 4.0/6.0 {
        (0.0, x, c)
    } else if h < 5.0/6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    Color::from_rgb(r + m, g + m, b + m)
}

/// Convert IRC color codes to RGB
fn irc_color_to_rgb(color_code: u8) -> Color {
    match color_code % 16 {
        0 => Color::from_rgb(1.0, 1.0, 1.0),     // White
        1 => Color::from_rgb(0.0, 0.0, 0.0),     // Black
        2 => Color::from_rgb(0.0, 0.0, 0.5),     // Blue
        3 => Color::from_rgb(0.0, 0.5, 0.0),     // Green
        4 => Color::from_rgb(0.8, 0.0, 0.0),     // Red
        5 => Color::from_rgb(0.5, 0.0, 0.0),     // Brown
        6 => Color::from_rgb(0.5, 0.0, 0.5),     // Purple
        7 => Color::from_rgb(0.8, 0.5, 0.0),     // Orange
        8 => Color::from_rgb(1.0, 1.0, 0.0),     // Yellow
        9 => Color::from_rgb(0.0, 1.0, 0.0),     // Light Green
        10 => Color::from_rgb(0.0, 0.5, 0.5),    // Cyan
        11 => Color::from_rgb(0.0, 1.0, 1.0),    // Light Cyan
        12 => Color::from_rgb(0.0, 0.0, 1.0),    // Light Blue
        13 => Color::from_rgb(1.0, 0.0, 1.0),    // Pink
        14 => Color::from_rgb(0.5, 0.5, 0.5),    // Grey
        15 => Color::from_rgb(0.8, 0.8, 0.8),    // Light Grey
        _ => Color::from_rgb(0.9, 0.9, 0.9),     // Default
    }
}