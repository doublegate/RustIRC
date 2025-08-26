//! Material Design 3 List Item component

use iced::{
    widget::{container, row, column, text, button},
    Element, Length, Background, Border, Color, Theme, Renderer,
    alignment::{Horizontal, Vertical},
};

use crate::themes::material_design_3::MaterialTheme;
use crate::components::atoms::icon::MaterialIcon;

/// Material Design 3 List Item component
#[derive(Debug, Clone)]
pub struct MaterialListItem<Message> {
    primary_text: String,
    secondary_text: Option<String>,
    tertiary_text: Option<String>,
    leading_content: Option<ListLeading>,
    trailing_content: Option<ListTrailing<Message>>,
    theme: MaterialTheme,
    variant: ListItemVariant,
    enabled: bool,
    selected: bool,
    on_press: Option<Message>,
}

/// List item variants
#[derive(Debug, Clone, PartialEq)]
pub enum ListItemVariant {
    /// Single line item
    OneLine,
    /// Two line item  
    TwoLine,
    /// Three line item
    ThreeLine,
}

/// Leading content for list items
#[derive(Debug, Clone)]
pub enum ListLeading {
    /// Icon
    Icon(String),
    /// Avatar with text
    Avatar(String),
    /// Checkbox state
    Checkbox(bool),
    /// Radio button state
    Radio(bool),
    /// Custom content placeholder
    Custom,
}

/// Trailing content for list items
#[derive(Debug, Clone)]
pub enum ListTrailing<Message> {
    /// Text label
    Text(String),
    /// Icon button
    IconButton { icon: String, on_press: Message },
    /// Switch state
    Switch(bool),
    /// Checkbox with action
    Checkbox { checked: bool, on_toggle: Message },
    /// Menu indicator
    Menu,
}

impl<Message: Clone> MaterialListItem<Message> {
    pub fn new(primary_text: impl Into<String>) -> Self {
        Self {
            primary_text: primary_text.into(),
            secondary_text: None,
            tertiary_text: None,
            leading_content: None,
            trailing_content: None,
            theme: MaterialTheme::default(),
            variant: ListItemVariant::OneLine,
            enabled: true,
            selected: false,
            on_press: None,
        }
    }

    pub fn secondary_text(mut self, text: impl Into<String>) -> Self {
        self.secondary_text = Some(text.into());
        if matches!(self.variant, ListItemVariant::OneLine) {
            self.variant = ListItemVariant::TwoLine;
        }
        self
    }

    pub fn tertiary_text(mut self, text: impl Into<String>) -> Self {
        self.tertiary_text = Some(text.into());
        self.variant = ListItemVariant::ThreeLine;
        self
    }

    pub fn leading_content(mut self, leading: ListLeading) -> Self {
        self.leading_content = Some(leading);
        self
    }

