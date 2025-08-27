//! Material Design 3 Card component

use iced::{
    widget::container,
    Background, Border, Color, Element, Length, Renderer, Theme,
};

use crate::themes::material_design_3::{ElevationLevel, MaterialTheme};

/// Material Design 3 Card component
#[derive()]
pub struct MaterialCard<'a, Message> {
    content: Element<'a, Message, Theme, Renderer>,
    theme: MaterialTheme,
    elevation: ElevationLevel,
    width: Length,
    height: Length,
    padding: u16,
    on_press: Option<Message>,
}

impl<'a, Message> MaterialCard<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            theme: MaterialTheme::default(),
            elevation: ElevationLevel::Level1,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: 16,
            on_press: None,
        }
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn elevation(mut self, elevation: ElevationLevel) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    pub fn view(self) -> Element<'a, Message, Theme, Renderer> {
        let card_container = container(self.content)
            .width(self.width)
            .height(self.height)
            .padding(self.padding)
            .style(move |_theme: &Theme| {
                let surface_color = match self.elevation {
                    ElevationLevel::Level0 => self.theme.scheme.surface,
                    ElevationLevel::Level1 => self.theme.scheme.surface_container_low,
                    ElevationLevel::Level2 => self.theme.scheme.surface_container,
                    ElevationLevel::Level3 => self.theme.scheme.surface_container_high,
                    ElevationLevel::Level4 => self.theme.scheme.surface_container_highest,
                    ElevationLevel::Level5 => self.theme.scheme.surface_container_highest,
                };

                container::Style {
                    background: Some(Background::Color(iced::Color::from(surface_color))),
                    border: Border {
                        color: iced::Color::from(self.theme.scheme.outline_variant),
                        width: 1.0,
                        radius: 12.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::BLACK,
                        offset: iced::Vector::new(0.0, self.elevation as u8 as f32),
                        blur_radius: (self.elevation as u8 as f32) * 2.0,
                    },
                    ..Default::default()
                }
            });

        if let Some(message) = self.on_press {
            iced::widget::button(card_container)
                .on_press(message)
                .style(|_theme: &Theme, _status| iced::widget::button::Style {
                    background: None,
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                })
                .into()
        } else {
            card_container.into()
        }
    }
}
