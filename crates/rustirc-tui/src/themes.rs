//! TUI color themes and styling
//!
//! Provides multiple color themes for the terminal interface including:
//! - Default dark theme
//! - Light theme
//! - High contrast theme
//! - Custom themes

use ratatui::style::Color;

/// Available theme names
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeName {
    Dark,
    Light,
    HighContrast,
    Monokai,
    Solarized,
}

impl ThemeName {
    pub fn all() -> &'static [ThemeName] {
        &[
            ThemeName::Dark,
            ThemeName::Light,
            ThemeName::HighContrast,
            ThemeName::Monokai,
            ThemeName::Solarized,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeName::Dark => "Dark",
            ThemeName::Light => "Light",
            ThemeName::HighContrast => "High Contrast",
            ThemeName::Monokai => "Monokai",
            ThemeName::Solarized => "Solarized",
        }
    }
}

/// Colors for TUI theming
#[derive(Debug, Clone)]
pub struct TuiColors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub background: Color,
    pub text: Color,
    pub text_muted: Color,
    pub border: Color,
    pub border_focused: Color,
    pub highlight: Color,
    pub activity: Color,
    pub error: Color,
    pub success: Color,
    pub warning: Color,
    pub info: Color,
}

impl TuiColors {
    /// Default dark theme
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            background: Color::Black,
            text: Color::White,
            text_muted: Color::Gray,
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            highlight: Color::Red,
            activity: Color::Yellow,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            info: Color::Cyan,
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            background: Color::White,
            text: Color::Black,
            text_muted: Color::DarkGray,
            border: Color::Gray,
            border_focused: Color::Blue,
            highlight: Color::Red,
            activity: Color::Rgb(255, 140, 0), // Orange
            error: Color::Red,
            success: Color::Green,
            warning: Color::Rgb(255, 140, 0), // Orange
            info: Color::Blue,
        }
    }

    /// High contrast theme for accessibility
    pub fn high_contrast() -> Self {
        Self {
            primary: Color::White,
            secondary: Color::Yellow,
            accent: Color::Cyan,
            background: Color::Black,
            text: Color::White,
            text_muted: Color::Rgb(192, 192, 192), // Light gray
            border: Color::White,
            border_focused: Color::Yellow,
            highlight: Color::Red,
            activity: Color::Yellow,
            error: Color::Red,
            success: Color::Green,
            warning: Color::Yellow,
            info: Color::Cyan,
        }
    }

    /// Monokai theme
    pub fn monokai() -> Self {
        Self {
            primary: Color::Rgb(166, 226, 46),        // Green
            secondary: Color::Rgb(102, 217, 239),     // Cyan
            accent: Color::Rgb(249, 38, 114),         // Pink
            background: Color::Rgb(39, 40, 34),       // Dark gray
            text: Color::Rgb(248, 248, 242),          // Light gray
            text_muted: Color::Rgb(117, 113, 94),     // Muted gray
            border: Color::Rgb(73, 72, 62),           // Border gray
            border_focused: Color::Rgb(166, 226, 46), // Green
            highlight: Color::Rgb(249, 38, 114),      // Pink
            activity: Color::Rgb(253, 151, 31),       // Orange
            error: Color::Rgb(249, 38, 114),          // Pink
            success: Color::Rgb(166, 226, 46),        // Green
            warning: Color::Rgb(253, 151, 31),        // Orange
            info: Color::Rgb(102, 217, 239),          // Cyan
        }
    }

    /// Solarized dark theme
    pub fn solarized() -> Self {
        Self {
            primary: Color::Rgb(42, 161, 152),        // Cyan
            secondary: Color::Rgb(38, 139, 210),      // Blue
            accent: Color::Rgb(211, 54, 130),         // Magenta
            background: Color::Rgb(0, 43, 54),        // Base03
            text: Color::Rgb(131, 148, 150),          // Base0
            text_muted: Color::Rgb(88, 110, 117),     // Base01
            border: Color::Rgb(7, 54, 66),            // Base02
            border_focused: Color::Rgb(42, 161, 152), // Cyan
            highlight: Color::Rgb(220, 50, 47),       // Red
            activity: Color::Rgb(181, 137, 0),        // Yellow
            error: Color::Rgb(220, 50, 47),           // Red
            success: Color::Rgb(133, 153, 0),         // Green
            warning: Color::Rgb(203, 75, 22),         // Orange
            info: Color::Rgb(42, 161, 152),           // Cyan
        }
    }

    /// Get theme by name
    pub fn from_theme(theme: ThemeName) -> Self {
        match theme {
            ThemeName::Dark => Self::dark(),
            ThemeName::Light => Self::light(),
            ThemeName::HighContrast => Self::high_contrast(),
            ThemeName::Monokai => Self::monokai(),
            ThemeName::Solarized => Self::solarized(),
        }
    }
}

impl Default for TuiColors {
    fn default() -> Self {
        Self::dark()
    }
}

/// Theme manager for switching between themes
#[derive(Debug)]
pub struct ThemeManager {
    current_theme: ThemeName,
    colors: TuiColors,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current_theme: ThemeName::Dark,
            colors: TuiColors::dark(),
        }
    }

    pub fn with_theme(theme: ThemeName) -> Self {
        Self {
            current_theme: theme,
            colors: TuiColors::from_theme(theme),
        }
    }

    pub fn set_theme(&mut self, theme: ThemeName) {
        self.current_theme = theme;
        self.colors = TuiColors::from_theme(theme);
    }

    pub fn current_theme(&self) -> ThemeName {
        self.current_theme
    }

    pub fn colors(&self) -> &TuiColors {
        &self.colors
    }

    pub fn next_theme(&mut self) {
        let themes = ThemeName::all();
        let current_index = themes
            .iter()
            .position(|&t| t == self.current_theme)
            .unwrap_or(0);
        let next_index = (current_index + 1) % themes.len();
        self.set_theme(themes[next_index]);
    }

    pub fn previous_theme(&mut self) {
        let themes = ThemeName::all();
        let current_index = themes
            .iter()
            .position(|&t| t == self.current_theme)
            .unwrap_or(0);
        let prev_index = if current_index == 0 {
            themes.len() - 1
        } else {
            current_index - 1
        };
        self.set_theme(themes[prev_index]);
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}
