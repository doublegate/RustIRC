//! Theme system for RustIRC GUI
//!
//! Provides comprehensive theming support with multiple built-in themes
//! and customization options for colors, fonts, and spacing.

use iced::{font, Color};
use serde::{Deserialize, Serialize};

/// Available theme types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeType {
    #[default]
    MaterialDesign3,
    Dark,
    Light,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
}

/// Complete theme configuration
#[derive(Debug, Clone)]
pub struct Theme {
    pub theme_type: ThemeType,
    pub palette: ColorPalette,
    pub typography: Typography,
    pub spacing: Spacing,
}

impl Theme {
    /// Create theme from type
    pub fn from_type(theme_type: ThemeType) -> Self {
        Self {
            theme_type,
            palette: ColorPalette::from_type(theme_type),
            typography: Typography::default(),
            spacing: Spacing::default(),
        }
    }

    /// Get IRC color by code
    pub fn get_irc_color(&self, code: u8) -> Color {
        self.palette.get_irc_color(code)
    }

    /// Get primary color
    pub fn get_primary_color(&self) -> Color {
        self.palette.primary
    }

    /// Get text color
    pub fn get_text_color(&self) -> Color {
        self.palette.text_primary
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_type(ThemeType::default())
    }
}

/// Color palette for themes
#[derive(Debug, Clone)]
pub struct ColorPalette {
    // Base colors
    pub background: Color,
    pub surface: Color,
    pub surface_variant: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    // IRC specific colors
    pub nick_colors: Vec<Color>,
    pub irc_colors: [Color; 16],

    // Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // UI element colors
    pub border: Color,
    pub border_focused: Color,
    pub selection: Color,
    pub hover: Color,
}

