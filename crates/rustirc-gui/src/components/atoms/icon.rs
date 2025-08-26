//! Material Design 3 Icon component

use iced::{
    widget::text,
    Element, Color, Theme, Renderer,
    font::{self, Font},
};

use crate::themes::material_design_3::MaterialTheme;

/// Material Design 3 Icon component
#[derive(Debug, Clone)]
pub struct MaterialIcon {
    icon: String,
    size: f32,
    color: Color,
    font: Font,
}

impl MaterialIcon {
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            size: 24.0,
            color: Color::BLACK,
            font: Font::DEFAULT,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn from_theme(mut self, theme: &MaterialTheme, variant: IconVariant) -> Self {
        self.color = match variant {
            IconVariant::Primary => theme.scheme.primary,
            IconVariant::OnSurface => theme.scheme.on_surface,
            IconVariant::OnSurfaceVariant => theme.scheme.on_surface_variant,
            IconVariant::OnPrimary => theme.scheme.on_primary,
            IconVariant::OnSecondary => theme.scheme.on_secondary,
            IconVariant::OnTertiary => theme.scheme.on_tertiary,
            IconVariant::OnError => theme.scheme.on_error,
            IconVariant::Outline => theme.scheme.outline,
        };
        self
    }

    pub fn view<Message>(self) -> Element<'static, Message, Theme, Renderer> {
        text(self.icon)
            .size(self.size)
            .color(self.color)
            .font(self.font)
            .into()
    }
}

/// Icon color variants based on Material Design 3 color roles
#[derive(Debug, Clone, PartialEq)]
pub enum IconVariant {
    Primary,
    OnSurface,
    OnSurfaceVariant,
    OnPrimary,
    OnSecondary,
    OnTertiary,
    OnError,
    Outline,
}

// Common Material Design icons as constants
pub mod icons {
    pub const HOME: &str = "🏠";
    pub const MENU: &str = "☰";
    pub const CLOSE: &str = "×";
    pub const BACK: &str = "←";
    pub const FORWARD: &str = "→";
    pub const UP: &str = "↑";
    pub const DOWN: &str = "↓";
    pub const SEARCH: &str = "🔍";
    pub const SETTINGS: &str = "⚙";
    pub const USER: &str = "👤";
    pub const MESSAGE: &str = "💬";
    pub const NOTIFICATION: &str = "🔔";
    pub const STAR: &str = "⭐";
    pub const HEART: &str = "❤";
    pub const PLUS: &str = "+";
    pub const MINUS: &str = "-";
    pub const CHECK: &str = "✓";
    pub const CROSS: &str = "✗";
    pub const INFO: &str = "ℹ";
    pub const WARNING: &str = "⚠";
    pub const ERROR: &str = "⚠";
    pub const SUCCESS: &str = "✓";
    
    // IRC specific icons
    pub const CONNECT: &str = "🔗";
    pub const DISCONNECT: &str = "🔌";
    pub const CHANNEL: &str = "#";
    pub const PRIVATE_MESSAGE: &str = "📧";
    pub const VOICE: &str = "🎤";
    pub const OP: &str = "@";
    pub const AWAY: &str = "💤";
    pub const ONLINE: &str = "🟢";
    pub const OFFLINE: &str = "🔴";
}