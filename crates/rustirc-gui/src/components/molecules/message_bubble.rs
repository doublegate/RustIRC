//! Modern Message Bubble Component
//!
//! This module implements a modern chat message bubble with:
//! - Material Design 3 styling
//! - Rich text content with IRC formatting
//! - User avatars and status indicators
//! - Reaction system
//! - Proper accessibility

use crate::{
    components::atoms::{
        button::{ButtonSize, ButtonVariant, MaterialButton},
        typography::{MaterialText, RichText, TextSpan, TypographyVariant},
    },
    themes::material_design_3::MaterialTheme,
};
use iced::{
    alignment::{Horizontal, Vertical},
    time::Instant,
    widget::{column, container, mouse_area, row, stack, text, tooltip},
    Background, Border, Color, Element, Length,
};
use rustirc_protocol::Message as IrcMessage;
use std::collections::HashMap;

/// Message bubble configuration
#[derive(Debug, Clone)]
pub struct MessageBubble {
    pub message: ChatMessage,
    pub theme: MaterialTheme,
    pub show_avatar: bool,
    pub grouped: bool, // True if this message is grouped with previous from same user
    pub show_timestamp: bool,
    pub compact_mode: bool,
    pub highlight: bool, // True if message mentions current user
    pub selected: bool,
}

/// Chat message data
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub timestamp: Instant,
    pub sender: MessageSender,
    pub content: MessageContent,
    pub reactions: HashMap<String, ReactionData>, // emoji -> ReactionData
    pub thread_count: Option<u32>,
    pub edited: bool,
    pub message_type: MessageType,
}

/// Message sender information
#[derive(Debug, Clone)]
pub struct MessageSender {
    pub nickname: String,
    pub user_id: String,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub color: Option<Color>,
    pub badges: Vec<UserBadge>, // Channel operator, voice, etc.
}

/// User status indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    Online,
    Away,
    Offline,
}

/// User badge (op, voice, etc.)
#[derive(Debug, Clone)]
pub struct UserBadge {
    pub badge_type: BadgeType,
    pub icon: String,
    pub color: Color,
    pub tooltip: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeType {
    Owner,        // ~
    Admin,        // &
    Operator,     // @
    HalfOperator, // %
    Voice,        // +
    Custom(u8),
}

/// Message content with rich formatting
#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(RichTextContent),
    Action(String), // /me action
    System(String), // Server messages (joins, parts, etc.)
    Notice(String),
    Ctcp(String),
}

/// Rich text content with IRC formatting
#[derive(Debug, Clone)]
pub struct RichTextContent {
    pub spans: Vec<FormattedSpan>,
    pub mentions: Vec<Mention>,
    pub links: Vec<Link>,
    pub emojis: Vec<Emoji>,
}

/// Formatted text span with IRC attributes
#[derive(Debug, Clone)]
pub struct FormattedSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub monospace: bool,
}

/// User mention
#[derive(Debug, Clone)]
pub struct Mention {
    pub text: String,
    pub user_id: String,
    pub start_index: usize,
    pub end_index: usize,
}

/// URL link
#[derive(Debug, Clone)]
pub struct Link {
    pub url: String,
    pub display_text: String,
    pub start_index: usize,
    pub end_index: usize,
    pub preview: Option<LinkPreview>,
}

/// Link preview data
#[derive(Debug, Clone)]
pub struct LinkPreview {
    pub title: String,
    pub description: String,
    pub image_url: Option<String>,
    pub site_name: String,
}

/// Emoji with metadata
#[derive(Debug, Clone)]
pub struct Emoji {
    pub name: String,
    pub unicode: String,
    pub start_index: usize,
    pub end_index: usize,
}

/// Reaction data
#[derive(Debug, Clone)]
pub struct ReactionData {
    pub emoji: String,
    pub count: u32,
    pub users: Vec<String>, // User IDs who reacted
    pub self_reacted: bool, // True if current user reacted
}

/// Message type for different styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Normal,
    System,
    Error,
    Notice,
    Action,
    Highlight, // Message mentions current user
}

/// Message bubble actions
#[derive(Debug, Clone)]
pub enum MessageAction {
    Reply(String),                  // message_id
    Edit(String),                   // message_id
    Delete(String),                 // message_id
    React(String, String),          // message_id, emoji
    RemoveReaction(String, String), // message_id, emoji
    CopyText(String),               // text
    CopyLink(String),               // url
    SelectMessage(String),          // message_id
    ShowThread(String),             // message_id
    ShowUserCard(String),           // user_id
}

impl MessageBubble {
    /// Create new message bubble
    pub fn new(message: ChatMessage) -> Self {
        Self {
            message,
            theme: MaterialTheme::dark(),
            show_avatar: true,
            grouped: false,
            show_timestamp: true,
            compact_mode: false,
            highlight: false,
            selected: false,
        }
    }

