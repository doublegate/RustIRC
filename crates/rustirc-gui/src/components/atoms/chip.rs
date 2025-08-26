//! Material Design 3 Chip components

use iced::{
    widget::{button, container, row, text},
    Element, Length, Background, Border, Color, Theme, Renderer,
    alignment::{Horizontal, Vertical},
};

use crate::themes::material_design_3::MaterialTheme;

/// Material Design 3 Chip variants
#[derive(Debug, Clone, PartialEq)]
pub enum ChipVariant {
    /// Assist chips help users complete tasks or provide suggestions
    Assist,
    /// Filter chips allow users to refine content by selecting or deselecting criteria
    Filter,
    /// Input chips represent discrete pieces of information entered by a user
    Input,
    /// Suggestion chips help users discover content or complete tasks
    Suggestion,
}

/// Material Design 3 Chip component
#[derive(Debug, Clone)]
pub struct MaterialChip<'a, Message> {
    label: String,
    variant: ChipVariant,
    theme: MaterialTheme,
    selected: bool,
    enabled: bool,
    on_press: Option<Message>,
    on_remove: Option<Message>,
    leading_icon: Option<&'a str>,
    trailing_icon: Option<&'a str>,
}

impl<'a, Message: Clone> MaterialChip<'a, Message> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ChipVariant::Assist,
            theme: MaterialTheme::default(),
            selected: false,
            enabled: true,
            on_press: None,
            on_remove: None,
            leading_icon: None,
            trailing_icon: None,
        }
    }

    pub fn variant(mut self, variant: ChipVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn on_remove(mut self, message: Message) -> Self {
        self.on_remove = Some(message);
        self
    }

    pub fn leading_icon(mut self, icon: &'a str) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_icon(mut self, icon: &'a str) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn view(self) -> Element<'a, Message, Theme, Renderer> {
        let mut content = row![];

        // Leading icon
        if let Some(icon) = self.leading_icon {
            content = content.push(
                text(icon)
                    .size(18)
                    .color(if self.enabled {
                        if self.selected {
                            self.theme.scheme.on_secondary_container
                        } else {
                            self.theme.scheme.primary
                        }
                    } else {
                        self.theme.scheme.on_surface.scale_alpha(0.38)
                    })
            );
        }

        // Label
        content = content.push(
            text(&self.label)
                .size(14)
                .color(if self.enabled {
                    if self.selected {
                        match self.variant {
                            ChipVariant::Filter => self.theme.scheme.on_secondary_container,
                            _ => self.theme.scheme.on_surface,
                        }
                    } else {
                        self.theme.scheme.on_surface
                    }
                } else {
                    self.theme.scheme.on_surface.scale_alpha(0.38)
                })
        );

        // Trailing icon or remove button
        if let Some(message) = &self.on_remove {
            content = content.push(
                button(text("×").size(16))
                    .on_press(message.clone())
                    .style(|_theme: &Theme, _status| button::Style {
                        background: None,
                        border: Border::default(),
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    })
            );
        } else if let Some(icon) = self.trailing_icon {
            content = content.push(
                text(icon)
                    .size(18)
                    .color(if self.enabled {
                        self.theme.scheme.on_surface_variant
                    } else {
                        self.theme.scheme.on_surface.scale_alpha(0.38)
                    })
            );
        }

        let chip_content = container(content.spacing(8))
            .padding([8, 16])
            .style(move |_theme: &Theme| {
                let (background_color, border_color) = match (&self.variant, self.selected, self.enabled) {
                    (ChipVariant::Filter, true, true) => (
                        self.theme.scheme.secondary_container,
                        Color::TRANSPARENT,
                    ),
                    (ChipVariant::Input, _, true) => (
                        self.theme.scheme.surface_container_low,
                        self.theme.scheme.outline,
                    ),
                    (_, false, true) => (
                        Color::TRANSPARENT,
                        self.theme.scheme.outline,
                    ),
                    (_, _, false) => (
                        self.theme.scheme.on_surface.scale_alpha(0.12),
                        self.theme.scheme.on_surface.scale_alpha(0.12),
                    ),
                };

                container::Style {
                    background: Some(Background::Color(background_color)),
                    border: Border {
                        color: border_color,
                        width: if border_color == Color::TRANSPARENT { 0.0 } else { 1.0 },
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            });

        if let Some(message) = self.on_press {
            button(chip_content)
                .on_press(message)
                .style(move |_theme: &Theme, status| {
                    let hover_color = match status {
                        button::Status::Hovered => Some(self.theme.scheme.on_surface.scale_alpha(0.08)),
                        button::Status::Pressed => Some(self.theme.scheme.on_surface.scale_alpha(0.12)),
                        _ => None,
                    };

                    button::Style {
                        background: hover_color.map(Background::Color),
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    }
                })
                .into()
        } else {
            chip_content.into()
        }
    }
}