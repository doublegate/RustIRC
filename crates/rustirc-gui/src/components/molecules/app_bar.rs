//! Material Design 3 App Bar components

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{button, container, row, text},
    Background, Border, Color, Element, Length, Renderer, Theme,
};

use crate::components::atoms::icon::MaterialIcon;
use crate::themes::material_design_3::MaterialTheme;

/// Material Design 3 App Bar component
#[derive(Debug, Clone)]
pub struct MaterialAppBar<Message> {
    title: String,
    theme: MaterialTheme,
    leading_icon: Option<AppBarAction<Message>>,
    trailing_actions: Vec<AppBarAction<Message>>,
    variant: AppBarVariant,
    height: f32,
}

/// App Bar variants following Material Design 3 guidelines
#[derive(Debug, Clone, PartialEq)]
pub enum AppBarVariant {
    /// Top app bar - primary navigation
    Top,
    /// Center-aligned top app bar
    CenterAligned,
    /// Small top app bar
    Small,
    /// Medium top app bar with more space
    Medium,
    /// Large top app bar with prominent title
    Large,
}

/// Action item for app bar
#[derive(Debug, Clone)]
pub struct AppBarAction<Message> {
    pub icon: String,
    pub label: Option<String>,
    pub on_press: Option<Message>,
}

impl<Message: Clone> MaterialAppBar<Message> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            theme: MaterialTheme::default(),
            leading_icon: None,
            trailing_actions: Vec::new(),
            variant: AppBarVariant::Top,
            height: 64.0,
        }
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn leading_icon(mut self, action: AppBarAction<Message>) -> Self {
        self.leading_icon = Some(action);
        self
    }

    pub fn add_action(mut self, action: AppBarAction<Message>) -> Self {
        self.trailing_actions.push(action);
        self
    }

    pub fn variant(mut self, variant: AppBarVariant) -> Self {
        self.variant = variant.clone();

        // Adjust height based on variant
        self.height = match variant {
            AppBarVariant::Small => 48.0,
            AppBarVariant::Top | AppBarVariant::CenterAligned => 64.0,
            AppBarVariant::Medium => 112.0,
            AppBarVariant::Large => 152.0,
        };

        self
    }

    pub fn view(self) -> Element<'static, Message, Theme, Renderer> {
        let mut content = row![];

        // Leading icon
        if let Some(leading) = &self.leading_icon {
            let leading_button = if let Some(message) = &leading.on_press {
                button(
                    MaterialIcon::new(&leading.icon)
                        .size(24.0)
                        .color(self.theme.scheme.on_surface.into())
                        .view(),
                )
                .on_press(message.clone())
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                })
                .into()
            } else {
                MaterialIcon::new(&leading.icon)
                    .size(24.0)
                    .color(self.theme.scheme.on_surface.into())
                    .view()
            };

            content = content.push(container(leading_button).padding(16).center_y(Length::Fill));
        }

        // Title
        let title_element = match self.variant {
            AppBarVariant::CenterAligned => container(
                text(&self.title)
                    .size(22)
                    .color(self.theme.scheme.on_surface.into()),
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
            AppBarVariant::Large => container(
                text(&self.title)
                    .size(32)
                    .color(self.theme.scheme.on_surface.into()),
            )
            .width(Length::Fill)
            .padding([24, 16])
            .align_y(iced::alignment::Vertical::Bottom),
            AppBarVariant::Medium => container(
                text(&self.title)
                    .size(28)
                    .color(self.theme.scheme.on_surface.into()),
            )
            .width(Length::Fill)
            .padding([16, 16])
            .align_y(iced::alignment::Vertical::Bottom),
            _ => container(
                text(&self.title)
                    .size(22)
                    .color(self.theme.scheme.on_surface.into()),
            )
            .width(Length::Fill)
            .center_y(Length::Fill)
            .padding([0, 16]),
        };

        content = content.push(title_element);

        // Trailing actions
        let mut actions_row = row![].spacing(8);
        for action in &self.trailing_actions {
            let action_button = if let Some(message) = &action.on_press {
                button(
                    MaterialIcon::new(&action.icon)
                        .size(24.0)
                        .color(self.theme.scheme.on_surface.into())
                        .view(),
                )
                .on_press(message.clone())
                .style(|_theme: &Theme, status| button::Style {
                    background: Some(Background::Color(match status {
                        button::Status::Hovered => self.theme.scheme.on_surface.into().scale_alpha(0.08),
                        button::Status::Pressed => self.theme.scheme.on_surface.into().scale_alpha(0.12),
                        _ => Color::TRANSPARENT,
                    })),
                    border: Border {
                        radius: 20.0.into(),
                        ..Default::default()
                    },
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                })
                .padding(8)
            } else {
                button(
                    MaterialIcon::new(&action.icon)
                        .size(24.0)
                        .color(self.theme.scheme.on_surface.into().scale_alpha(0.38))
                        .view(),
                )
                .style(|_theme: &Theme, _status| button::Style {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    border: Border::default(),
                    shadow: iced::Shadow::default(),
                    ..Default::default()
                })
                .padding(8)
            };

            actions_row = actions_row.push(action_button);
        }

        if !self.trailing_actions.is_empty() {
            content = content.push(container(actions_row).padding([0, 16]).center_y(Length::Fill));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fixed(self.height))
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(match self.variant {
                    AppBarVariant::Small | AppBarVariant::Top | AppBarVariant::CenterAligned => {
                        self.theme.scheme.surface.into()
                    }
                    AppBarVariant::Medium | AppBarVariant::Large => {
                        self.theme.scheme.surface.into()_container
                    }
                })),
                border: Border::default(),
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.1),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            })
            .into()
    }
}

impl<Message> AppBarAction<Message> {
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: None,
            on_press: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }
}
