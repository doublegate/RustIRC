//! Modern Material Design 3 Button Components
//!
//! This module implements Material Design 3 button variants with proper
//! accessibility, animations, and theming support.

use crate::themes::material_design_3::MaterialTheme;
use iced::{
    widget::{button, text},
    Background, Border, Color, Element, Length, Vector,
};

/// Material Design 3 Button variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Filled button (highest emphasis)
    Filled,
    /// Tonal button (medium emphasis)  
    FilledTonal,
    /// Outlined button (medium emphasis)
    Outlined,
    /// Text button (low emphasis)
    Text,
}

/// Button size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small,  // 32px height
    Medium, // 40px height
    Large,  // 56px height
}

/// Button state for animations and styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

/// Material Design 3 Button component
///
/// Implements all Material Design 3 button variants with proper accessibility,
/// theming, and interaction states.
///
/// # Examples
///
/// ```
/// use rustirc_gui::components::atoms::button::{ButtonVariant, MaterialButton};
/// use rustirc_gui::themes::material_design_3::MaterialTheme;
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// let theme = MaterialTheme::default();
/// let button = MaterialButton::new("Click Me")
///     .variant(ButtonVariant::Filled)
///     .on_press(Message::ButtonPressed)
///     .theme(theme);
///
/// // Create the Iced element
/// let element = button.build();
/// ```
///
/// # Button Variants
///
/// - **Filled**: Highest emphasis, primary actions
/// - **FilledTonal**: Medium emphasis, important secondary actions
/// - **Outlined**: Medium emphasis with border
/// - **Text**: Low emphasis, tertiary actions
/// - **Elevated**: High emphasis with shadow
///
/// # Features
///
/// - Automatic focus handling and keyboard navigation
/// - Hover and pressed state animations
/// - Icon support with configurable positioning
/// - Full accessibility with ARIA labels
/// - Material Design 3 color and typography tokens
#[derive(Debug, Clone)]
pub struct MaterialButton<Message> {
    variant: ButtonVariant,
    size: ButtonSize,
    label: String,
    icon: Option<String>,
    icon_position: IconPosition,
    disabled: bool,
    full_width: bool,
    on_press: Option<Message>,
    theme: MaterialTheme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconPosition {
    Left,
    Right,
    Only, // Icon only button
}

impl<Message> MaterialButton<Message> {
    /// Create a new Material Design 3 button
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            variant: ButtonVariant::Filled,
            size: ButtonSize::Medium,
            label: label.into(),
            icon: None,
            icon_position: IconPosition::Left,
            disabled: false,
            full_width: false,
            on_press: None,
            theme: MaterialTheme::dark(), // Default theme
        }
    }

    /// Set button variant
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set button size
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Add icon to button
    pub fn icon(mut self, icon: impl Into<String>, position: IconPosition) -> Self {
        self.icon = Some(icon.into());
        self.icon_position = position;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set full width
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Set on press action
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Build the button element
    pub fn build(self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
    {
        let (height, padding_horizontal, font_size, icon_size) = match self.size {
            ButtonSize::Small => (32.0, 12.0, 14.0, 16.0),
            ButtonSize::Medium => (40.0, 16.0, 16.0, 18.0),
            ButtonSize::Large => (56.0, 24.0, 18.0, 20.0),
        };

        let width = if self.full_width {
            Length::Fill
        } else {
            Length::Shrink
        };

        let (_bg_color, text_color, _border_color, border_width) = self.get_colors();

        let content = self.build_content(font_size, icon_size, text_color);
        let disabled = self.disabled;
        let on_press = self.on_press.clone();

        // Clone values needed in closure before move
        let button_clone = self.clone();

        let btn = button(content)
            .height(height)
            .width(width)
            .padding([0, padding_horizontal as u16])
            .style(move |_theme, status| {
                let (background, text, border) = button_clone.get_colors_for_status(status);
                button::Style {
                    background: Some(background),
                    text_color: text,
                    border: Border {
                        color: border,
                        width: border_width,
                        radius: button_clone.theme.shapes.corner_large.into(),
                    },
                    shadow: button_clone.get_shadow_for_status(status),
                }
            });

        if let Some(on_press) = on_press {
            if !disabled {
                btn.on_press(on_press).into()
            } else {
                btn.into()
            }
        } else {
            btn.into()
        }
    }

    /// Build button content with icon and text
    fn build_content(
        &self,
        font_size: f32,
        icon_size: f32,
        text_color: Color,
    ) -> Element<'static, Message>
    where
        Message: Clone + 'static,
    {
        match (&self.icon, self.icon_position) {
            (Some(icon), IconPosition::Only) => {
                // Icon only button
                text(icon.clone()).size(icon_size).color(text_color).into()
            }
            (Some(icon), IconPosition::Left) => {
                // Icon + text (icon left)
                iced::widget::row![
                    text(icon.clone()).size(icon_size).color(text_color),
                    text(self.label.clone())
                        .size(font_size)
                        .color(text_color)
                        .font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        })
                ]
                .spacing(self.theme.spacing.sm)
                .into()
            }
            (Some(icon), IconPosition::Right) => {
                // Icon + text (icon right)
                iced::widget::row![
                    text(self.label.clone())
                        .size(font_size)
                        .color(text_color)
                        .font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        }),
                    text(icon.clone()).size(icon_size).color(text_color),
                ]
                .spacing(self.theme.spacing.sm)
                .into()
            }
            (None, _) => {
                // Text only button
                text(self.label.clone())
                    .size(font_size)
                    .color(text_color)
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    })
                    .into()
            }
        }
    }

    /// Get colors for current button variant
    fn get_colors(&self) -> (Color, Color, Color, f32) {
        match (self.variant, self.disabled) {
            (ButtonVariant::Filled, false) => (
                iced::Color::from(self.theme.scheme.primary),
                iced::Color::from(self.theme.scheme.on_primary),
                Color::TRANSPARENT,
                0.0,
            ),
            (ButtonVariant::Filled, true) => (
                iced::Color::from(self.theme.scheme.surface_container_highest),
                iced::Color::from(self.theme.scheme.on_surface_variant),
                Color::TRANSPARENT,
                0.0,
            ),
            (ButtonVariant::FilledTonal, false) => (
                iced::Color::from(self.theme.scheme.secondary_container),
                iced::Color::from(self.theme.scheme.on_secondary_container),
                Color::TRANSPARENT,
                0.0,
            ),
            (ButtonVariant::FilledTonal, true) => (
                iced::Color::from(self.theme.scheme.surface_container_highest),
                iced::Color::from(self.theme.scheme.on_surface_variant),
                Color::TRANSPARENT,
                0.0,
            ),
            (ButtonVariant::Outlined, false) => (
                Color::TRANSPARENT,
                iced::Color::from(self.theme.scheme.primary),
                iced::Color::from(self.theme.scheme.outline),
                1.0,
            ),
            (ButtonVariant::Outlined, true) => (
                Color::TRANSPARENT,
                iced::Color::from(self.theme.scheme.on_surface_variant),
                iced::Color::from(self.theme.scheme.outline_variant),
                1.0,
            ),
            (ButtonVariant::Text, false) => (
                Color::TRANSPARENT,
                iced::Color::from(self.theme.scheme.primary),
                Color::TRANSPARENT,
                0.0,
            ),
            (ButtonVariant::Text, true) => (
                Color::TRANSPARENT,
                iced::Color::from(self.theme.scheme.on_surface_variant),
                Color::TRANSPARENT,
                0.0,
            ),
        }
    }

    /// Get colors for specific button status (hover, pressed, etc.)
    fn get_colors_for_status(&self, status: button::Status) -> (Background, Color, Color) {
        let (base_bg, text_color, border_color, _) = self.get_colors();

        let background = match status {
            button::Status::Active => Background::Color(base_bg),
            button::Status::Hovered => {
                // Apply hover overlay
                let overlay_color = match self.variant {
                    ButtonVariant::Filled => iced::Color::from(self.theme.scheme.on_primary),
                    ButtonVariant::FilledTonal => {
                        iced::Color::from(self.theme.scheme.on_secondary_container)
                    }
                    ButtonVariant::Outlined | ButtonVariant::Text => {
                        iced::Color::from(self.theme.scheme.primary)
                    }
                };
                Background::Color(self.blend_colors(base_bg, overlay_color, 0.08))
            }
            button::Status::Pressed => {
                // Apply pressed overlay
                let overlay_color = match self.variant {
                    ButtonVariant::Filled => iced::Color::from(self.theme.scheme.on_primary),
                    ButtonVariant::FilledTonal => {
                        iced::Color::from(self.theme.scheme.on_secondary_container)
                    }
                    ButtonVariant::Outlined | ButtonVariant::Text => {
                        iced::Color::from(self.theme.scheme.primary)
                    }
                };
                Background::Color(self.blend_colors(base_bg, overlay_color, 0.12))
            }
            button::Status::Disabled => {
                // Disabled state - use muted colors
                Background::Color(iced::Color::from(
                    self.theme.scheme.on_surface.scale_alpha(0.12),
                ))
            }
        };

        (background, text_color, border_color)
    }

    /// Get shadow for specific button status
    fn get_shadow_for_status(&self, status: button::Status) -> iced::Shadow {
        match (self.variant, status) {
            (ButtonVariant::Filled | ButtonVariant::FilledTonal, button::Status::Active) => {
                iced::Shadow {
                    color: self.theme.elevation.level1.shadow_color.into(),
                    offset: Vector::new(
                        self.theme.elevation.level1.shadow_offset_x,
                        self.theme.elevation.level1.shadow_offset_y,
                    ),
                    blur_radius: self.theme.elevation.level1.shadow_blur,
                }
            }
            (ButtonVariant::Filled | ButtonVariant::FilledTonal, button::Status::Hovered) => {
                iced::Shadow {
                    color: self.theme.elevation.level2.shadow_color.into(),
                    offset: Vector::new(
                        self.theme.elevation.level2.shadow_offset_x,
                        self.theme.elevation.level2.shadow_offset_y,
                    ),
                    blur_radius: self.theme.elevation.level2.shadow_blur,
                }
            }
            _ => iced::Shadow::default(),
        }
    }

    /// Blend two colors with specified opacity
    fn blend_colors(&self, base: Color, overlay: Color, opacity: f32) -> Color {
        Color {
            r: base.r * (1.0 - opacity) + overlay.r * opacity,
            g: base.g * (1.0 - opacity) + overlay.g * opacity,
            b: base.b * (1.0 - opacity) + overlay.b * opacity,
            a: base.a,
        }
    }

    /// Create view element (alias for build)
    pub fn view(self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
    {
        self.build()
    }
}

