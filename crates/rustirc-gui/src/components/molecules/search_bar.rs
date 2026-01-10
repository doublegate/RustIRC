//! Material Design 3 Search Bar component

use iced::{
    widget::{button, container, row, text_input},
    Background, Border, Color, Element, Length, Renderer, Theme,
};

use crate::components::atoms::icon::MaterialIcon;
use crate::themes::material_design_3::MaterialTheme;

/// Material Design 3 Search Bar component
#[derive()]
pub struct MaterialSearchBar<'a, Message> {
    value: String,
    placeholder: String,
    theme: MaterialTheme,
    variant: SearchBarVariant,
    is_focused: bool,
    show_leading_icon: bool,
    show_trailing_icon: bool,
    width: Length,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    on_clear: Option<Message>,
    on_focus: Option<Message>,
    on_blur: Option<Message>,
}

/// Search bar variants
#[derive(Debug, Clone, PartialEq)]
pub enum SearchBarVariant {
    /// Docked search bar integrated with app bar
    Docked,
    /// Full-width search bar
    FullWidth,
}

impl<'a, Message: 'a + Clone> MaterialSearchBar<'a, Message> {
    pub fn new(placeholder: impl Into<String>, value: &str) -> Self {
        Self {
            value: value.to_string(),
            placeholder: placeholder.into(),
            theme: MaterialTheme::default(),
            variant: SearchBarVariant::Docked,
            is_focused: false,
            show_leading_icon: true,
            show_trailing_icon: true,
            width: Length::Fill,
            on_input: None,
            on_submit: None,
            on_clear: None,
            on_focus: None,
            on_blur: None,
        }
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn variant(mut self, variant: SearchBarVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.is_focused = focused;
        self
    }

    pub fn show_leading_icon(mut self, show: bool) -> Self {
        self.show_leading_icon = show;
        self
    }

    pub fn show_trailing_icon(mut self, show: bool) -> Self {
        self.show_trailing_icon = show;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn on_input<F>(mut self, f: F) -> Self
    where
        F: 'a + Fn(String) -> Message,
    {
        self.on_input = Some(Box::new(f));
        self
    }

    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    pub fn on_clear(mut self, message: Message) -> Self {
        self.on_clear = Some(message);
        self
    }

    pub fn on_focus(mut self, message: Message) -> Self {
        self.on_focus = Some(message);
        self
    }

    pub fn on_blur(mut self, message: Message) -> Self {
        self.on_blur = Some(message);
        self
    }

    pub fn view(self) -> Element<'a, Message, Theme, Renderer> {
        let mut search_content = row![].spacing(12);

        // Leading icon
        if self.show_leading_icon {
            let search_icon = MaterialIcon::new("🔍")
                .size(24.0)
                .color(if self.is_focused {
                    iced::Color::from(self.theme.scheme.primary)
                } else {
                    iced::Color::from(self.theme.scheme.on_surface_variant)
                })
                .view();

            search_content = search_content.push(container(search_icon).padding([0, 4]));
        }

        // Search input
        let mut search_input = text_input(&self.placeholder, &self.value)
            .width(Length::Fill)
            .size(16)
            .padding([0, 8]);

        if let Some(on_input_fn) = self.on_input {
            search_input = search_input.on_input(on_input_fn);
        }

        if let Some(on_submit_msg) = self.on_submit {
            search_input = search_input.on_submit(on_submit_msg);
        }

        let styled_input = search_input.style(move |_theme: &Theme, status| {
            let _is_focused = matches!(status, text_input::Status::Focused { .. });

            text_input::Style {
                background: Background::Color(Color::TRANSPARENT),
                border: Border::default(),
                icon: iced::Color::from(self.theme.scheme.on_surface_variant),
                placeholder: iced::Color::from(self.theme.scheme.on_surface_variant)
                    .scale_alpha(0.6),
                value: iced::Color::from(self.theme.scheme.on_surface),
                selection: iced::Color::from(self.theme.scheme.primary).scale_alpha(0.2),
            }
        });

        search_content = search_content.push(styled_input);

        // Trailing icon (clear button)
        if self.show_trailing_icon && !self.value.is_empty() {
            let clear_button = if let Some(clear_message) = &self.on_clear {
                let on_surface_color = iced::Color::from(self.theme.scheme.on_surface);
                button(
                    MaterialIcon::new("×")
                        .size(24.0)
                        .color(iced::Color::from(self.theme.scheme.on_surface_variant))
                        .view::<Message>(),
                )
                .on_press(clear_message.clone())
                .style(move |_theme: &Theme, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered => on_surface_color.scale_alpha(0.08),
                        button::Status::Pressed => on_surface_color.scale_alpha(0.12),
                        _ => Color::TRANSPARENT,
                    })),
                    border: Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                })
                .padding(4)
            } else {
                button(
                    MaterialIcon::new("×")
                        .size(24.0)
                        .color(
                            iced::Color::from(self.theme.scheme.on_surface_variant)
                                .scale_alpha(0.38),
                        )
                        .view::<Message>(),
                )
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                })
                .padding(4)
            };

            search_content = search_content.push(clear_button);
        }

        let search_height = match self.variant {
            SearchBarVariant::Docked => 48.0,
            SearchBarVariant::FullWidth => 56.0,
        };

        container(search_content)
            .width(self.width)
            .height(Length::Fixed(search_height))
            .padding([0, 16])
            .center_y(Length::Fill)
            .style(move |_theme: &Theme| {
                let background_color = match self.variant {
                    SearchBarVariant::Docked => {
                        if self.is_focused {
                            self.theme.scheme.surface_container_high
                        } else {
                            self.theme.scheme.surface_container
                        }
                    }
                    SearchBarVariant::FullWidth => self.theme.scheme.surface_container,
                };

                let border_color = if self.is_focused {
                    self.theme.scheme.primary
                } else {
                    self.theme.scheme.outline
                };

                let border_width = if self.is_focused { 2.0 } else { 1.0 };

                container::Style {
                    background: Some(Background::Color(iced::Color::from(background_color))),
                    border: Border {
                        color: iced::Color::from(border_color),
                        width: border_width,
                        radius: match self.variant {
                            SearchBarVariant::Docked => 24.0.into(),
                            SearchBarVariant::FullWidth => 28.0.into(),
                        },
                    },
                    shadow: if self.is_focused {
                        iced::Shadow {
                            color: Color::BLACK.scale_alpha(0.1),
                            offset: iced::Vector::new(0.0, 2.0),
                            blur_radius: 4.0,
                        }
                    } else {
                        iced::Shadow::default()
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}