impl ColorPalette {
    pub fn from_type(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::Dark => Self::dark(),
            ThemeType::Light => Self::light(),
            ThemeType::Dracula => Self::dracula(),
            ThemeType::Nord => Self::nord(),
            ThemeType::MaterialDesign3 => Self::material_design_3(),
            _ => Self::dark(), // Default fallback
        }
    }

    pub fn dark() -> Self {
        Self {
            background: Color::from_rgb(0.1, 0.1, 0.1),
            surface: Color::from_rgb(0.15, 0.15, 0.15),
            surface_variant: Color::from_rgb(0.2, 0.2, 0.2),
            primary: Color::from_rgb(0.4, 0.7, 1.0),
            secondary: Color::from_rgb(0.6, 0.6, 0.6),
            accent: Color::from_rgb(1.0, 0.4, 0.4),

            text_primary: Color::WHITE,
            text_secondary: Color::from_rgb(0.8, 0.8, 0.8),
            text_disabled: Color::from_rgb(0.5, 0.5, 0.5),

            nick_colors: vec![
                Color::from_rgb(1.0, 0.4, 0.4), // Red
                Color::from_rgb(0.4, 1.0, 0.4), // Green
                Color::from_rgb(0.4, 0.4, 1.0), // Blue
                Color::from_rgb(1.0, 1.0, 0.4), // Yellow
                Color::from_rgb(1.0, 0.4, 1.0), // Magenta
                Color::from_rgb(0.4, 1.0, 1.0), // Cyan
            ],

            irc_colors: [
                Color::WHITE,                   // 0: White
                Color::BLACK,                   // 1: Black
                Color::from_rgb(0.0, 0.0, 0.5), // 2: Blue
                Color::from_rgb(0.0, 0.5, 0.0), // 3: Green
                Color::from_rgb(1.0, 0.0, 0.0), // 4: Red
                Color::from_rgb(0.5, 0.0, 0.0), // 5: Brown
                Color::from_rgb(0.5, 0.0, 0.5), // 6: Purple
                Color::from_rgb(1.0, 0.5, 0.0), // 7: Orange
                Color::from_rgb(1.0, 1.0, 0.0), // 8: Yellow
                Color::from_rgb(0.0, 1.0, 0.0), // 9: Light Green
                Color::from_rgb(0.0, 0.5, 0.5), // 10: Cyan
                Color::from_rgb(0.0, 1.0, 1.0), // 11: Light Cyan
                Color::from_rgb(0.0, 0.0, 1.0), // 12: Light Blue
                Color::from_rgb(1.0, 0.0, 1.0), // 13: Pink
                Color::from_rgb(0.5, 0.5, 0.5), // 14: Grey
                Color::from_rgb(0.8, 0.8, 0.8), // 15: Light Grey
            ],

            success: Color::from_rgb(0.0, 0.8, 0.0),
            warning: Color::from_rgb(1.0, 0.6, 0.0),
            error: Color::from_rgb(1.0, 0.2, 0.2),
            info: Color::from_rgb(0.2, 0.6, 1.0),

            border: Color::from_rgb(0.3, 0.3, 0.3),
            border_focused: Color::from_rgb(0.4, 0.7, 1.0),
            selection: Color::from_rgba(0.4, 0.7, 1.0, 0.3),
            hover: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::WHITE,
            surface: Color::from_rgb(0.95, 0.95, 0.95),
            surface_variant: Color::from_rgb(0.9, 0.9, 0.9),
            primary: Color::from_rgb(0.2, 0.4, 0.8),
            secondary: Color::from_rgb(0.4, 0.4, 0.4),
            accent: Color::from_rgb(0.8, 0.2, 0.2),

            text_primary: Color::BLACK,
            text_secondary: Color::from_rgb(0.2, 0.2, 0.2),
            text_disabled: Color::from_rgb(0.5, 0.5, 0.5),

            nick_colors: vec![
                Color::from_rgb(0.8, 0.2, 0.2), // Red
                Color::from_rgb(0.2, 0.6, 0.2), // Green
                Color::from_rgb(0.2, 0.2, 0.8), // Blue
                Color::from_rgb(0.8, 0.6, 0.0), // Orange
                Color::from_rgb(0.6, 0.2, 0.8), // Purple
                Color::from_rgb(0.2, 0.6, 0.8), // Cyan
            ],

            irc_colors: [
                Color::BLACK,                   // 0: White (inverted for light theme)
                Color::WHITE,                   // 1: Black (inverted for light theme)
                Color::from_rgb(0.0, 0.0, 0.8), // 2: Blue
                Color::from_rgb(0.0, 0.6, 0.0), // 3: Green
                Color::from_rgb(0.8, 0.0, 0.0), // 4: Red
                Color::from_rgb(0.6, 0.3, 0.0), // 5: Brown
                Color::from_rgb(0.6, 0.0, 0.6), // 6: Purple
                Color::from_rgb(0.8, 0.4, 0.0), // 7: Orange
                Color::from_rgb(0.8, 0.8, 0.0), // 8: Yellow
                Color::from_rgb(0.0, 0.8, 0.0), // 9: Light Green
                Color::from_rgb(0.0, 0.6, 0.6), // 10: Cyan
                Color::from_rgb(0.0, 0.8, 0.8), // 11: Light Cyan
                Color::from_rgb(0.0, 0.0, 1.0), // 12: Light Blue
                Color::from_rgb(0.8, 0.0, 0.8), // 13: Pink
                Color::from_rgb(0.4, 0.4, 0.4), // 14: Grey
                Color::from_rgb(0.6, 0.6, 0.6), // 15: Light Grey
            ],

            success: Color::from_rgb(0.0, 0.6, 0.0),
            warning: Color::from_rgb(0.8, 0.5, 0.0),
            error: Color::from_rgb(0.8, 0.0, 0.0),
            info: Color::from_rgb(0.0, 0.4, 0.8),

            border: Color::from_rgb(0.7, 0.7, 0.7),
            border_focused: Color::from_rgb(0.2, 0.4, 0.8),
            selection: Color::from_rgba(0.2, 0.4, 0.8, 0.3),
            hover: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
        }
    }

    pub fn dracula() -> Self {
        Self {
            background: Color::from_rgb(0.16, 0.16, 0.21),
            surface: Color::from_rgb(0.19, 0.20, 0.25),
            surface_variant: Color::from_rgb(0.23, 0.24, 0.29),
            primary: Color::from_rgb(0.74, 0.58, 0.98),
            secondary: Color::from_rgb(0.63, 0.69, 0.75),
            accent: Color::from_rgb(1.0, 0.47, 0.78),

            text_primary: Color::from_rgb(0.95, 0.95, 0.95),
            text_secondary: Color::from_rgb(0.75, 0.75, 0.75),
            text_disabled: Color::from_rgb(0.5, 0.5, 0.5),

            nick_colors: vec![
                Color::from_rgb(1.0, 0.34, 0.33),  // Red
                Color::from_rgb(0.31, 0.98, 0.48), // Green
                Color::from_rgb(0.74, 0.58, 0.98), // Purple
                Color::from_rgb(1.0, 0.73, 0.42),  // Orange
                Color::from_rgb(0.50, 0.89, 1.0),  // Cyan
                Color::from_rgb(1.0, 0.47, 0.78),  // Pink
            ],

            irc_colors: [
                Color::from_rgb(0.95, 0.95, 0.95), // 0: White
                Color::from_rgb(0.16, 0.16, 0.21), // 1: Black
                Color::from_rgb(0.74, 0.58, 0.98), // 2: Blue
                Color::from_rgb(0.31, 0.98, 0.48), // 3: Green
                Color::from_rgb(1.0, 0.34, 0.33),  // 4: Red
                Color::from_rgb(1.0, 0.73, 0.42),  // 5: Brown
                Color::from_rgb(0.74, 0.58, 0.98), // 6: Purple
                Color::from_rgb(1.0, 0.73, 0.42),  // 7: Orange
                Color::from_rgb(0.95, 0.98, 0.31), // 8: Yellow
                Color::from_rgb(0.31, 0.98, 0.48), // 9: Light Green
                Color::from_rgb(0.50, 0.89, 1.0),  // 10: Cyan
                Color::from_rgb(0.50, 0.89, 1.0),  // 11: Light Cyan
                Color::from_rgb(0.74, 0.58, 0.98), // 12: Light Blue
                Color::from_rgb(1.0, 0.47, 0.78),  // 13: Pink
                Color::from_rgb(0.63, 0.69, 0.75), // 14: Grey
                Color::from_rgb(0.75, 0.75, 0.75), // 15: Light Grey
            ],

            success: Color::from_rgb(0.31, 0.98, 0.48),
            warning: Color::from_rgb(0.95, 0.98, 0.31),
            error: Color::from_rgb(1.0, 0.34, 0.33),
            info: Color::from_rgb(0.50, 0.89, 1.0),

            border: Color::from_rgb(0.35, 0.36, 0.42),
            border_focused: Color::from_rgb(0.74, 0.58, 0.98),
            selection: Color::from_rgba(0.74, 0.58, 0.98, 0.3),
            hover: Color::from_rgba(0.95, 0.95, 0.95, 0.1),
        }
    }

    pub fn nord() -> Self {
        Self {
            background: Color::from_rgb(0.18, 0.20, 0.25),
            surface: Color::from_rgb(0.21, 0.23, 0.29),
            surface_variant: Color::from_rgb(0.24, 0.27, 0.33),
            primary: Color::from_rgb(0.53, 0.75, 0.82),
            secondary: Color::from_rgb(0.60, 0.68, 0.75),
            accent: Color::from_rgb(0.75, 0.38, 0.42),

            text_primary: Color::from_rgb(0.93, 0.94, 0.96),
            text_secondary: Color::from_rgb(0.73, 0.75, 0.78),
            text_disabled: Color::from_rgb(0.53, 0.55, 0.58),

            nick_colors: vec![
                Color::from_rgb(0.75, 0.38, 0.42), // Red
                Color::from_rgb(0.64, 0.75, 0.54), // Green
                Color::from_rgb(0.53, 0.75, 0.82), // Blue
                Color::from_rgb(0.92, 0.80, 0.55), // Yellow
                Color::from_rgb(0.70, 0.56, 0.68), // Purple
                Color::from_rgb(0.55, 0.77, 0.71), // Cyan
            ],

            irc_colors: [
                Color::from_rgb(0.93, 0.94, 0.96), // 0: White
                Color::from_rgb(0.18, 0.20, 0.25), // 1: Black
                Color::from_rgb(0.53, 0.75, 0.82), // 2: Blue
                Color::from_rgb(0.64, 0.75, 0.54), // 3: Green
                Color::from_rgb(0.75, 0.38, 0.42), // 4: Red
                Color::from_rgb(0.85, 0.65, 0.45), // 5: Brown
                Color::from_rgb(0.70, 0.56, 0.68), // 6: Purple
                Color::from_rgb(0.85, 0.65, 0.45), // 7: Orange
                Color::from_rgb(0.92, 0.80, 0.55), // 8: Yellow
                Color::from_rgb(0.64, 0.75, 0.54), // 9: Light Green
                Color::from_rgb(0.55, 0.77, 0.71), // 10: Cyan
                Color::from_rgb(0.55, 0.77, 0.71), // 11: Light Cyan
                Color::from_rgb(0.53, 0.75, 0.82), // 12: Light Blue
                Color::from_rgb(0.70, 0.56, 0.68), // 13: Pink
                Color::from_rgb(0.53, 0.55, 0.58), // 14: Grey
                Color::from_rgb(0.73, 0.75, 0.78), // 15: Light Grey
            ],

            success: Color::from_rgb(0.64, 0.75, 0.54),
            warning: Color::from_rgb(0.92, 0.80, 0.55),
            error: Color::from_rgb(0.75, 0.38, 0.42),
            info: Color::from_rgb(0.53, 0.75, 0.82),

            border: Color::from_rgb(0.35, 0.38, 0.45),
            border_focused: Color::from_rgb(0.53, 0.75, 0.82),
            selection: Color::from_rgba(0.53, 0.75, 0.82, 0.3),
            hover: Color::from_rgba(0.93, 0.94, 0.96, 0.1),
        }
    }

    pub fn material_design_3() -> Self {
        // Material Design 3 color scheme - based on Material You design system
        Self {
            background: Color::from_rgb(0.11, 0.11, 0.118), // #1C1B1F - Surface
            surface: Color::from_rgb(0.16, 0.15, 0.19),     // #28272C - Surface Container
            surface_variant: Color::from_rgb(0.28, 0.27, 0.32), // #49454F - Surface Variant
            primary: Color::from_rgb(0.82, 0.69, 1.0),      // #D0BCFF - Primary
            secondary: Color::from_rgb(0.8, 0.78, 0.86),    // #CCC2DC - Secondary
            accent: Color::from_rgb(0.92, 0.73, 0.77),      // #EFBBC3 - Tertiary

            text_primary: Color::from_rgb(0.91, 0.89, 0.94), // #E6E1E5 - On Surface
            text_secondary: Color::from_rgb(0.79, 0.76, 0.81), // #CAC4D0 - On Surface Variant
            text_disabled: Color::from_rgb(0.56, 0.54, 0.58), // #8B8B93 - Outline

            // Material Design 3 harmonized nick colors
            nick_colors: vec![
                Color::from_rgb(0.96, 0.46, 0.48), // Error Red
                Color::from_rgb(0.77, 0.86, 0.45), // Success Green
                Color::from_rgb(0.82, 0.69, 1.0),  // Primary Purple
                Color::from_rgb(1.0, 0.86, 0.38),  // Warning Yellow
                Color::from_rgb(0.92, 0.73, 0.77), // Tertiary Pink
                Color::from_rgb(0.43, 0.85, 0.86), // Info Cyan
            ],

            // Standard IRC colors with Material Design 3 adjustments
            irc_colors: [
                Color::from_rgb(0.91, 0.89, 0.94),  // 0: White (On Surface)
                Color::from_rgb(0.11, 0.11, 0.118), // 1: Black (Surface)
                Color::from_rgb(0.42, 0.51, 0.93),  // 2: Blue
                Color::from_rgb(0.47, 0.67, 0.35),  // 3: Green
                Color::from_rgb(0.96, 0.46, 0.48),  // 4: Red
                Color::from_rgb(0.64, 0.35, 0.23),  // 5: Brown
                Color::from_rgb(0.65, 0.39, 0.80),  // 6: Purple
                Color::from_rgb(0.95, 0.55, 0.22),  // 7: Orange
                Color::from_rgb(1.0, 0.86, 0.38),   // 8: Yellow
                Color::from_rgb(0.77, 0.86, 0.45),  // 9: Light Green
                Color::from_rgb(0.43, 0.72, 0.73),  // 10: Cyan
                Color::from_rgb(0.43, 0.85, 0.86),  // 11: Light Cyan
                Color::from_rgb(0.67, 0.76, 0.98),  // 12: Light Blue
                Color::from_rgb(0.96, 0.68, 0.86),  // 13: Pink
                Color::from_rgb(0.48, 0.48, 0.48),  // 14: Grey
                Color::from_rgb(0.73, 0.73, 0.73),  // 15: Light Grey
            ],

            error: Color::from_rgb(0.96, 0.46, 0.48), // #F2B8B5 - Error
            warning: Color::from_rgb(1.0, 0.86, 0.38), // #FADB61 - Warning
            success: Color::from_rgb(0.77, 0.86, 0.45), // #C3DC73 - Success
            info: Color::from_rgb(0.67, 0.76, 0.98),  // #ABC2FA - Info

            border: Color::from_rgb(0.37, 0.35, 0.40), // #5E5C66 - Outline Variant
            border_focused: Color::from_rgb(0.82, 0.69, 1.0), // Primary
            selection: Color::from_rgba(0.82, 0.69, 1.0, 0.3), // Primary with alpha
            hover: Color::from_rgba(0.91, 0.89, 0.94, 0.08), // On Surface with low alpha
        }
    }

    pub fn get_irc_color(&self, code: u8) -> Color {
        self.irc_colors
            .get(code as usize % 16)
            .copied()
            .unwrap_or(self.text_primary)
    }
}

/// Typography settings
#[derive(Debug, Clone)]
pub struct Typography {
    pub default_font: font::Family,
    pub monospace_font: font::Family,
    pub default_size: f32,
    pub small_size: f32,
    pub large_size: f32,
    pub line_height: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            default_font: font::Family::SansSerif,
            monospace_font: font::Family::Monospace,
            default_size: 13.0,
            small_size: 11.0,
            large_size: 16.0,
            line_height: 1.4,
        }
    }
}

/// Spacing and layout settings
#[derive(Debug, Clone)]
pub struct Spacing {
    pub extra_small: f32,
    pub small: f32,
    pub medium: f32,
    pub large: f32,
    pub extra_large: f32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            extra_small: 2.0,
            small: 4.0,
            medium: 8.0,
            large: 16.0,
            extra_large: 32.0,
        }
    }
}
