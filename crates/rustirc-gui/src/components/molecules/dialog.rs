//! Material Design 3 Dialog components

use iced::{
    widget::{column, container, row, text},
    Background, Border, Color, Element, Length, Renderer, Theme,
};

use crate::components::atoms::button::{ButtonVariant, MaterialButton};
use crate::themes::material_design_3::MaterialTheme;

/// Material Design 3 Dialog component
#[derive()]
pub struct MaterialDialog<'a, Message> {
    title: Option<String>,
    content: Element<'a, Message, Theme, Renderer>,
    actions: Vec<DialogAction<Message>>,
    theme: MaterialTheme,
    variant: DialogVariant,
    dismissible: bool,
    width: Option<f32>,
    height: Option<f32>,
}

/// Dialog variants
#[derive(Debug, Clone, PartialEq)]
pub enum DialogVariant {
    /// Basic dialog for simple interactions
    Basic,
    /// Alert dialog for important messages
    Alert,
    /// Confirmation dialog for actions requiring confirmation
    Confirmation,
    /// Full-screen dialog for complex content
    FullScreen,
}

/// Dialog action button
#[derive()]
pub struct DialogAction<Message> {
    pub label: String,
    pub variant: ButtonVariant,
    pub on_press: Option<Message>,
}

impl<'a, Message: Clone + 'static> MaterialDialog<'a, Message> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            title: None,
            content: content.into(),
            actions: Vec::new(),
            theme: MaterialTheme::default(),
            variant: DialogVariant::Basic,
            dismissible: true,
            width: Some(280.0),
            height: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn variant(mut self, variant: DialogVariant) -> Self {
        self.variant = variant.clone();

        // Adjust default width based on variant
        self.width = match variant {
            DialogVariant::FullScreen => None,
            DialogVariant::Alert => Some(320.0),
            DialogVariant::Confirmation => Some(320.0),
            DialogVariant::Basic => Some(280.0),
        };

        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn add_action(mut self, action: DialogAction<Message>) -> Self {
        self.actions.push(action);
        self
    }

    pub fn view(self) -> Element<'a, Message, Theme, Renderer> {
        let mut dialog_content = column![].spacing(16);

        // Title
        if let Some(title) = self.title.clone() {
            dialog_content = dialog_content.push(
                text(title)
                    .size(24)
                    .color(iced::Color::from(self.theme.scheme.on_surface)),
            );
        }

        // Content
        dialog_content = dialog_content.push(self.content);

        // Actions
        if !self.actions.is_empty() {
            let mut actions_row = row![].spacing(8);

            for action in &self.actions {
                let action_button = if let Some(message) = &action.on_press {
                    MaterialButton::new(&action.label)
                        .variant(action.variant)
                        .on_press(message.clone())
                        .build()
                } else {
                    MaterialButton::new(&action.label)
                        .variant(ButtonVariant::Text)
                        .build()
                };

                actions_row = actions_row.push(action_button);
            }

            dialog_content = dialog_content.push(
                container(actions_row)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Right),
            );
        }

        let variant = self.variant.clone();
        let theme = self.theme.clone();

        let dialog_container =
            container(dialog_content)
                .padding(24)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(match variant {
                        DialogVariant::FullScreen => iced::Color::from(theme.scheme.surface),
                        _ => iced::Color::from(theme.scheme.surface_container_high),
                    })),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: match variant {
                            DialogVariant::FullScreen => 0.0.into(),
                            _ => 28.0.into(),
                        },
                    },
                    shadow: match variant {
                        DialogVariant::FullScreen => iced::Shadow::default(),
                        _ => iced::Shadow {
                            color: Color::BLACK.scale_alpha(0.15),
                            offset: iced::Vector::new(0.0, 8.0),
                            blur_radius: 16.0,
                        },
                    },
                    ..Default::default()
                });

        // Apply width and height constraints
        let sized_dialog = if let Some(width) = self.width {
            if let Some(height) = self.height {
                container(dialog_container)
                    .width(Length::Fixed(width))
                    .height(Length::Fixed(height))
            } else {
                container(dialog_container).width(Length::Fixed(width))
            }
        } else if let Some(height) = self.height {
            container(dialog_container).height(Length::Fixed(height))
        } else {
            dialog_container
        };

        // Center the dialog unless it's full screen
        if matches!(self.variant, DialogVariant::FullScreen) {
            sized_dialog.into()
        } else {
            container(sized_dialog)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Background::Color(Color::BLACK.scale_alpha(0.32))),
                    border: Border::default(),
                    ..Default::default()
                })
                .into()
        }
    }
}

impl<Message> DialogAction<Message> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Text,
            on_press: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// Creates a primary action (usually for confirmation)
    pub fn primary(label: impl Into<String>, message: Message) -> Self {
        Self::new(label)
            .variant(ButtonVariant::Filled)
            .on_press(message)
    }

    /// Creates a secondary action (usually for cancellation)
    pub fn secondary(label: impl Into<String>, message: Message) -> Self {
        Self::new(label)
            .variant(ButtonVariant::Text)
            .on_press(message)
    }
}
