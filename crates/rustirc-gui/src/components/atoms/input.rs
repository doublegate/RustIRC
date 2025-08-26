//! Material Design 3 Input components

use iced::{
    widget::{text_input, container, column, text},
    Element, Length, Background, Border, Color, Theme, Renderer,
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
        let mut input_column = column![];

        // Label
        if let Some(label) = &self.label {
            input_column = input_column.push(
                text(label)
                    .size(12)
                    .color(if self.error_text.is_some() {
                        self.theme.scheme.error
                    } else if self.is_enabled {
                        self.theme.scheme.on_surface_variant
                    } else {
                        self.theme.scheme.on_surface.scale_alpha(0.38)
                    })
            );
        }

        // Input field
        let mut input = text_input(&self.placeholder, &self.value)
            .width(self.width)
            .padding(16)
            .size(16);

        if let Some(on_input_fn) = self.on_input {
            input = input.on_input(on_input_fn);
        }

        if let Some(on_submit_msg) = self.on_submit {
            input = input.on_submit(on_submit_msg);
        }

        if self.is_secure {
            input = input.secure(true);
        }

        let styled_input = input.style(move |theme: &Theme, status| {
            let is_focused = matches!(status, text_input::Status::Focused);
            let is_error = self.error_text.is_some();

            let (background_color, border_color, border_width) = match (&self.variant, is_focused, is_error, self.is_enabled) {
                (InputVariant::Filled, _, true, true) => (
                    self.theme.scheme.error_container.scale_alpha(0.08),
                    self.theme.scheme.error,
                    2.0,
                ),
                (InputVariant::Filled, true, false, true) => (
                    self.theme.scheme.surface_container_highest,
                    self.theme.scheme.primary,
                    2.0,
                ),
                (InputVariant::Filled, false, false, true) => (
                    self.theme.scheme.surface_container_highest,
                    self.theme.scheme.on_surface_variant,
                    1.0,
                ),
                (InputVariant::Filled, _, _, false) => (
                    self.theme.scheme.on_surface.scale_alpha(0.04),
                    self.theme.scheme.on_surface.scale_alpha(0.38),
                    1.0,
                ),
                (InputVariant::Outlined, _, true, true) => (
                    Color::TRANSPARENT,
                    self.theme.scheme.error,
                    2.0,
                ),
                (InputVariant::Outlined, true, false, true) => (
                    Color::TRANSPARENT,
                    self.theme.scheme.primary,
                    2.0,
                ),
                (InputVariant::Outlined, false, false, true) => (
                    Color::TRANSPARENT,
                    self.theme.scheme.outline,
                    1.0,
                ),
                (InputVariant::Outlined, _, _, false) => (
                    Color::TRANSPARENT,
                    self.theme.scheme.on_surface.scale_alpha(0.38),
                    1.0,
                ),
            };

            let border_radius = match self.variant {
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
                icon: self.theme.scheme.on_surface_variant,
                placeholder: self.theme.scheme.on_surface_variant.scale_alpha(0.6),
                value: if self.is_enabled {
                    self.theme.scheme.on_surface
                } else {
                    self.theme.scheme.on_surface.scale_alpha(0.38)
                },
                selection: self.theme.scheme.primary.scale_alpha(0.2),
            }
        });

        input_column = input_column.push(styled_input);

        // Helper or error text
        if let Some(error) = &self.error_text {
            input_column = input_column.push(
                text(error)
                    .size(12)
                    .color(self.theme.scheme.error)
            );
        } else if let Some(helper) = &self.helper_text {
            input_column = input_column.push(
                text(helper)
                    .size(12)
                    .color(if self.is_enabled {
                        self.theme.scheme.on_surface_variant
                    } else {
                        self.theme.scheme.on_surface.scale_alpha(0.38)
                    })
            );
        }

        container(input_column.spacing(4))
            .width(self.width)
            .into()
    }
}