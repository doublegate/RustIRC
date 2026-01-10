use iced::{
    widget::{button, column, container, row, scrollable, text, text_input},
    Background, Border, Color, Element, Length, Renderer, Theme,
};

use crate::components::atoms::button::{ButtonVariant, MaterialButton};
use crate::components::atoms::typography::{MaterialText, TypographyVariant};
use crate::themes::material_design_3::MaterialTheme;

// Rich text editor with IRC formatting support
#[derive(Debug, Clone)]
pub struct RichTextEditor {
    content: String,
    cursor_position: usize,
    selection_start: Option<usize>,
    selection_end: Option<usize>,
    formatting_stack: Vec<FormatType>,
    show_toolbar: bool,
    show_emoji_picker: bool,
    recent_emojis: Vec<String>,
    theme: MaterialTheme,
    placeholder: String,
    max_length: Option<usize>,
    is_multiline: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormatType {
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Color(u8),      // IRC color code 0-15
    Background(u8), // IRC background color
    Monospace,
    Reset,
}

#[derive(Debug, Clone)]
pub enum RichTextMessage {
    ContentChanged(String),
    CursorMoved(usize),
    SelectionChanged(usize, usize),
    FormatToggled(FormatType),
    ColorSelected(u8),
    BackgroundColorSelected(u8),
    EmojiSelected(String),
    ToggleToolbar,
    ToggleEmojiPicker,
    InsertText(String),
    PasteText(String),
    Undo,
    Redo,
    Clear,
    Submit,
}

// IRC color palette
#[allow(dead_code)]
const IRC_COLORS: [(u8, Color); 16] = [
    (0, Color::WHITE),                       // White
    (1, Color::BLACK),                       // Black
    (2, Color::from_rgb(0.0, 0.0, 0.5)),     // Navy
    (3, Color::from_rgb(0.0, 0.5, 0.0)),     // Green
    (4, Color::from_rgb(1.0, 0.0, 0.0)),     // Red
    (5, Color::from_rgb(0.5, 0.0, 0.0)),     // Maroon
    (6, Color::from_rgb(0.5, 0.0, 0.5)),     // Purple
    (7, Color::from_rgb(1.0, 0.5, 0.0)),     // Orange
    (8, Color::from_rgb(1.0, 1.0, 0.0)),     // Yellow
    (9, Color::from_rgb(0.0, 1.0, 0.0)),     // Lime
    (10, Color::from_rgb(0.0, 0.5, 0.5)),    // Teal
    (11, Color::from_rgb(0.0, 1.0, 1.0)),    // Cyan
    (12, Color::from_rgb(0.0, 0.0, 1.0)),    // Blue
    (13, Color::from_rgb(1.0, 0.0, 1.0)),    // Magenta
    (14, Color::from_rgb(0.5, 0.5, 0.5)),    // Gray
    (15, Color::from_rgb(0.75, 0.75, 0.75)), // Light Gray
];

// Common emojis for IRC
const COMMON_EMOJIS: &[&str] = &[
    "😀", "😃", "😄", "😁", "😆", "😅", "🤣", "😂", "🙂", "🙃", "😉", "😊", "😇", "🥰", "😍", "🤩",
    "😘", "😗", "😚", "😙", "😋", "😛", "😜", "🤪", "😝", "🤑", "🤗", "🤭", "🤫", "🤔", "🤐", "🤨",
    "😐", "😑", "😶", "😏", "😒", "🙄", "😬", "🤥", "😔", "😕", "🙁", "☹️", "😣", "😖", "😫", "😩",
    "🥺", "😢", "😭", "😤", "😠", "😡", "🤬", "🤯", "😳", "🥵", "🥶", "😱", "😨", "😰", "😥", "😓",
    "🤗", "🤔", "😴", "😪", "😵", "🤐", "🥴", "🤢", "🤮", "🤧", "😷", "🤒", "🤕", "🤑", "🤠", "😎",
    "🤓", "🧐", "😕", "😟", "🙁", "☹️", "😮", "😯", "😲", "😳", "🥱", "😴", "🤤", "😪", "😵", "🤐",
    "🥴", "🤢", "🤮", "🤧", "👍", "👎", "👌", "🤌", "🤏", "✌️", "🤞", "🤟", "🤘", "🤙", "👈", "👉",
    "👆", "🖕", "👇", "☝️", "👋", "🤚", "🖐", "✋", "🖖", "👏", "🙌", "🤝", "🙏", "✍️", "💪", "🦾",
    "🦿", "🦵",
];

impl RichTextEditor {
    pub fn new(theme: MaterialTheme) -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            selection_start: None,
            selection_end: None,
            formatting_stack: Vec::new(),
            show_toolbar: true,
            show_emoji_picker: false,
            recent_emojis: vec!["😀".to_string(), "👍".to_string(), "❤️".to_string()],
            theme,
            placeholder: "Type a message...".to_string(),
            max_length: Some(512), // Standard IRC message limit
            is_multiline: false,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn max_length(mut self, max_length: Option<usize>) -> Self {
        self.max_length = max_length;
        self
    }

