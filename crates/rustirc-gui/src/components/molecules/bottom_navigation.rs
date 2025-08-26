//! Material Design 3 Bottom Navigation component

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, row, text},
    Background, Border, Color, Element, Length, Renderer, Theme,
};

use crate::components::atoms::icon::MaterialIcon;
use crate::themes::material_design_3::MaterialTheme;

/// Material Design 3 Bottom Navigation component
#[derive(Debug, Clone)]
pub struct MaterialBottomNavigation<Message> {
    items: Vec<BottomNavigationItem<Message>>,
    selected_index: usize,
    theme: MaterialTheme,
    show_labels: bool,
}

/// Bottom navigation item
#[derive(Debug, Clone)]
pub struct BottomNavigationItem<Message> {
    pub icon: String,
    pub selected_icon: Option<String>,
    pub label: String,
    pub badge: Option<String>,
    pub on_press: Message,
}

impl<Message: Clone> MaterialBottomNavigation<Message> {
    pub fn new(items: Vec<BottomNavigationItem<Message>>) -> Self {
        Self {
            items,
            selected_index: 0,
            theme: MaterialTheme::default(),
            show_labels: true,
        }
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.items.len().saturating_sub(1));
        self
    }

    pub fn show_labels(mut self, show_labels: bool) -> Self {
        self.show_labels = show_labels;
        self
    }

    pub fn view(self) -> Element<'static, Message, Theme, Renderer> {
        let mut nav_items = row![].spacing(0).align_y(iced::Alignment::Center);

        let item_width = Length::Fill;

        for (index, item) in self.items.iter().enumerate() {
            let is_selected = index == self.selected_index;

            // Icon
            let icon_text = if is_selected {
                item.selected_icon.as_ref().unwrap_or(&item.icon)
            } else {
                &item.icon
            };

            let icon_color = if is_selected {
                self.theme.scheme.on_secondary_container.into()
            } else {
                self.theme.scheme.on_surface_variant.into()
            };

            let mut item_content = column![].spacing(4);

            // Icon with badge
            let mut icon_container = container(
                MaterialIcon::new(icon_text)
                    .size(24.0)
                    .color(icon_color)
                    .build(),
            )
            .padding(4);

            if let Some(badge_text) = &item.badge {
                // TODO: Add badge overlay when available in Iced
                icon_container = icon_container;
            }

            item_content = item_content.push(icon_container);

            // Label
            if self.show_labels {
                let label_color = if is_selected {
                    self.theme.scheme.on_surface.into()
                } else {
                    self.theme.scheme.on_surface.into()_variant
                };

                item_content = item_content.push(text(&item.label).size(12).color(label_color));
            }

            // Navigation item button
            let nav_item = button(container(item_content).width(Length::Fill).center_x(Length::Fill))
                .on_press(item.on_press.clone())
                .width(item_width)
                .padding([12, 8])
                .style(move |_theme: &Theme, status| {
                    let background_color = if is_selected {
                        Some(Background::Color(self.theme.scheme.secondary_container))
                    } else {
                        match status {
                            button::Status::Hovered => Some(Background::Color(
                                self.theme.scheme.on_surface.into().scale_alpha(0.08),
                            )),
                            button::Status::Pressed => Some(Background::Color(
                                self.theme.scheme.on_surface.into().scale_alpha(0.12),
                            )),
                            _ => Some(Background::Color(Color::TRANSPARENT)),
                        }
                    };

                    button::Style {
                        background: background_color,
                        border: Border {
                            radius: 16.0.into(),
                            ..Default::default()
                        },
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    }
                });

            nav_items = nav_items.push(nav_item);
        }

        container(nav_items)
            .width(Length::Fill)
            .height(Length::Fixed(if self.show_labels { 80.0 } else { 64.0 }))
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(self.theme.scheme.surface.into()_container)),
                border: Border {
                    color: self.theme.scheme.outline_variant.into(),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.08),
                    offset: iced::Vector::new(0.0, -2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            })
            .into()
    }
}

impl<Message> BottomNavigationItem<Message> {
    pub fn new(icon: impl Into<String>, label: impl Into<String>, on_press: Message) -> Self {
        Self {
            icon: icon.into(),
            selected_icon: None,
            label: label.into(),
            badge: None,
            on_press,
        }
    }

    pub fn selected_icon(mut self, icon: impl Into<String>) -> Self {
        self.selected_icon = Some(icon.into());
        self
    }

    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}
