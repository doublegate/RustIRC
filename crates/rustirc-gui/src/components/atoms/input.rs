//! Material Design 3 Input components

use iced::{
    widget::{column, container, text, text_input},
    Background, Border, Color, Element, Length, Renderer, Theme,
};

use crate::themes::material_design_3::MaterialTheme;

/// Material Design 3 Input component
#[derive()]
pub struct MaterialInput<'a, Message> {
    value: String,
    placeholder: String,
    label: Option<String>,
    helper_text: Option<String>,
    error_text: Option<String>,
    theme: MaterialTheme,
    width: Length,
    is_secure: bool,
    is_enabled: bool,
    variant: InputVariant,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
}

/// Material Design 3 Input variants
#[derive(Debug, Clone, PartialEq)]
pub enum InputVariant {
    /// Filled input field
    Filled,
    /// Outlined input field  
    Outlined,
}

impl<'a, Message: 'a + Clone> MaterialInput<'a, Message> {
    pub fn new(placeholder: impl Into<String>, value: &str) -> Self {
        Self {
            value: value.to_string(),
            placeholder: placeholder.into(),
            label: None,
            helper_text: None,
            error_text: None,
            theme: MaterialTheme::default(),
            width: Length::Fill,
            is_secure: false,
            is_enabled: true,
            variant: InputVariant::Outlined,
            on_input: None,
            on_submit: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn helper_text(mut self, helper_text: impl Into<String>) -> Self {
        self.helper_text = Some(helper_text.into());
        self
    }

    pub fn error_text(mut self, error_text: impl Into<String>) -> Self {
        self.error_text = Some(error_text.into());
        self
    }

    pub fn theme(mut self, theme: MaterialTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn secure(mut self, secure: bool) -> Self {
        self.is_secure = secure;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.is_enabled = enabled;
        self
    }

    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
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

    pub fn view(self) -> Element<'a, Message, Theme, Renderer> {
        // Extract all values we need to avoid lifetime issues
        let label = self.label.clone();
        let error_text = self.error_text.clone();
        let helper_text = self.helper_text.clone();
        let is_enabled = self.is_enabled;
        let theme = self.theme.clone();
        let placeholder = self.placeholder.clone();
        let value = self.value.clone();
        let width = self.width;
        let on_input = self.on_input;
        let on_submit = self.on_submit;
        let is_secure = self.is_secure;
        let variant = self.variant.clone();

        let mut input_column = column![];

        // Label
        if let Some(label_text) = label {
            input_column =
                input_column.push(text(label_text).size(12).color(if error_text.is_some() {
                    theme.scheme.error
                } else if is_enabled {
                    theme.scheme.on_surface_variant
                } else {
                    theme.scheme.on_surface.scale_alpha(0.38)
                }));
        }

        // Input field
        let mut input = text_input(&placeholder, &value)
            .width(width)
            .padding(16)
            .size(16);

        if let Some(on_input_fn) = on_input {
            input = input.on_input(on_input_fn);
        }

        if let Some(on_submit_msg) = on_submit {
            input = input.on_submit(on_submit_msg);
        }

        if is_secure {
            input = input.secure(true);
        }

        // Values for the styling closure
        let has_error = error_text.is_some();
        let theme_clone = theme.clone();

        let styled_input = input.style(move |_theme: &Theme, status| {
            let is_focused = matches!(status, text_input::Status::Focused { .. });

            let (background_color, border_color, border_width) =
                match (&variant, is_focused, has_error, is_enabled) {
                    (InputVariant::Filled, _, true, true) => (
                        theme_clone.scheme.error_container.scale_alpha(0.08).into(),
                        theme_clone.scheme.error.into(),
                        2.0,
                    ),
                    (InputVariant::Filled, true, false, true) => (
                        theme_clone.scheme.surface_container_highest.into(),
                        theme_clone.scheme.primary.into(),
                        2.0,
                    ),
                    (InputVariant::Filled, false, false, true) => (
                        theme_clone.scheme.surface_container_highest.into(),
                        theme_clone.scheme.on_surface_variant.into(),
                        1.0,
                    ),
                    (InputVariant::Filled, _, _, false) => (
                        theme_clone.scheme.on_surface.scale_alpha(0.04).into(),
                        theme_clone.scheme.on_surface.scale_alpha(0.38).into(),
                        1.0,
                    ),
                    (InputVariant::Outlined, _, true, true) => {
                        (Color::TRANSPARENT, theme_clone.scheme.error.into(), 2.0)
                    }
                    (InputVariant::Outlined, true, false, true) => {
                        (Color::TRANSPARENT, theme_clone.scheme.primary.into(), 2.0)
                    }
                    (InputVariant::Outlined, false, false, true) => {
                        (Color::TRANSPARENT, theme_clone.scheme.outline.into(), 1.0)
                    }
                    (InputVariant::Outlined, _, _, false) => (
                        Color::TRANSPARENT,
                        theme_clone.scheme.on_surface.scale_alpha(0.38).into(),
                        1.0,
                    ),
                };

            let _border_radius = match &variant {
                InputVariant::Filled => [4.0, 4.0, 0.0, 0.0],
                InputVariant::Outlined => [4.0, 4.0, 4.0, 4.0],
            };

            text_input::Style {
                background: Background::Color(background_color),
                border: Border {
                    color: border_color,
                    width: border_width,
                    radius: 4.0.into(),
                },
                icon: theme_clone.scheme.on_surface_variant.into(),
                placeholder: theme_clone
                    .scheme
                    .on_surface_variant
                    .scale_alpha(0.6)
                    .into(),
                value: if is_enabled {
                    theme_clone.scheme.on_surface.into()
                } else {
                    theme_clone.scheme.on_surface.scale_alpha(0.38).into()
                },
                selection: theme_clone.scheme.primary.scale_alpha(0.2).into(),
            }
        });

        input_column = input_column.push(styled_input);

        // Helper or error text
        if let Some(error) = error_text {
            input_column = input_column.push(
                text(error)
                    .size(12)
                    .color(iced::Color::from(theme.scheme.error)),
            );
        } else if let Some(helper) = helper_text {
            input_column = input_column.push(text(helper).size(12).color(if is_enabled {
                iced::Color::from(theme.scheme.on_surface_variant)
            } else {
                iced::Color::from(theme.scheme.on_surface.scale_alpha(0.38))
            }));
        }

        container(input_column.spacing(4)).width(width).into()
    }
}