    /// Set theme
    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set grouped mode (no avatar, condensed spacing)
    pub fn grouped(mut self, grouped: bool) -> Self {
        self.grouped = grouped;
        self
    }

    /// Set compact mode
    pub fn compact_mode(mut self, compact: bool) -> Self {
        self.compact_mode = compact;
        self
    }

    /// Set highlight mode (for mentions)
    pub fn highlight(mut self, highlight: bool) -> Self {
        self.highlight = highlight;
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Build message bubble element
    pub fn build<Message>(self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
        MessageAction: Into<Message>,
    {
        let spacing = if self.compact_mode {
            self.theme.spacing.xs
        } else {
            self.theme.spacing.sm
        };

        let content = if self.grouped && self.message.message_type == MessageType::Normal {
            self.build_grouped_content()
        } else {
            self.build_full_content()
        };

        let background_color = self.get_background_color();
        let border_color = if self.selected {
            iced::Color::from(self.theme.scheme.primary)
        } else {
            Color::TRANSPARENT
        };

        let bubble = container(content)
            .padding([spacing, self.theme.spacing.md])
            .width(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(Background::Color(background_color)),
                border: Border {
                    width: if self.selected { 2.0 } else { 0.0 },
                    color: border_color,
                    radius: self.theme.shapes.corner_small.into(),
                },
                text_color: Some(iced::Color::from(self.theme.scheme.on_surface)),
                shadow: if self.message.message_type == MessageType::Highlight {
                    iced::Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
                        offset: iced::Vector::new(0.0, 2.0),
                        blur_radius: 4.0,
                    }
                } else {
                    Default::default()
                },
            });

        // Add hover and click interactions
        mouse_area(bubble)
            .on_press(MessageAction::SelectMessage(self.message.id.clone()).into())
            .into()
    }

