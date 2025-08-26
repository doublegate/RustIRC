//! Material Design 3 Chip components

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, row, text},
    Background, Border, Color, Element, Length, Renderer, Theme,
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

impl<'a, Message: Clone + 'a> MaterialChip<'a, Message> {
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
        // Extract all values we need to avoid lifetime issues
        let label = self.label.clone();
        let enabled = self.enabled;
        let selected = self.selected;
        let variant = self.variant.clone();
        let theme = self.theme.clone();
        let on_press = self.on_press.clone();
        let on_remove = self.on_remove.clone();
        let leading_icon = self.leading_icon;
        let trailing_icon = self.trailing_icon;

        let mut content = row![];

        // Leading icon
        if let Some(icon) = leading_icon {
            content = content.push(text(icon).size(18).color(if enabled {
                if selected {
                    theme.scheme.on_secondary_container
                } else {
                    theme.scheme.primary
                }
            } else {
                theme.scheme.on_surface.scale_alpha(0.38)
            }));
        }

        // Label
        content = content.push(text(label).size(14).color(if enabled {
            if selected {
                match variant {
                    ChipVariant::Filter => theme.scheme.on_secondary_container,
                    _ => theme.scheme.on_surface,
                }
            } else {
                theme.scheme.on_surface
            }
        } else {
            theme.scheme.on_surface.scale_alpha(0.38)
        }));

        // Trailing icon or remove button
        if let Some(message) = &on_remove {
            content = content.push(button(text("×").size(16)).on_press(message.clone()).style(
                |_theme: &Theme, _status| button::Style {
                    background: None,
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                },
            ));
        } else if let Some(icon) = trailing_icon {
            content = content.push(text(icon).size(18).color(if enabled {
                theme.scheme.on_surface_variant
            } else {
                theme.scheme.on_surface.scale_alpha(0.38)
            }));
        }

        let chip_content =
            container(content.spacing(8))
                .padding([8, 16])
                .style(move |_theme: &Theme| {
                    let (background_color, border_color) =
                        match (&variant, selected, enabled) {
                            (ChipVariant::Filter, true, true) => {
                                (theme.scheme.secondary_container, Color::TRANSPARENT)
                            }
                            (ChipVariant::Input, _, true) => (
                                theme.scheme.surface_container_low,
                                theme.scheme.outline,
                            ),
                            (ChipVariant::Assist, true, true) => {
                                (theme.scheme.surface_container_low, Color::TRANSPARENT)
                            }
                            (ChipVariant::Suggestion, true, true) => {
                                (theme.scheme.surface_container_low, Color::TRANSPARENT)
                            }
                            (_, false, true) => (Color::TRANSPARENT, theme.scheme.outline),
                            (_, _, false) => (
                                theme.scheme.on_surface.scale_alpha(0.12),
                                theme.scheme.on_surface.scale_alpha(0.12),
                            ),
                        };

                    container::Style {
                        background: Some(Background::Color(background_color)),
                        border: Border {
                            color: border_color,
                            width: if border_color == Color::TRANSPARENT {
                                0.0
                            } else {
                                1.0
                            },
                            radius: 8.0.into(),
                        },
                        ..Default::default()
                    }
                });

        if let Some(message) = on_press {
            button(chip_content)
                .on_press(message)
                .style(move |_theme: &Theme, status| {
                    let hover_color = match status {
                        button::Status::Hovered => {
                            Some(theme.scheme.on_surface.scale_alpha(0.08))
                        }
                        button::Status::Pressed => {
                            Some(theme.scheme.on_surface.scale_alpha(0.12))
                        }
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
