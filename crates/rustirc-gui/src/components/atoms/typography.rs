//! Material Design 3 Typography System
//! 
//! This module provides semantic typography components following
//! Material Design 3 specifications with proper accessibility.

use crate::themes::material_design_3::{MaterialTheme, TypographyToken, FontWeight};
use iced::{
    widget::{text, rich_text, span, text_input, container},
    Element, Color, Font, Length,
};

/// Typography variant based on Material Design 3 type scale
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypographyVariant {
    // Display styles (large headlines)
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,

    // Headline styles
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,

    // Title styles
    TitleLarge,
    TitleMedium,
    TitleSmall,

    // Label styles (buttons, chips, etc.)
    LabelLarge,
    LabelMedium,
    LabelSmall,

    // Body styles (primary text)
    BodyLarge,
    BodyMedium,
    BodySmall,

    // Code styles (monospace)
    CodeLarge,
    CodeMedium,
    CodeSmall,
}

/// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Material Design 3 Text Component
#[derive(Debug, Clone)]
pub struct MaterialText {
    content: String,
    variant: TypographyVariant,
    color: Option<Color>,
    align: TextAlign,
    max_lines: Option<usize>,
    selectable: bool,
    theme: MaterialTheme,
}

impl MaterialText {
    /// Create new text component
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            variant: TypographyVariant::BodyMedium,
            color: None,
            align: TextAlign::Left,
            max_lines: None,
            selectable: false,
            theme: MaterialTheme::dark(),
        }
    }

    /// Set typography variant
    pub fn variant(mut self, variant: TypographyVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set text color (overrides theme default)
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set text alignment
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Set maximum number of lines (with ellipsis)
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }

    /// Make text selectable
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Build text element
    pub fn build<Message>(self) -> Element<'static, Message>
    where
        Message: 'static,
    {
        let token = self.get_typography_token();
        let text_color = self.color.unwrap_or_else(|| self.get_default_color());
        
        let font = Font {
            family: iced::font::Family::Name(&token.font_family),
            weight: self.convert_font_weight(token.font_weight),
            stretch: iced::font::Stretch::Normal,
            style: iced::font::Style::Normal,
        };

        let horizontal_alignment = match self.align {
            TextAlign::Left => iced::alignment::Horizontal::Left,
            TextAlign::Center => iced::alignment::Horizontal::Center,
            TextAlign::Right => iced::alignment::Horizontal::Right,
            TextAlign::Justify => iced::alignment::Horizontal::Left, // Iced doesn't support justify
        };

        let mut txt = text(&self.content)
            .size(token.font_size)
            .color(text_color)
            .font(font)
            ;

        // Apply line height through container if needed
        if token.line_height != token.font_size {
            // Note: Iced doesn't directly support line-height, 
            // this would need custom implementation
        }

        txt.into()
    }

    /// Get typography token for current variant
    fn get_typography_token(&self) -> &TypographyToken {
        match self.variant {
            TypographyVariant::DisplayLarge => &self.theme.typography.display_large,
            TypographyVariant::DisplayMedium => &self.theme.typography.display_medium,
            TypographyVariant::DisplaySmall => &self.theme.typography.display_small,
            TypographyVariant::HeadlineLarge => &self.theme.typography.headline_large,
            TypographyVariant::HeadlineMedium => &self.theme.typography.headline_medium,
            TypographyVariant::HeadlineSmall => &self.theme.typography.headline_small,
            TypographyVariant::TitleLarge => &self.theme.typography.title_large,
            TypographyVariant::TitleMedium => &self.theme.typography.title_medium,
            TypographyVariant::TitleSmall => &self.theme.typography.title_small,
            TypographyVariant::LabelLarge => &self.theme.typography.label_large,
            TypographyVariant::LabelMedium => &self.theme.typography.label_medium,
            TypographyVariant::LabelSmall => &self.theme.typography.label_small,
            TypographyVariant::BodyLarge => &self.theme.typography.body_large,
            TypographyVariant::BodyMedium => &self.theme.typography.body_medium,
            TypographyVariant::BodySmall => &self.theme.typography.body_small,
            TypographyVariant::CodeLarge => &self.theme.typography.code_large,
            TypographyVariant::CodeMedium => &self.theme.typography.code_medium,
            TypographyVariant::CodeSmall => &self.theme.typography.code_small,
        }
    }

    /// Get default color for current variant
    fn get_default_color(&self) -> Color {
        match self.variant {
            TypographyVariant::DisplayLarge
            | TypographyVariant::DisplayMedium
            | TypographyVariant::DisplaySmall
            | TypographyVariant::HeadlineLarge
            | TypographyVariant::HeadlineMedium
            | TypographyVariant::HeadlineSmall
            | TypographyVariant::TitleLarge
            | TypographyVariant::BodyLarge => self.theme.scheme.on_surface,
            
            TypographyVariant::TitleMedium
            | TypographyVariant::TitleSmall
            | TypographyVariant::BodyMedium
            | TypographyVariant::CodeLarge
            | TypographyVariant::CodeMedium => self.theme.scheme.on_surface_variant,
            
            TypographyVariant::LabelLarge
            | TypographyVariant::LabelMedium
            | TypographyVariant::LabelSmall => self.theme.scheme.primary,
            
            TypographyVariant::BodySmall
            | TypographyVariant::CodeSmall => self.theme.scheme.on_surface_variant,
        }
    }

    /// Convert custom font weight to Iced font weight
    fn convert_font_weight(&self, weight: FontWeight) -> iced::font::Weight {
        match weight {
            FontWeight::Thin => iced::font::Weight::Thin,
            FontWeight::ExtraLight => iced::font::Weight::ExtraLight,
            FontWeight::Light => iced::font::Weight::Light,
            FontWeight::Regular => iced::font::Weight::Normal,
            FontWeight::Medium => iced::font::Weight::Medium,
            FontWeight::SemiBold => iced::font::Weight::Semibold,
            FontWeight::Bold => iced::font::Weight::Bold,
            FontWeight::ExtraBold => iced::font::Weight::ExtraBold,
            FontWeight::Black => iced::font::Weight::Black,
        }
    }
}