    /// Build full message content (with avatar and metadata)  
    fn build_full_content<Message>(&self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
        MessageAction: Into<Message>,
    {
        let avatar_size = if self.compact_mode { 24.0 } else { 32.0 };

        // Build avatar element
        let avatar_element = if self.show_avatar {
            self.build_avatar(avatar_size)
        } else {
            container(text(""))
                .width(Length::Fixed(avatar_size + self.theme.spacing.sm))
                .into()
        };

        row![
            // Avatar column
            avatar_element,
            // Content column
            column![
                // Header (username, timestamp, badges)
                self.build_message_header(),
                // Message content
                self.build_message_content(),
                // Reactions
                if !self.message.reactions.is_empty() {
                    self.build_reactions()
                } else {
                    Element::from(text(""))
                }
            ]
            .spacing(self.theme.spacing.xs)
            .width(Length::Fill)
        ]
        .spacing(self.theme.spacing.sm)
        .align_y(Vertical::Top)
        .into()
    }

    /// Build grouped message content (no avatar, minimal spacing)
    fn build_grouped_content<Message>(&self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
        MessageAction: Into<Message>,
    {
        let avatar_size = if self.compact_mode { 24.0 } else { 32.0 };

        row![
            // Spacer for alignment with non-grouped messages
            container(text("")).width(Length::Fixed(avatar_size + self.theme.spacing.sm)),
            // Message content only
            column![
                self.build_message_content(),
                // Reactions
                if !self.message.reactions.is_empty() {
                    self.build_reactions()
                } else {
                    Element::from(text(""))
                }
            ]
            .spacing(self.theme.spacing.xs)
            .width(Length::Fill)
        ]
        .spacing(self.theme.spacing.sm)
        .align_y(Vertical::Top)
        .into()
    }

    /// Build user avatar with status indicator
    fn build_avatar<Message>(&self, size: f32) -> Element<'static, Message>
    where
        Message: Clone + 'static,
        MessageAction: Into<Message>,
    {
        // Avatar background (placeholder for now - would show actual image)
        let avatar_color = self
            .message
            .sender
            .color
            .unwrap_or_else(|| self.get_user_color(&self.message.sender.nickname));

        let avatar_initial = self
            .message
            .sender
            .nickname
            .chars()
            .next()
            .unwrap_or('?')
            .to_string();
        let avatar_content = container(
            MaterialText::new(&avatar_initial)
                .variant(TypographyVariant::LabelLarge)
                .color(Color::WHITE)
                .theme(self.theme.clone())
                .build(),
        )
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(Background::Color(avatar_color)),
            border: Border {
                width: 0.0,
                color: Color::TRANSPARENT,
                radius: (size / 2.0).into(),
            },
            ..Default::default()
        });

        // Add status indicator
        let status_color = match self.message.sender.status {
            UserStatus::Online => self.theme.scheme.connection_good,
            UserStatus::Away => self.theme.scheme.connection_poor,
            UserStatus::Offline => self.theme.scheme.outline_variant,
        };

        // Clone surface color before closure to avoid lifetime issues
        let surface_color = iced::Color::from(self.theme.scheme.surface);
        let status_indicator = container(text(""))
            .width(Length::Fixed(8.0))
            .height(Length::Fixed(8.0))
            .style(move |_theme| container::Style {
                background: Some(Background::Color(status_color.into())),
                border: Border {
                    width: 2.0,
                    color: surface_color,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        // Stack avatar with status indicator
        let avatar_with_status = stack![
            avatar_content,
            container(status_indicator)
                .align_x(Horizontal::Right)
                .align_y(Vertical::Bottom)
        ];

        // Make avatar clickable for user card
        let user_id = self.message.sender.user_id.clone();
        mouse_area(avatar_with_status)
            .on_press(MessageAction::ShowUserCard(user_id).into())
            .into()
    }

    /// Build message header (username, badges, timestamp)
    fn build_message_header<Message>(&self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
    {
        let mut header_elements = Vec::new();

        // Username
        let username_color = self
            .message
            .sender
            .color
            .unwrap_or_else(|| self.get_user_color(&self.message.sender.nickname));
        let nickname = self.message.sender.nickname.clone();

        header_elements.push(
            MaterialText::new(&nickname)
                .variant(TypographyVariant::LabelMedium)
                .color(username_color)
                .theme(self.theme.clone())
                .build(),
        );

        // User badges (op, voice, etc.)
        for badge in &self.message.sender.badges {
            let badge_icon = badge.icon.clone();
            let badge_tooltip = badge.tooltip.clone();
            header_elements.push(
                tooltip(
                    MaterialText::new(&badge_icon)
                        .variant(TypographyVariant::LabelSmall)
                        .color(badge.color)
                        .theme(self.theme.clone())
                        .build(),
                    text(badge_tooltip),
                    tooltip::Position::Top,
                )
                .into(),
            );
        }

        // Timestamp
        if self.show_timestamp {
            let timestamp_text = format_timestamp(self.message.timestamp);
            header_elements.push(
                MaterialText::new(timestamp_text)
                    .variant(TypographyVariant::BodySmall)
                    .color(iced::Color::from(self.theme.scheme.on_surface_variant))
                    .theme(self.theme.clone())
                    .build(),
            );
        }

        // Edited indicator
        if self.message.edited {
            header_elements.push(
                MaterialText::new("(edited)")
                    .variant(TypographyVariant::BodySmall)
                    .color(iced::Color::from(self.theme.scheme.on_surface_variant))
                    .theme(self.theme.clone())
                    .build(),
            );
        }

        let spacing = self.theme.spacing.xs;
        row(header_elements)
            .spacing(spacing)
            .align_y(Vertical::Center)
            .into()
    }

    /// Build message content based on type
    fn build_message_content<Message>(&self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
        MessageAction: Into<Message>,
    {
        match &self.message.content {
            MessageContent::Text(rich_content) => self.build_rich_text_content(rich_content),
            MessageContent::Action(action) => {
                MaterialText::new(format!("* {} {}", self.message.sender.nickname, action))
                    .variant(TypographyVariant::BodyMedium)
                    .color(iced::Color::from(self.theme.scheme.on_surface_variant))
                    .theme(self.theme.clone())
                    .build()
            }
            MessageContent::System(system) => MaterialText::new(system)
                .variant(TypographyVariant::BodySmall)
                .color(iced::Color::from(self.theme.scheme.on_surface_variant))
                .theme(self.theme.clone())
                .build(),
            MessageContent::Notice(notice) => MaterialText::new(format!("Notice: {notice}"))
                .variant(TypographyVariant::BodyMedium)
                .color(iced::Color::from(self.theme.scheme.tertiary))
                .theme(self.theme.clone())
                .build(),
            MessageContent::Ctcp(ctcp) => MaterialText::new(format!("[CTCP] {ctcp}"))
                .variant(TypographyVariant::BodySmall)
                .color(iced::Color::from(self.theme.scheme.on_surface_variant))
                .theme(self.theme.clone())
                .build(),
        }
    }

    /// Build rich text content with IRC formatting
    fn build_rich_text_content<Message>(
        &self,
        content: &RichTextContent,
    ) -> Element<'static, Message>
    where
        Message: Clone + 'static,
        MessageAction: Into<Message>,
    {
        let mut rich_spans = Vec::new();

        for span in &content.spans {
            let mut text_span = TextSpan::new(&span.text);

            if span.bold {
                text_span = text_span.bold();
            }
            if span.italic {
                text_span = text_span.italic();
            }
            if span.underline {
                text_span = text_span.underline();
            }
            if span.strikethrough {
                text_span = text_span.strikethrough();
            }
            if span.monospace {
                text_span = text_span.code();
            }
            if let Some(color) = span.color {
                text_span = text_span.color(color);
            }

            rich_spans.push(text_span);
        }

        // Build rich text with proper mentions and links highlighting
        let mut rich_text = RichText::new().theme(self.theme.clone()).selectable(true);

        for span in rich_spans {
            rich_text = rich_text.span(span);
        }

        // TODO: Add link previews below text if present
        let text_element = rich_text.build();

        if content.links.iter().any(|link| link.preview.is_some()) {
            column![text_element, self.build_link_previews(&content.links)]
                .spacing(self.theme.spacing.sm)
                .into()
        } else {
            text_element
        }
    }

    /// Build link previews
    fn build_link_previews<Message>(&self, links: &[Link]) -> Element<'static, Message>
    where
        Message: Clone + 'static,
    {
        let mut previews = Vec::new();

        for link in links {
            if let Some(preview) = &link.preview {
                let preview_content = container(
                    column![
                        MaterialText::new(&preview.title)
                            .variant(TypographyVariant::TitleSmall)
                            .theme(self.theme.clone())
                            .build(),
                        MaterialText::new(&preview.description)
                            .variant(TypographyVariant::BodySmall)
                            .color(iced::Color::from(self.theme.scheme.on_surface_variant))
                            .theme(self.theme.clone())
                            .build(),
                        MaterialText::new(&preview.site_name)
                            .variant(TypographyVariant::LabelSmall)
                            .color(iced::Color::from(self.theme.scheme.primary))
                            .theme(self.theme.clone())
                            .build()
                    ]
                    .spacing(self.theme.spacing.xs),
                )
                .padding(self.theme.spacing.sm)
                .width(Length::Fill);

                previews.push(preview_content.into());
            }
        }

        column(previews).spacing(self.theme.spacing.xs).into()
    }

    /// Build message reactions
    fn build_reactions<Message>(&self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
        MessageAction: Into<Message>,
    {
        let reaction_buttons: Vec<Element<_>> = self
            .message
            .reactions
            .iter()
            .map(|(emoji, data)| {
                let _bg_color = if data.self_reacted {
                    self.theme.scheme.primary_container
                } else {
                    self.theme.scheme.surface_container
                };

                let _text_color = if data.self_reacted {
                    self.theme.scheme.on_primary_container
                } else {
                    self.theme.scheme.on_surface_variant
                };

                let reaction_text = if data.count > 1 {
                    format!("{} {}", emoji, data.count)
                } else {
                    emoji.clone()
                };

                let action = if data.self_reacted {
                    MessageAction::RemoveReaction(self.message.id.clone(), emoji.clone())
                } else {
                    MessageAction::React(self.message.id.clone(), emoji.clone())
                };

                MaterialButton::new(reaction_text)
                    .variant(ButtonVariant::Text)
                    .size(ButtonSize::Small)
                    .theme(self.theme.clone())
                    .on_press(action.into())
                    .build()
            })
            .collect();

        row(reaction_buttons).spacing(self.theme.spacing.xs).into()
    }

    /// Get background color based on message type and state
    fn get_background_color(&self) -> Color {
        match (self.highlight, self.selected, self.message.message_type) {
            (true, _, _) => iced::Color::from(self.theme.scheme.tertiary_container),
            (_, true, _) => iced::Color::from(self.theme.scheme.primary_container),
            (_, _, MessageType::System) => {
                iced::Color::from(self.theme.scheme.surface_container_low)
            }
            (_, _, MessageType::Error) => iced::Color::from(self.theme.scheme.error_container),
            (_, _, MessageType::Notice) => iced::Color::from(self.theme.scheme.secondary_container),
            _ => Color::TRANSPARENT,
        }
    }

    /// Get consistent color for user nickname
    fn get_user_color(&self, nickname: &str) -> Color {
        let colors = &self.theme.scheme.nick_colors;
        let index = nickname
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_add(b as u32)) as usize
            % colors.len();
        colors[index].into()
    }
}

/// Format timestamp for display
fn format_timestamp(timestamp: Instant) -> String {
    // This is a simplified implementation
    // In a real implementation, you would format based on user preferences
    format!(
        "{:02}:{:02}",
        timestamp.elapsed().as_secs() / 3600 % 24,
        timestamp.elapsed().as_secs() / 60 % 60
    )
}

/// Convert IRC message to chat message
pub fn irc_to_chat_message(_irc_msg: &IrcMessage, _theme: &MaterialTheme) -> Option<ChatMessage> {
    // This would parse IRC message into structured chat message
    // Implementation depends on the IRC message format
    None // Placeholder
}