    pub fn multiline(mut self, multiline: bool) -> Self {
        self.is_multiline = multiline;
        self
    }

    pub fn update(&mut self, message: RichTextMessage) {
        match message {
            RichTextMessage::ContentChanged(content) => {
                if let Some(max_len) = self.max_length {
                    if content.len() <= max_len {
                        self.content = content;
                    }
                } else {
                    self.content = content;
                }
            }
            RichTextMessage::CursorMoved(position) => {
                self.cursor_position = position.min(self.content.len());
                self.selection_start = None;
                self.selection_end = None;
            }
            RichTextMessage::SelectionChanged(start, end) => {
                self.selection_start = Some(start);
                self.selection_end = Some(end);
            }
            RichTextMessage::FormatToggled(format_type) => {
                self.apply_formatting(format_type);
            }
            RichTextMessage::ColorSelected(color_code) => {
                self.apply_formatting(FormatType::Color(color_code));
            }
            RichTextMessage::BackgroundColorSelected(color_code) => {
                self.apply_formatting(FormatType::Background(color_code));
            }
            RichTextMessage::EmojiSelected(emoji) => {
                self.insert_text_at_cursor(&emoji);
                self.add_to_recent_emojis(emoji);
                self.show_emoji_picker = false;
            }
            RichTextMessage::ToggleToolbar => {
                self.show_toolbar = !self.show_toolbar;
            }
            RichTextMessage::ToggleEmojiPicker => {
                self.show_emoji_picker = !self.show_emoji_picker;
            }
            RichTextMessage::InsertText(text) => {
                self.insert_text_at_cursor(&text);
            }
            RichTextMessage::PasteText(text) => {
                // Clean pasted text and insert
                let cleaned = self.clean_pasted_text(text);
                self.insert_text_at_cursor(&cleaned);
            }
            RichTextMessage::Clear => {
                self.content.clear();
                self.cursor_position = 0;
                self.selection_start = None;
                self.selection_end = None;
                self.formatting_stack.clear();
            }
            RichTextMessage::Submit => {
                // This would typically be handled by the parent component
            }
            RichTextMessage::Undo | RichTextMessage::Redo => {
                // TODO: Implement undo/redo functionality
            }
        }
    }

    pub fn view(&self) -> Element<'_, RichTextMessage, Theme, Renderer> {
        let mut content = column![];

        // Formatting toolbar
        if self.show_toolbar {
            let toolbar = self.create_toolbar();
            content = content.push(toolbar);
        }

        // Main input area
        let input = self.create_input();
        content = content.push(input);

        // Emoji picker
        if self.show_emoji_picker {
            let emoji_picker = self.create_emoji_picker();
            content = content.push(emoji_picker);
        }