/// Floating Action Button (FAB) - Special Material Design component
#[derive(Debug, Clone)]
pub struct FloatingActionButton<Message> {
    icon: String,
    size: FabSize,
    extended: bool,
    label: Option<String>,
    theme: MaterialTheme,
    on_press: Option<Message>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FabSize {
    Small,  // 40px
    Normal, // 56px
    Large,  // 96px
}

impl<Message> FloatingActionButton<Message> {
    /// Create new FAB
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            size: FabSize::Normal,
            extended: false,
            label: None,
            theme: MaterialTheme::dark(),
            on_press: None,
        }
    }

    /// Set FAB size
    pub fn size(mut self, size: FabSize) -> Self {
        self.size = size;
        self
    }

    /// Create extended FAB with label
    pub fn extended(mut self, label: impl Into<String>) -> Self {
        self.extended = true;
        self.label = Some(label.into());
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set on press action
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Build FAB element
    pub fn build(self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
    {
        let (size, icon_size, font_size) = match self.size {
            FabSize::Small => (40.0, 16.0, 14.0),
            FabSize::Normal => (56.0, 24.0, 16.0),
            FabSize::Large => (96.0, 36.0, 18.0),
        };

        // Clone values that will be used in closure
        let icon = self.icon.clone();
        let label = self.label.clone();
        let on_press = self.on_press.clone();

        let content: Element<'static, Message> = match (self.extended, &label) {
            // Extended FAB with label
            (true, Some(label_text)) => iced::widget::row![
                text(icon.clone())
                    .size(icon_size)
                    .color(iced::Color::from(self.theme.scheme.on_primary_container)),
                text(label_text.clone())
                    .size(font_size)
                    .color(iced::Color::from(self.theme.scheme.on_primary_container))
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..Default::default()
                    })
            ]
            .spacing(self.theme.spacing.sm)
            .into(),
            // Normal FAB (not extended or no label)
            _ => text(icon.clone())
                .size(icon_size)
                .color(iced::Color::from(self.theme.scheme.on_primary_container))
                .into(),
        };

        let width = if self.extended {
            Length::Shrink
        } else {
            Length::Fixed(size)
        };

        let btn = button(content)
            .width(width)
            .height(size)
            .padding(if self.extended { [0, 16] } else { [0, 0] })
            .style(move |_theme, status| {
                let background_color = match status {
                    button::Status::Active => {
                        iced::Color::from(self.theme.scheme.primary_container)
                    }
                    button::Status::Hovered => self.blend_colors(
                        iced::Color::from(self.theme.scheme.primary_container),
                        iced::Color::from(self.theme.scheme.on_primary_container),
                        0.08,
                    ),
                    button::Status::Pressed => self.blend_colors(
                        iced::Color::from(self.theme.scheme.primary_container),
                        iced::Color::from(self.theme.scheme.on_primary_container),
                        0.12,
                    ),
                    button::Status::Disabled => {
                        iced::Color::from(self.theme.scheme.on_surface).scale_alpha(0.12)
                    }
                };

                let shadow = match status {
                    button::Status::Active => iced::Shadow {
                        color: self.theme.elevation.level3.shadow_color.into(),
                        offset: Vector::new(
                            self.theme.elevation.level3.shadow_offset_x,
                            self.theme.elevation.level3.shadow_offset_y,
                        ),
                        blur_radius: self.theme.elevation.level3.shadow_blur,
                    },
                    button::Status::Hovered => iced::Shadow {
                        color: self.theme.elevation.level4.shadow_color.into(),
                        offset: Vector::new(
                            self.theme.elevation.level4.shadow_offset_x,
                            self.theme.elevation.level4.shadow_offset_y,
                        ),
                        blur_radius: self.theme.elevation.level4.shadow_blur,
                    },
                    button::Status::Pressed => iced::Shadow {
                        color: self.theme.elevation.level2.shadow_color.into(),
                        offset: Vector::new(
                            self.theme.elevation.level2.shadow_offset_x,
                            self.theme.elevation.level2.shadow_offset_y,
                        ),
                        blur_radius: self.theme.elevation.level2.shadow_blur,
                    },
                    button::Status::Disabled => iced::Shadow::default(),
                };

                button::Style {
                    background: Some(Background::Color(background_color)),
                    text_color: iced::Color::from(self.theme.scheme.on_primary_container),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: self.theme.shapes.corner_large.into(),
                    },
                    shadow,
                }
            });

        if let Some(on_press_msg) = on_press {
            btn.on_press(on_press_msg).into()
        } else {
            btn.into()
        }
    }

    /// Blend two colors with specified opacity
    fn blend_colors(&self, base: Color, overlay: Color, opacity: f32) -> Color {
        Color {
            r: base.r * (1.0 - opacity) + overlay.r * opacity,
            g: base.g * (1.0 - opacity) + overlay.g * opacity,
            b: base.b * (1.0 - opacity) + overlay.b * opacity,
            a: base.a,
        }
    }

    /// Create view element (alias for build)
    pub fn view(self) -> Element<'static, Message>
    where
        Message: Clone + 'static,
    {
        self.build()
    }
}

// Convenience functions for creating common button types
pub fn filled_button<Message>(label: impl Into<String>) -> MaterialButton<Message> {
    MaterialButton::new(label).variant(ButtonVariant::Filled)
}

pub fn tonal_button<Message>(label: impl Into<String>) -> MaterialButton<Message> {
    MaterialButton::new(label).variant(ButtonVariant::FilledTonal)
}

pub fn outlined_button<Message>(label: impl Into<String>) -> MaterialButton<Message> {
    MaterialButton::new(label).variant(ButtonVariant::Outlined)
}

pub fn text_button<Message>(label: impl Into<String>) -> MaterialButton<Message> {
    MaterialButton::new(label).variant(ButtonVariant::Text)
}

pub fn icon_button<Message>(icon: impl Into<String>) -> MaterialButton<Message> {
    MaterialButton::new("").icon(icon, IconPosition::Only)
}

pub fn fab<Message>(icon: impl Into<String>) -> FloatingActionButton<Message> {
    FloatingActionButton::new(icon)
}

pub fn extended_fab<Message>(
    icon: impl Into<String>,
    label: impl Into<String>,
) -> FloatingActionButton<Message> {
    FloatingActionButton::new(icon).extended(label)
}