/// Rich Text Component for formatted IRC messages
#[derive(Debug, Clone)]
pub struct RichText {
    spans: Vec<TextSpan>,
    theme: MaterialTheme,
    selectable: bool,
}

/// Individual text span with formatting
#[derive(Debug, Clone)]
pub struct TextSpan {
    text: String,
    color: Option<Color>,
    weight: Option<FontWeight>,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    code: bool,
}

impl RichText {
    /// Create new rich text component
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            theme: MaterialTheme::dark(),
            selectable: true,
        }
    }

    /// Add text span
    pub fn span(mut self, span: TextSpan) -> Self {
        self.spans.push(span);
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Make text selectable
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Build rich text element
    pub fn build<Message>(self) -> Element<'static, Message>
    where
        Message: 'static,
    {
        // Create rich text spans
        let mut rich_spans = Vec::new();
        
        for text_span in self.spans {
            let color = text_span.color.unwrap_or(self.theme.scheme.on_surface);
            let weight = text_span.weight.unwrap_or(FontWeight::Regular);
            
            let font = Font {
                family: if text_span.code {
                    iced::font::Family::Name("JetBrains Mono")
                } else {
                    iced::font::Family::Name("Inter")
                },
                weight: self.convert_font_weight(weight),
                stretch: iced::font::Stretch::Normal,
                style: if text_span.italic {
                    iced::font::Style::Italic
                } else {
                    iced::font::Style::Normal
                },
            };

            let mut rich_span = span(&text_span.text)
                .color(color)
                .font(font);

            if text_span.underline {
                rich_span = rich_span.underline(true);
            }

            if text_span.strikethrough {
                rich_span = rich_span.strikethrough(true);
            }

            rich_spans.push(rich_span);
        }

        rich_text(rich_spans)
            .size(self.theme.typography.body_medium.font_size)
            .into()
    }

    /// Convert custom font weight to Iced font weight
    fn convert_font_weight(&self, weight: FontWeight) -> iced::font::Weight {
        match weight {
            FontWeight::Thin => iced::font::Weight::Thin,
            FontWeight::ExtraLight => iced::font::Weight::ExtraLight,
            FontWeight::Light => iced::font::Weight::Light,
            FontWeight::Regular => iced::font::Weight::Normal,
            FontWeight::Medium => iced::font::Weight::Medium,
            FontWeight::SemiBold => iced::font::Weight::Semibold,
            FontWeight::Bold => iced::font::Weight::Bold,
            FontWeight::ExtraBold => iced::font::Weight::ExtraBold,
            FontWeight::Black => iced::font::Weight::Black,
        }
    }
}

impl TextSpan {
    /// Create new text span
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            weight: None,
            italic: false,
            underline: false,
            strikethrough: false,
            code: false,
        }
    }

    /// Set span color
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set font weight
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Make text bold
    pub fn bold(mut self) -> Self {
        self.weight = Some(FontWeight::Bold);
        self
    }

    /// Make text italic
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// Underline text
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    /// Strikethrough text
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = true;
        self
    }

    /// Mark as code (monospace)
    pub fn code(mut self) -> Self {
        self.code = true;
        self
    }
}

impl Default for RichText {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience functions for common text styles
pub fn display_large<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::DisplayLarge)
}

pub fn display_medium<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::DisplayMedium)
}

pub fn display_small<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::DisplaySmall)
}

pub fn headline_large<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::HeadlineLarge)
}

pub fn headline_medium<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::HeadlineMedium)
}

pub fn headline_small<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::HeadlineSmall)
}

pub fn title_large<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::TitleLarge)
}

pub fn title_medium<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::TitleMedium)
}

pub fn title_small<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::TitleSmall)
}

pub fn body_large<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::BodyLarge)
}

pub fn body_medium<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::BodyMedium)
}

pub fn body_small<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::BodySmall)
}

pub fn label_large<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::LabelLarge)
}

pub fn label_medium<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::LabelMedium)
}

pub fn label_small<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::LabelSmall)
}

pub fn code_large<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::CodeLarge)
}

pub fn code_medium<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::CodeMedium)
}

pub fn code_small<Message>(text: impl Into<String>) -> MaterialText {
    MaterialText::new(text).variant(TypographyVariant::CodeSmall)
}