//! Material Design 3 Typography System
//!
//! This module provides semantic typography components following
//! Material Design 3 specifications with proper accessibility.

use crate::themes::material_design_3::{FontWeight, MaterialTheme, TypographyToken};
use iced::{
    widget::text,
    Color, Element, Font,
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
        let token = self.get_typography_token().clone();
        let text_color = self.color.unwrap_or_else(|| self.get_default_color());

        // Use static font family for lifetime requirements
        let font_family = match token.font_family.as_str() {
            "Inter" => iced::font::Family::Name("Inter"),
            "Roboto" => iced::font::Family::Name("Roboto"),
            "System UI" => iced::font::Family::Name("System UI"),
            _ => iced::font::Family::SansSerif, // Fallback to default sans-serif font
        };

        let font = Font {
            family: font_family,
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

        let mut txt = text(self.content.clone())
            .size(token.font_size)
            .color(text_color)
            .font(font);

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
            | TypographyVariant::BodyLarge => self.theme.scheme.on_surface.into(),

            TypographyVariant::TitleMedium
            | TypographyVariant::TitleSmall
            | TypographyVariant::BodyMedium
            | TypographyVariant::CodeLarge
            | TypographyVariant::CodeMedium => self.theme.scheme.on_surface_variant.into(),

            TypographyVariant::LabelLarge
            | TypographyVariant::LabelMedium
            | TypographyVariant::LabelSmall => self.theme.scheme.primary.into(),

            TypographyVariant::BodySmall | TypographyVariant::CodeSmall => {
                self.theme.scheme.on_surface_variant.into()
            }
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
        Message: 'static + Clone,
    {
        // For now, we'll combine all spans into one text widget
        // A full rich text implementation would need custom widget development
        let combined_text = self.spans
            .iter()
            .map(|span| span.text.as_str())
            .collect::<Vec<_>>()
            .join("");

        // Use the first span's formatting if any exists
        let (font, color) = if let Some(first_span) = self.spans.first() {
            let span_color = first_span.color.unwrap_or(self.theme.scheme.on_surface.into());
            let weight = first_span.weight.unwrap_or(FontWeight::Regular);

            let font = Font {
                family: if first_span.code {
                    iced::font::Family::Name("JetBrains Mono")
                } else {
                    iced::font::Family::Name("Inter")
                },
                weight: self.convert_font_weight(weight),
                stretch: iced::font::Stretch::Normal,
                style: if first_span.italic {
                    iced::font::Style::Italic
                } else {
                    iced::font::Style::Normal
                },
            };
            (font, span_color)
        } else {
            (
                Font {
                    family: iced::font::Family::Name("Inter"),
                    weight: iced::font::Weight::Normal,
                    stretch: iced::font::Stretch::Normal,
                    style: iced::font::Style::Normal,
                },
                self.theme.scheme.on_surface,
            )
        };

        text(combined_text)
            .size(self.theme.typography.body_medium.font_size)
            .color(color)
            .font(font)
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

// Convenience functions for common text styles that return Element
pub fn display_large<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::DisplayLarge).build()
}

pub fn display_medium<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::DisplayMedium).build()
}

pub fn display_small<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::DisplaySmall).build()
}

pub fn headline_large<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::HeadlineLarge).build()
}

pub fn headline_medium<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::HeadlineMedium).build()
}

pub fn headline_small<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::HeadlineSmall).build()
}

pub fn title_large<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::TitleLarge).build()
}

pub fn title_medium<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::TitleMedium).build()
}

pub fn title_small<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::TitleSmall).build()
}

pub fn body_large<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::BodyLarge).build()
}

pub fn body_medium<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::BodyMedium).build()
}

pub fn body_small<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::BodySmall).build()
}

pub fn label_large<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::LabelLarge).build()
}

pub fn label_medium<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::LabelMedium).build()
}

pub fn label_small<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::LabelSmall).build()
}

pub fn code_large<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::CodeLarge).build()
}

pub fn code_medium<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::CodeMedium).build()
}

pub fn code_small<Message>(text_content: impl Into<String>) -> Element<'static, Message>
where
    Message: 'static,
{
    MaterialText::new(text_content).variant(TypographyVariant::CodeSmall).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typography_variants() {
        let text = MaterialText::new("Hello, World!")
            .variant(TypographyVariant::HeadlineLarge)
            .color(Color::BLACK);
        
        assert_eq!(text.variant, TypographyVariant::HeadlineLarge);
        assert_eq!(text.color, Some(Color::BLACK));
    }

    #[test]
    fn test_rich_text_creation() {
        let rich_text = RichText::new()
            .span(TextSpan::new("Bold").bold())
            .span(TextSpan::new(" and ").italic())
            .span(TextSpan::new("code").code());
        
        assert_eq!(rich_text.spans.len(), 3);
        assert!(rich_text.spans[0].weight == Some(FontWeight::Bold));
        assert!(rich_text.spans[1].italic);
        assert!(rich_text.spans[2].code);
    }

    #[test]
    fn test_text_span_formatting() {
        let span = TextSpan::new("Formatted text")
            .bold()
            .italic()
            .underline()
            .strikethrough()
            .color(Color::from_rgb(0.5, 0.5, 0.5));
        
        assert_eq!(span.weight, Some(FontWeight::Bold));
        assert!(span.italic);
        assert!(span.underline);
        assert!(span.strikethrough);
        assert!(span.color.is_some());
    }
}