    pub fn trailing_content(mut self, trailing: ListTrailing<Message>) -> Self {
        self.trailing_content = Some(trailing);
        self
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn view(self) -> Element<'static, Message, Theme, Renderer> {
        let mut content_row = row![]
                        .spacing(16);

        // Leading content
        if let Some(leading) = &self.leading_content {
            let leading_element = match leading {
                ListLeading::Icon(icon) => {
                    MaterialIcon::new(icon)
                        .size(24.0)
                        .color(if self.enabled {
                            self.theme.scheme.on_surface_variant
                        } else {
                            self.theme.scheme.on_surface.scale_alpha(0.38)
                        })
                        .view()
                },
                ListLeading::Avatar(initial) => {
                    container(
                        text(initial)
                            .size(16)
                            .color(self.theme.scheme.on_primary_container)
                    )
                    .width(Length::Fixed(40.0))
                    .height(Length::Fixed(40.0))
                    .center_x()
                    .center_y()
                    .style(move |_theme: &Theme| container::Style {
                        background: Some(Background::Color(self.theme.scheme.primary_container)),
                        border: Border {
                            radius: 20.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .into()
                },
                ListLeading::Checkbox(checked) => {
                    // TODO: Replace with proper checkbox when available
                    container(
                        text(if *checked { "☑" } else { "☐" })
                            .size(20)
                            .color(if *checked {
                                self.theme.scheme.primary
                            } else {
                                self.theme.scheme.on_surface_variant
                            })
                    )
                    .into()
                },
                ListLeading::Radio(selected) => {
                    // TODO: Replace with proper radio button when available
                    container(
                        text(if *selected { "●" } else { "○" })
                            .size(20)
                            .color(if *selected {
                                self.theme.scheme.primary
                            } else {
                                self.theme.scheme.on_surface_variant
                            })
                    )
                    .into()
                },
                ListLeading::Custom => {
                    // Placeholder for custom content
                    container(text(""))
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(40.0))
                        .into()
                },
            };

            content_row = content_row.push(leading_element);
        }

        // Text content
        let mut text_content = column![]
            .spacing(2);

        // Primary text
        text_content = text_content.push(
            text(&self.primary_text)
                .size(16)
                .color(if self.enabled {
                    self.theme.scheme.on_surface
                } else {
                    self.theme.scheme.on_surface.scale_alpha(0.38)
                })
        );

        // Secondary text
        if let Some(secondary) = &self.secondary_text {
            text_content = text_content.push(
                text(secondary)
                    .size(14)
                    .color(if self.enabled {
                        self.theme.scheme.on_surface_variant
                    } else {
                        self.theme.scheme.on_surface.scale_alpha(0.38)
                    })
            );
        }

        // Tertiary text
        if let Some(tertiary) = &self.tertiary_text {
            text_content = text_content.push(
                text(tertiary)
                    .size(12)
                    .color(if self.enabled {
                        self.theme.scheme.on_surface_variant
                    } else {
                        self.theme.scheme.on_surface.scale_alpha(0.38)
                    })
            );
        }

        content_row = content_row.push(
            container(text_content)
                .width(Length::Fill)
        );

        // Trailing content
        if let Some(trailing) = &self.trailing_content {
            let trailing_element = match trailing {
                ListTrailing::Text(text_val) => {
                    text(text_val)
                        .size(14)
                        .color(self.theme.scheme.on_surface_variant)
                        .into()
                },
                ListTrailing::IconButton { icon, on_press } => {
                    button(
                        MaterialIcon::new(icon)
                            .size(24.0)
                            .color(self.theme.scheme.on_surface_variant)
                            .view()
                    )
                    .on_press(on_press.clone())
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        border: Border::default(),
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    })
                    .into()
                },
                ListTrailing::Switch(enabled) => {
                    // TODO: Replace with proper switch when available
                    container(
                        text(if *enabled { "🔛" } else { "🔘" })
                            .size(20)
                            .color(if *enabled {
                                self.theme.scheme.primary
                            } else {
                                self.theme.scheme.outline
                            })
                    )
                    .into()
                },
                ListTrailing::Checkbox { checked, on_toggle } => {
                    button(
                        text(if *checked { "☑" } else { "☐" })
                            .size(20)
                            .color(if *checked {
                                self.theme.scheme.primary
                            } else {
                                self.theme.scheme.on_surface_variant
                            })
                    )
                    .on_press(on_toggle.clone())
                    .style(|_theme: &Theme, _status| button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        border: Border::default(),
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    })
                    .into()
                },
                ListTrailing::Menu => {
                    MaterialIcon::new("⋮")
                        .size(24.0)
                        .color(self.theme.scheme.on_surface_variant)
                        .view()
                },
            };

            content_row = content_row.push(trailing_element);
        }

        let item_height = match self.variant {
            ListItemVariant::OneLine => 56.0,
            ListItemVariant::TwoLine => 72.0,
            ListItemVariant::ThreeLine => 88.0,
        };

        let list_item_container = container(content_row)
            .width(Length::Fill)
            .height(Length::Fixed(item_height))
            .padding([0, 16])
            .center_y()
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(if self.selected {
                    self.theme.scheme.secondary_container.scale_alpha(0.08)
                } else {
                    Color::TRANSPARENT
                })),
                border: Border::default(),
                ..Default::default()
            });

        if let Some(message) = self.on_press {
            button(list_item_container)
                .on_press(message)
                .width(Length::Fill)
                .style(move |_theme: &Theme, status| {
                    let hover_color = match status {
                        button::Status::Hovered => Some(self.theme.scheme.on_surface.scale_alpha(0.08)),
                        button::Status::Pressed => Some(self.theme.scheme.on_surface.scale_alpha(0.12)),
                        _ => None,
                    };

                    button::Style {
                        background: hover_color.map(Background::Color),
                        border: Border::default(),
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    }
                })
                .into()
        } else {
            list_item_container.into()
        }
    }
}