        // Character counter
        if let Some(max_len) = self.max_length {
            let counter_color = if self.content.len() > max_len * 9 / 10 {
                self.theme.scheme.error
            } else {
                self.theme.scheme.outline
            };

            let counter = MaterialText::new(format!("{}/{}", self.content.len(), max_len))
                .variant(TypographyVariant::LabelSmall)
                .color(iced::Color::from(counter_color))
                .build();

            content = content.push(
                container(counter)
                    .padding([4, 8])
                    .width(Length::Fill)
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        border: Border::default(),
                        ..Default::default()
                    }),
            );
        }

        container(content)
            .padding(8)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(iced::Color::from(
                    self.theme.scheme.surface_container,
                ))),
                border: Border {
                    color: iced::Color::from(self.theme.scheme.outline_variant),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn create_toolbar(&self) -> Element<'_, RichTextMessage, Theme, Renderer> {
        let toolbar_content = row![
            // Text formatting buttons
            MaterialButton::new("B")
                .variant(if self.formatting_stack.contains(&FormatType::Bold) {
                    ButtonVariant::Filled
                } else {
                    ButtonVariant::Text
                })
                .on_press(RichTextMessage::FormatToggled(FormatType::Bold))
                .build(),
            MaterialButton::new("I")
                .variant(if self.formatting_stack.contains(&FormatType::Italic) {
                    ButtonVariant::Filled
                } else {
                    ButtonVariant::Text
                })
                .on_press(RichTextMessage::FormatToggled(FormatType::Italic))
                .build(),
            MaterialButton::new("U")
                .variant(if self.formatting_stack.contains(&FormatType::Underline) {
                    ButtonVariant::Filled
                } else {
                    ButtonVariant::Text
                })
                .on_press(RichTextMessage::FormatToggled(FormatType::Underline))
                .build(),
            MaterialButton::new("S")
                .variant(
                    if self.formatting_stack.contains(&FormatType::Strikethrough) {
                        ButtonVariant::Filled
                    } else {
                        ButtonVariant::Text
                    }
                )
                .on_press(RichTextMessage::FormatToggled(FormatType::Strikethrough))
                .build(),
            // Color palette button
            MaterialButton::new("🎨")
                .variant(ButtonVariant::Text)
                .on_press(RichTextMessage::ToggleToolbar) // TODO: Show color picker
                .build(),
            // Monospace toggle
            MaterialButton::new("</>")
                .variant(if self.formatting_stack.contains(&FormatType::Monospace) {
                    ButtonVariant::Filled
                } else {
                    ButtonVariant::Text
                })
                .on_press(RichTextMessage::FormatToggled(FormatType::Monospace))
                .build(),
            // Emoji button
            MaterialButton::new("😀")
                .variant(if self.show_emoji_picker {
                    ButtonVariant::Filled
                } else {
                    ButtonVariant::Text
                })
                .on_press(RichTextMessage::ToggleEmojiPicker)
                .build(),
            // Clear formatting
            MaterialButton::new("X")
                .variant(ButtonVariant::Text)
                .on_press(RichTextMessage::FormatToggled(FormatType::Reset))
                .build(),
        ]
        .spacing(4);

        container(toolbar_content)
            .padding([4, 8])
            .width(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(iced::Color::from(
                    self.theme.scheme.surface_variant,
                ))),
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    fn create_input(&self) -> Element<'_, RichTextMessage, Theme, Renderer> {
        let _input_height = if self.is_multiline {
            Length::Fixed(120.0)
        } else {
            Length::Fixed(40.0)
        };

        // TODO: Replace with actual rich text input when available
        // For now, use a styled text_input with IRC codes visible
        text_input(&self.placeholder, &self.content)
            .on_input(RichTextMessage::ContentChanged)
            .on_submit(RichTextMessage::Submit)
            .padding(12)
            .size(14)
            .width(Length::Fill)
            .style(move |theme: &Theme, status| {
                let _palette = theme.extended_palette();

                let background_color = match status {
                    text_input::Status::Active => {
                        iced::Color::from(self.theme.scheme.surface_container_high)
                    }
                    text_input::Status::Hovered => {
                        iced::Color::from(self.theme.scheme.surface_container_highest)
                    }
                    text_input::Status::Focused { .. } => {
                        iced::Color::from(self.theme.scheme.surface_container_highest)
                    }
                    text_input::Status::Disabled => {
                        iced::Color::from(self.theme.scheme.surface_variant)
                    }
                };

                text_input::Style {
                    background: Background::Color(background_color),
                    border: Border {
                        color: match status {
                            text_input::Status::Focused { .. } => {
                                iced::Color::from(self.theme.scheme.primary)
                            }
                            _ => iced::Color::from(self.theme.scheme.outline),
                        },
                        width: if matches!(status, text_input::Status::Focused { .. }) {
                            2.0
                        } else {
                            1.0
                        },
                        radius: 8.0.into(),
                    },
                    icon: iced::Color::from(self.theme.scheme.on_surface_variant),
                    placeholder: iced::Color::from(self.theme.scheme.on_surface_variant),
                    value: iced::Color::from(self.theme.scheme.on_surface),
                    selection: iced::Color::from(self.theme.scheme.primary_container),
                }
            })
            .into()
    }

    fn create_emoji_picker(&self) -> Element<'_, RichTextMessage, Theme, Renderer> {
        let mut emoji_grid = column![];

        // Recent emojis section
        if !self.recent_emojis.is_empty() {
            let recent_label = MaterialText::new("Recently Used")
                .variant(TypographyVariant::LabelSmall)
                .color(iced::Color::from(self.theme.scheme.on_surface_variant))
                .build();

            emoji_grid = emoji_grid.push(container(recent_label).padding([8, 12]));

            let mut recent_row = row![].spacing(4);
            for emoji in &self.recent_emojis {
                let emoji_button = button(text(emoji).size(20))
                    .on_press(RichTextMessage::EmojiSelected(emoji.clone()))
                    .style(|_theme: &Theme, status| button::Style {
                        background: Some(Background::Color(match status {
                            button::Status::Hovered => {
                                iced::Color::from(self.theme.scheme.surface_container_high)
                            }
                            button::Status::Pressed => {
                                iced::Color::from(self.theme.scheme.surface_container_highest)
                            }
                            _ => Color::TRANSPARENT,
                        })),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .padding(4);

                recent_row = recent_row.push(emoji_button);
            }
            emoji_grid = emoji_grid.push(container(recent_row).padding([0, 8]));
        }

        // All emojis grid
        let all_label = MaterialText::new("All Emojis")
            .variant(TypographyVariant::LabelSmall)
            .color(iced::Color::from(self.theme.scheme.on_surface_variant))
            .build();

        emoji_grid = emoji_grid.push(container(all_label).padding(12));

        let mut current_row = row![].spacing(2);
        let emojis_per_row = 8;

        for (i, emoji) in COMMON_EMOJIS.iter().enumerate() {
            let emoji_button = button(text(*emoji).size(18))
                .on_press(RichTextMessage::EmojiSelected(emoji.to_string()))
                .style(|_theme: &Theme, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered => {
                            iced::Color::from(self.theme.scheme.surface_container_high)
                        }
                        button::Status::Pressed => {
                            iced::Color::from(self.theme.scheme.surface_container_highest)
                        }
                        _ => Color::TRANSPARENT,
                    })),
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .padding(6);

            current_row = current_row.push(emoji_button);

            if (i + 1) % emojis_per_row == 0 {
                emoji_grid = emoji_grid.push(container(current_row).padding([0, 8]));
                current_row = row![].spacing(2);
            }
        }

        // Add remaining emojis if any
        if COMMON_EMOJIS.len() % emojis_per_row != 0 {
            emoji_grid = emoji_grid.push(container(current_row).padding([0, 8]));
        }

        container(scrollable(emoji_grid).height(Length::Fixed(200.0)))
            .padding(4)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(iced::Color::from(
                    self.theme.scheme.surface_container,
                ))),
                border: Border {
                    color: iced::Color::from(self.theme.scheme.outline_variant),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    // Helper methods
    fn apply_formatting(&mut self, format_type: FormatType) {
        match format_type {
            FormatType::Reset => {
                self.formatting_stack.clear();
                self.insert_text_at_cursor("\u{000F}"); // IRC reset character
            }
            _ => {
                if let Some(pos) = self
                    .formatting_stack
                    .iter()
                    .position(|f| std::mem::discriminant(f) == std::mem::discriminant(&format_type))
                {
                    self.formatting_stack.remove(pos);
                } else {
                    self.formatting_stack.push(format_type.clone());
                }

                match format_type {
                    FormatType::Bold => self.insert_text_at_cursor("\u{0002}"),
                    FormatType::Italic => self.insert_text_at_cursor("\u{001D}"),
                    FormatType::Underline => self.insert_text_at_cursor("\u{001F}"),
                    FormatType::Strikethrough => self.insert_text_at_cursor("\u{001E}"),
                    FormatType::Color(code) => {
                        // Insert proper IRC color code
                        let color_code = format!("\u{0003}{code:02}");
                        self.insert_text_at_cursor(&color_code);
                    }
                    FormatType::Background(code) => {
                        // Insert proper IRC background color code
                        let bg_code = format!("\u{0003},{code:02}");
                        self.insert_text_at_cursor(&bg_code);
                    }
                    FormatType::Monospace => self.insert_text_at_cursor("\u{0011}"),
                    FormatType::Reset => self.insert_text_at_cursor("\u{000F}"),
                };
            }
        }
    }

    fn insert_text_at_cursor(&mut self, text: &str) {
        if let Some(max_len) = self.max_length {
            if self.content.len() + text.len() > max_len {
                return;
            }
        }

        self.content.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
    }

    fn add_to_recent_emojis(&mut self, emoji: String) {
        // Remove if already in recent
        self.recent_emojis.retain(|e| e != &emoji);

        // Add to front
        self.recent_emojis.insert(0, emoji);

        // Keep only last 8 recent emojis
        if self.recent_emojis.len() > 8 {
            self.recent_emojis.truncate(8);
        }
    }

    fn clean_pasted_text(&self, text: String) -> String {
        // Remove potentially harmful characters and limit length
        let cleaned = text
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect::<String>();

        if let Some(max_len) = self.max_length {
            if self.content.len() + cleaned.len() > max_len {
                let available = max_len.saturating_sub(self.content.len());
                cleaned.chars().take(available).collect()
            } else {
                cleaned
            }
        } else {
            cleaned
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
        self.selection_start = None;
        self.selection_end = None;
        self.formatting_stack.clear();
    }
}

impl Default for RichTextEditor {
    fn default() -> Self {
        Self::new(MaterialTheme::default())
    }
}
