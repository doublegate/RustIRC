//! Theme system for RustIRC GUI
//!
//! Provides comprehensive theming support with multiple built-in themes
//! and customization options for colors, fonts, and spacing.

use iced::{font, Color, Border, Background};
use serde::{Deserialize, Serialize};

/// Available theme types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeType {
    DefaultLight,
    DefaultDark,
    SolarizedLight,
    SolarizedDark,
    Dracula,
    Nord,
    Monokai,
    GitHubLight,
    MaterialLight,
}

impl Default for ThemeType {
    fn default() -> Self {
        ThemeType::DefaultDark
    }
}

/// Color palette for themes
#[derive(Debug, Clone)]
pub struct ColorPalette {
    // Background colors
    pub background: Color,
    pub surface: Color,
    pub surface_variant: Color,
    
    // Text colors
    pub on_background: Color,
    pub on_surface: Color,
    pub on_surface_variant: Color,
    
    // Primary colors
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    
    // Secondary colors
    pub secondary: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
    
    // Error colors
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,
    
    // Border colors
    pub outline: Color,
    pub outline_variant: Color,
    
    // IRC-specific colors
    pub server_online: Color,
    pub server_offline: Color,
    pub channel_active: Color,
    pub channel_inactive: Color,
    pub highlight: Color,
    pub mention: Color,
    pub timestamp: Color,
    pub nickname: Color,
    pub action: Color,
    pub notice: Color,
    pub join_part: Color,
    pub mode_change: Color,
    
    // IRC color codes (mIRC colors)
    pub irc_colors: [Color; 16],
}

/// Typography configuration
#[derive(Debug, Clone)]
pub struct Typography {
    pub default_font: font::Family,
    pub monospace_font: font::Family,
    pub ui_font_size: f32,
    pub message_font_size: f32,
    pub small_font_size: f32,
    pub large_font_size: f32,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            default_font: font::Family::SansSerif,
            monospace_font: font::Family::Monospace,
            ui_font_size: 14.0,
            message_font_size: 13.0,
            small_font_size: 11.0,
            large_font_size: 16.0,
        }
    }
}

/// Spacing configuration
#[derive(Debug, Clone)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
        }
    }
}

/// Complete theme configuration
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub theme_type: ThemeType,
    pub colors: ColorPalette,
    pub typography: Typography,
    pub spacing: Spacing,
}

impl Theme {
    pub fn from_type(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::DefaultLight => Self::default_light(),
            ThemeType::DefaultDark => Self::default_dark(),
            ThemeType::SolarizedLight => Self::solarized_light(),
            ThemeType::SolarizedDark => Self::solarized_dark(),
            ThemeType::Dracula => Self::dracula(),
            ThemeType::Nord => Self::nord(),
            ThemeType::Monokai => Self::monokai(),
            ThemeType::GitHubLight => Self::github_light(),
            ThemeType::MaterialLight => Self::material_light(),
        }
    }
    
    fn default_dark() -> Self {
        Self {
            name: "Default Dark".to_string(),
            theme_type: ThemeType::DefaultDark,
            colors: ColorPalette {
                background: Color::from_rgb(0.1, 0.1, 0.1),
                surface: Color::from_rgb(0.15, 0.15, 0.15),
                surface_variant: Color::from_rgb(0.2, 0.2, 0.2),
                
                on_background: Color::from_rgb(0.9, 0.9, 0.9),
                on_surface: Color::from_rgb(0.9, 0.9, 0.9),
                on_surface_variant: Color::from_rgb(0.7, 0.7, 0.7),
                
                primary: Color::from_rgb(0.4, 0.6, 1.0),
                on_primary: Color::from_rgb(0.0, 0.0, 0.0),
                primary_container: Color::from_rgb(0.1, 0.2, 0.4),
                on_primary_container: Color::from_rgb(0.8, 0.9, 1.0),
                
                secondary: Color::from_rgb(0.7, 0.7, 0.7),
                on_secondary: Color::from_rgb(0.0, 0.0, 0.0),
                secondary_container: Color::from_rgb(0.3, 0.3, 0.3),
                on_secondary_container: Color::from_rgb(0.9, 0.9, 0.9),
                
                error: Color::from_rgb(1.0, 0.4, 0.4),
                on_error: Color::from_rgb(0.0, 0.0, 0.0),
                error_container: Color::from_rgb(0.4, 0.1, 0.1),
                on_error_container: Color::from_rgb(1.0, 0.8, 0.8),
                
                outline: Color::from_rgb(0.4, 0.4, 0.4),
                outline_variant: Color::from_rgb(0.3, 0.3, 0.3),
                
                server_online: Color::from_rgb(0.4, 0.8, 0.4),
                server_offline: Color::from_rgb(0.8, 0.4, 0.4),
                channel_active: Color::from_rgb(0.4, 0.6, 1.0),
                channel_inactive: Color::from_rgb(0.5, 0.5, 0.5),
                highlight: Color::from_rgb(1.0, 1.0, 0.3),
                mention: Color::from_rgb(1.0, 0.3, 0.3),
                timestamp: Color::from_rgb(0.6, 0.6, 0.6),
                nickname: Color::from_rgb(0.4, 0.8, 1.0),
                action: Color::from_rgb(1.0, 0.4, 1.0),
                notice: Color::from_rgb(0.4, 0.8, 0.4),
                join_part: Color::from_rgb(0.6, 0.6, 0.6),
                mode_change: Color::from_rgb(1.0, 0.6, 0.2),
                
                irc_colors: [
                    Color::from_rgb(1.0, 1.0, 1.0), // white
                    Color::from_rgb(0.0, 0.0, 0.0), // black
                    Color::from_rgb(0.0, 0.0, 0.8), // blue
                    Color::from_rgb(0.0, 0.8, 0.0), // green
                    Color::from_rgb(1.0, 0.0, 0.0), // red
                    Color::from_rgb(0.8, 0.4, 0.0), // brown
                    Color::from_rgb(0.8, 0.0, 0.8), // purple
                    Color::from_rgb(1.0, 0.6, 0.0), // orange
                    Color::from_rgb(1.0, 1.0, 0.0), // yellow
                    Color::from_rgb(0.0, 1.0, 0.0), // light green
                    Color::from_rgb(0.0, 0.8, 0.8), // cyan
                    Color::from_rgb(0.0, 1.0, 1.0), // light cyan
                    Color::from_rgb(0.0, 0.0, 1.0), // light blue
                    Color::from_rgb(1.0, 0.0, 1.0), // pink
                    Color::from_rgb(0.5, 0.5, 0.5), // grey
                    Color::from_rgb(0.8, 0.8, 0.8), // light grey
                ],
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
        }
    }
    
    fn default_light() -> Self {
        let mut theme = Self::default_dark();
        theme.name = "Default Light".to_string();
        theme.theme_type = ThemeType::DefaultLight;
        
        // Swap colors for light theme
        let colors = &mut theme.colors;
        colors.background = Color::from_rgb(1.0, 1.0, 1.0);
        colors.surface = Color::from_rgb(0.98, 0.98, 0.98);
        colors.surface_variant = Color::from_rgb(0.95, 0.95, 0.95);
        colors.on_background = Color::from_rgb(0.0, 0.0, 0.0);
        colors.on_surface = Color::from_rgb(0.0, 0.0, 0.0);
        colors.on_surface_variant = Color::from_rgb(0.3, 0.3, 0.3);
        
        theme
    }
    
    fn solarized_dark() -> Self {
        Self {
            name: "Solarized Dark".to_string(),
            theme_type: ThemeType::SolarizedDark,
            colors: ColorPalette {
                background: Color::from_rgb(0.0, 0.168, 0.212),
                surface: Color::from_rgb(0.027, 0.212, 0.258),
                surface_variant: Color::from_rgb(0.345, 0.431, 0.459),
                
                on_background: Color::from_rgb(0.514, 0.580, 0.588),
                on_surface: Color::from_rgb(0.576, 0.631, 0.631),
                on_surface_variant: Color::from_rgb(0.396, 0.482, 0.514),
                
                primary: Color::from_rgb(0.149, 0.545, 0.824),
                on_primary: Color::from_rgb(0.992, 0.964, 0.890),
                primary_container: Color::from_rgb(0.027, 0.212, 0.258),
                on_primary_container: Color::from_rgb(0.576, 0.631, 0.631),
                
                secondary: Color::from_rgb(0.522, 0.600, 0.000),
                on_secondary: Color::from_rgb(0.0, 0.168, 0.212),
                secondary_container: Color::from_rgb(0.345, 0.431, 0.459),
                on_secondary_container: Color::from_rgb(0.576, 0.631, 0.631),
                
                error: Color::from_rgb(0.863, 0.196, 0.184),
                on_error: Color::from_rgb(0.992, 0.964, 0.890),
                error_container: Color::from_rgb(0.027, 0.212, 0.258),
                on_error_container: Color::from_rgb(0.863, 0.196, 0.184),
                
                server_online: Color::from_rgb(0.522, 0.600, 0.000),
                server_offline: Color::from_rgb(0.863, 0.196, 0.184),
                channel_active: Color::from_rgb(0.149, 0.545, 0.824),
                channel_inactive: Color::from_rgb(0.396, 0.482, 0.514),
                highlight: Color::from_rgb(0.710, 0.537, 0.000),
                mention: Color::from_rgb(0.863, 0.196, 0.184),
                timestamp: Color::from_rgb(0.396, 0.482, 0.514),
                nickname: Color::from_rgb(0.149, 0.545, 0.824),
                action: Color::from_rgb(0.827, 0.212, 0.510),
                notice: Color::from_rgb(0.522, 0.600, 0.000),
                join_part: Color::from_rgb(0.345, 0.431, 0.459),
                mode_change: Color::from_rgb(0.710, 0.537, 0.000),
                
                outline: Color::from_rgb(0.396, 0.482, 0.514),
                outline_variant: Color::from_rgb(0.345, 0.431, 0.459),
                
                irc_colors: [
                    Color::from_rgb(0.992, 0.964, 0.890),
                    Color::from_rgb(0.0, 0.168, 0.212),
                    Color::from_rgb(0.149, 0.545, 0.824),
                    Color::from_rgb(0.522, 0.600, 0.000),
                    Color::from_rgb(0.863, 0.196, 0.184),
                    Color::from_rgb(0.710, 0.537, 0.000),
                    Color::from_rgb(0.827, 0.212, 0.510),
                    Color::from_rgb(0.710, 0.537, 0.000),
                    Color::from_rgb(0.710, 0.537, 0.000),
                    Color::from_rgb(0.522, 0.600, 0.000),
                    Color::from_rgb(0.164, 0.631, 0.596),
                    Color::from_rgb(0.164, 0.631, 0.596),
                    Color::from_rgb(0.149, 0.545, 0.824),
                    Color::from_rgb(0.827, 0.212, 0.510),
                    Color::from_rgb(0.345, 0.431, 0.459),
                    Color::from_rgb(0.576, 0.631, 0.631),
                ],
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
        }
    }
    
    fn solarized_light() -> Self {
        let mut theme = Self::solarized_dark();
        theme.name = "Solarized Light".to_string();
        theme.theme_type = ThemeType::SolarizedLight;
        
        // Swap light and dark colors for light theme
        let colors = &mut theme.colors;
        std::mem::swap(&mut colors.background, &mut colors.on_background);
        std::mem::swap(&mut colors.surface, &mut colors.on_surface);
        std::mem::swap(&mut colors.surface_variant, &mut colors.on_surface_variant);
        
        theme
    }
    
    fn dracula() -> Self {
        Self {
            name: "Dracula".to_string(),
            theme_type: ThemeType::Dracula,
            colors: ColorPalette {
                background: Color::from_rgb(0.157, 0.165, 0.212),
                surface: Color::from_rgb(0.196, 0.204, 0.251),
                surface_variant: Color::from_rgb(0.235, 0.243, 0.290),
                
                on_background: Color::from_rgb(0.949, 0.949, 0.949),
                on_surface: Color::from_rgb(0.949, 0.949, 0.949),
                on_surface_variant: Color::from_rgb(0.741, 0.741, 0.741),
                
                primary: Color::from_rgb(0.741, 0.576, 0.976),
                on_primary: Color::from_rgb(0.157, 0.165, 0.212),
                primary_container: Color::from_rgb(0.196, 0.204, 0.251),
                on_primary_container: Color::from_rgb(0.741, 0.576, 0.976),
                
                secondary: Color::from_rgb(0.502, 0.859, 0.612),
                on_secondary: Color::from_rgb(0.157, 0.165, 0.212),
                secondary_container: Color::from_rgb(0.235, 0.243, 0.290),
                on_secondary_container: Color::from_rgb(0.502, 0.859, 0.612),
                
                error: Color::from_rgb(1.0, 0.337, 0.337),
                on_error: Color::from_rgb(0.157, 0.165, 0.212),
                error_container: Color::from_rgb(0.196, 0.204, 0.251),
                on_error_container: Color::from_rgb(1.0, 0.337, 0.337),
                
                server_online: Color::from_rgb(0.502, 0.859, 0.612),
                server_offline: Color::from_rgb(1.0, 0.337, 0.337),
                channel_active: Color::from_rgb(0.741, 0.576, 0.976),
                channel_inactive: Color::from_rgb(0.502, 0.502, 0.502),
                highlight: Color::from_rgb(1.0, 0.725, 0.424),
                mention: Color::from_rgb(1.0, 0.337, 0.337),
                timestamp: Color::from_rgb(0.502, 0.502, 0.502),
                nickname: Color::from_rgb(0.329, 0.914, 0.976),
                action: Color::from_rgb(1.0, 0.475, 0.776),
                notice: Color::from_rgb(0.502, 0.859, 0.612),
                join_part: Color::from_rgb(0.741, 0.741, 0.741),
                mode_change: Color::from_rgb(1.0, 0.725, 0.424),
                
                outline: Color::from_rgb(0.502, 0.502, 0.502),
                outline_variant: Color::from_rgb(0.235, 0.243, 0.290),
                
                irc_colors: [
                    Color::from_rgb(0.949, 0.949, 0.949),
                    Color::from_rgb(0.157, 0.165, 0.212),
                    Color::from_rgb(0.329, 0.914, 0.976),
                    Color::from_rgb(0.502, 0.859, 0.612),
                    Color::from_rgb(1.0, 0.337, 0.337),
                    Color::from_rgb(1.0, 0.725, 0.424),
                    Color::from_rgb(0.741, 0.576, 0.976),
                    Color::from_rgb(1.0, 0.725, 0.424),
                    Color::from_rgb(0.945, 0.980, 0.549),
                    Color::from_rgb(0.502, 0.859, 0.612),
                    Color::from_rgb(0.329, 0.914, 0.976),
                    Color::from_rgb(0.329, 0.914, 0.976),
                    Color::from_rgb(0.329, 0.914, 0.976),
                    Color::from_rgb(1.0, 0.475, 0.776),
                    Color::from_rgb(0.502, 0.502, 0.502),
                    Color::from_rgb(0.741, 0.741, 0.741),
                ],
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
        }
    }
    
    fn nord() -> Self {
        Self {
            name: "Nord".to_string(),
            theme_type: ThemeType::Nord,
            colors: ColorPalette {
                background: Color::from_rgb(0.180, 0.204, 0.251),
                surface: Color::from_rgb(0.231, 0.259, 0.322),
                surface_variant: Color::from_rgb(0.263, 0.298, 0.368),
                
                on_background: Color::from_rgb(0.925, 0.937, 0.957),
                on_surface: Color::from_rgb(0.925, 0.937, 0.957),
                on_surface_variant: Color::from_rgb(0.851, 0.878, 0.922),
                
                primary: Color::from_rgb(0.365, 0.506, 0.675),
                on_primary: Color::from_rgb(0.925, 0.937, 0.957),
                primary_container: Color::from_rgb(0.231, 0.259, 0.322),
                on_primary_container: Color::from_rgb(0.365, 0.506, 0.675),
                
                secondary: Color::from_rgb(0.635, 0.745, 0.549),
                on_secondary: Color::from_rgb(0.180, 0.204, 0.251),
                secondary_container: Color::from_rgb(0.263, 0.298, 0.368),
                on_secondary_container: Color::from_rgb(0.635, 0.745, 0.549),
                
                error: Color::from_rgb(0.749, 0.380, 0.415),
                on_error: Color::from_rgb(0.925, 0.937, 0.957),
                error_container: Color::from_rgb(0.231, 0.259, 0.322),
                on_error_container: Color::from_rgb(0.749, 0.380, 0.415),
                
                server_online: Color::from_rgb(0.635, 0.745, 0.549),
                server_offline: Color::from_rgb(0.749, 0.380, 0.415),
                channel_active: Color::from_rgb(0.365, 0.506, 0.675),
                channel_inactive: Color::from_rgb(0.298, 0.337, 0.415),
                highlight: Color::from_rgb(0.922, 0.796, 0.545),
                mention: Color::from_rgb(0.749, 0.380, 0.415),
                timestamp: Color::from_rgb(0.298, 0.337, 0.415),
                nickname: Color::from_rgb(0.365, 0.506, 0.675),
                action: Color::from_rgb(0.706, 0.556, 0.678),
                notice: Color::from_rgb(0.635, 0.745, 0.549),
                join_part: Color::from_rgb(0.851, 0.878, 0.922),
                mode_change: Color::from_rgb(0.922, 0.796, 0.545),
                
                outline: Color::from_rgb(0.298, 0.337, 0.415),
                outline_variant: Color::from_rgb(0.263, 0.298, 0.368),
                
                irc_colors: [
                    Color::from_rgb(0.925, 0.937, 0.957),
                    Color::from_rgb(0.180, 0.204, 0.251),
                    Color::from_rgb(0.365, 0.506, 0.675),
                    Color::from_rgb(0.635, 0.745, 0.549),
                    Color::from_rgb(0.749, 0.380, 0.415),
                    Color::from_rgb(0.922, 0.796, 0.545),
                    Color::from_rgb(0.706, 0.556, 0.678),
                    Color::from_rgb(0.922, 0.796, 0.545),
                    Color::from_rgb(0.922, 0.796, 0.545),
                    Color::from_rgb(0.635, 0.745, 0.549),
                    Color::from_rgb(0.513, 0.627, 0.757),
                    Color::from_rgb(0.513, 0.627, 0.757),
                    Color::from_rgb(0.365, 0.506, 0.675),
                    Color::from_rgb(0.706, 0.556, 0.678),
                    Color::from_rgb(0.298, 0.337, 0.415),
                    Color::from_rgb(0.851, 0.878, 0.922),
                ],
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
        }
    }
    
    fn monokai() -> Self {
        Self {
            name: "Monokai".to_string(),
            theme_type: ThemeType::Monokai,
            colors: ColorPalette {
                background: Color::from_rgb(0.157, 0.157, 0.118),
                surface: Color::from_rgb(0.196, 0.196, 0.157),
                surface_variant: Color::from_rgb(0.235, 0.235, 0.196),
                
                on_background: Color::from_rgb(0.973, 0.973, 0.949),
                on_surface: Color::from_rgb(0.973, 0.973, 0.949),
                on_surface_variant: Color::from_rgb(0.827, 0.827, 0.827),
                
                primary: Color::from_rgb(0.647, 0.886, 0.180),
                on_primary: Color::from_rgb(0.157, 0.157, 0.118),
                primary_container: Color::from_rgb(0.196, 0.196, 0.157),
                on_primary_container: Color::from_rgb(0.647, 0.886, 0.180),
                
                secondary: Color::from_rgb(0.980, 0.922, 0.169),
                on_secondary: Color::from_rgb(0.157, 0.157, 0.118),
                secondary_container: Color::from_rgb(0.235, 0.235, 0.196),
                on_secondary_container: Color::from_rgb(0.980, 0.922, 0.169),
                
                error: Color::from_rgb(0.973, 0.388, 0.573),
                on_error: Color::from_rgb(0.157, 0.157, 0.118),
                error_container: Color::from_rgb(0.196, 0.196, 0.157),
                on_error_container: Color::from_rgb(0.973, 0.388, 0.573),
                
                server_online: Color::from_rgb(0.647, 0.886, 0.180),
                server_offline: Color::from_rgb(0.973, 0.388, 0.573),
                channel_active: Color::from_rgb(0.404, 0.851, 0.937),
                channel_inactive: Color::from_rgb(0.502, 0.502, 0.502),
                highlight: Color::from_rgb(0.980, 0.922, 0.169),
                mention: Color::from_rgb(0.973, 0.388, 0.573),
                timestamp: Color::from_rgb(0.502, 0.502, 0.502),
                nickname: Color::from_rgb(0.404, 0.851, 0.937),
                action: Color::from_rgb(0.682, 0.506, 1.0),
                notice: Color::from_rgb(0.647, 0.886, 0.180),
                join_part: Color::from_rgb(0.827, 0.827, 0.827),
                mode_change: Color::from_rgb(0.980, 0.922, 0.169),
                
                outline: Color::from_rgb(0.502, 0.502, 0.502),
                outline_variant: Color::from_rgb(0.235, 0.235, 0.196),
                
                irc_colors: [
                    Color::from_rgb(0.973, 0.973, 0.949),
                    Color::from_rgb(0.157, 0.157, 0.118),
                    Color::from_rgb(0.404, 0.851, 0.937),
                    Color::from_rgb(0.647, 0.886, 0.180),
                    Color::from_rgb(0.973, 0.388, 0.573),
                    Color::from_rgb(0.980, 0.922, 0.169),
                    Color::from_rgb(0.682, 0.506, 1.0),
                    Color::from_rgb(0.992, 0.592, 0.122),
                    Color::from_rgb(0.980, 0.922, 0.169),
                    Color::from_rgb(0.647, 0.886, 0.180),
                    Color::from_rgb(0.404, 0.851, 0.937),
                    Color::from_rgb(0.404, 0.851, 0.937),
                    Color::from_rgb(0.404, 0.851, 0.937),
                    Color::from_rgb(0.682, 0.506, 1.0),
                    Color::from_rgb(0.502, 0.502, 0.502),
                    Color::from_rgb(0.827, 0.827, 0.827),
                ],
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
        }
    }
    
    fn github_light() -> Self {
        Self {
            name: "GitHub Light".to_string(),
            theme_type: ThemeType::GitHubLight,
            colors: ColorPalette {
                background: Color::from_rgb(1.0, 1.0, 1.0),
                surface: Color::from_rgb(0.969, 0.973, 0.976),
                surface_variant: Color::from_rgb(0.945, 0.953, 0.961),
                
                on_background: Color::from_rgb(0.137, 0.176, 0.220),
                on_surface: Color::from_rgb(0.137, 0.176, 0.220),
                on_surface_variant: Color::from_rgb(0.420, 0.482, 0.549),
                
                primary: Color::from_rgb(0.031, 0.384, 0.722),
                on_primary: Color::from_rgb(1.0, 1.0, 1.0),
                primary_container: Color::from_rgb(0.969, 0.973, 0.976),
                on_primary_container: Color::from_rgb(0.031, 0.384, 0.722),
                
                secondary: Color::from_rgb(0.102, 0.635, 0.322),
                on_secondary: Color::from_rgb(1.0, 1.0, 1.0),
                secondary_container: Color::from_rgb(0.945, 0.953, 0.961),
                on_secondary_container: Color::from_rgb(0.102, 0.635, 0.322),
                
                error: Color::from_rgb(0.827, 0.298, 0.298),
                on_error: Color::from_rgb(1.0, 1.0, 1.0),
                error_container: Color::from_rgb(0.969, 0.973, 0.976),
                on_error_container: Color::from_rgb(0.827, 0.298, 0.298),
                
                server_online: Color::from_rgb(0.102, 0.635, 0.322),
                server_offline: Color::from_rgb(0.827, 0.298, 0.298),
                channel_active: Color::from_rgb(0.031, 0.384, 0.722),
                channel_inactive: Color::from_rgb(0.420, 0.482, 0.549),
                highlight: Color::from_rgb(1.0, 0.957, 0.663),
                mention: Color::from_rgb(0.827, 0.298, 0.298),
                timestamp: Color::from_rgb(0.420, 0.482, 0.549),
                nickname: Color::from_rgb(0.031, 0.384, 0.722),
                action: Color::from_rgb(0.486, 0.322, 0.675),
                notice: Color::from_rgb(0.102, 0.635, 0.322),
                join_part: Color::from_rgb(0.420, 0.482, 0.549),
                mode_change: Color::from_rgb(0.933, 0.600, 0.133),
                
                outline: Color::from_rgb(0.420, 0.482, 0.549),
                outline_variant: Color::from_rgb(0.647, 0.694, 0.741),
                
                irc_colors: [
                    Color::from_rgb(1.0, 1.0, 1.0),
                    Color::from_rgb(0.137, 0.176, 0.220),
                    Color::from_rgb(0.031, 0.384, 0.722),
                    Color::from_rgb(0.102, 0.635, 0.322),
                    Color::from_rgb(0.827, 0.298, 0.298),
                    Color::from_rgb(0.933, 0.600, 0.133),
                    Color::from_rgb(0.486, 0.322, 0.675),
                    Color::from_rgb(0.933, 0.600, 0.133),
                    Color::from_rgb(1.0, 0.957, 0.663),
                    Color::from_rgb(0.102, 0.635, 0.322),
                    Color::from_rgb(0.110, 0.533, 0.690),
                    Color::from_rgb(0.110, 0.533, 0.690),
                    Color::from_rgb(0.031, 0.384, 0.722),
                    Color::from_rgb(0.486, 0.322, 0.675),
                    Color::from_rgb(0.420, 0.482, 0.549),
                    Color::from_rgb(0.647, 0.694, 0.741),
                ],
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
        }
    }
    
    fn material_light() -> Self {
        Self {
            name: "Material Light".to_string(),
            theme_type: ThemeType::MaterialLight,
            colors: ColorPalette {
                background: Color::from_rgb(1.0, 1.0, 1.0),
                surface: Color::from_rgb(0.976, 0.976, 0.976),
                surface_variant: Color::from_rgb(0.953, 0.953, 0.953),
                
                on_background: Color::from_rgb(0.122, 0.122, 0.122),
                on_surface: Color::from_rgb(0.122, 0.122, 0.122),
                on_surface_variant: Color::from_rgb(0.380, 0.380, 0.380),
                
                primary: Color::from_rgb(0.392, 0.584, 0.929),
                on_primary: Color::from_rgb(1.0, 1.0, 1.0),
                primary_container: Color::from_rgb(0.976, 0.976, 0.976),
                on_primary_container: Color::from_rgb(0.392, 0.584, 0.929),
                
                secondary: Color::from_rgb(0.298, 0.686, 0.314),
                on_secondary: Color::from_rgb(1.0, 1.0, 1.0),
                secondary_container: Color::from_rgb(0.953, 0.953, 0.953),
                on_secondary_container: Color::from_rgb(0.298, 0.686, 0.314),
                
                error: Color::from_rgb(0.957, 0.263, 0.212),
                on_error: Color::from_rgb(1.0, 1.0, 1.0),
                error_container: Color::from_rgb(0.976, 0.976, 0.976),
                on_error_container: Color::from_rgb(0.957, 0.263, 0.212),
                
                server_online: Color::from_rgb(0.298, 0.686, 0.314),
                server_offline: Color::from_rgb(0.957, 0.263, 0.212),
                channel_active: Color::from_rgb(0.392, 0.584, 0.929),
                channel_inactive: Color::from_rgb(0.380, 0.380, 0.380),
                highlight: Color::from_rgb(1.0, 0.922, 0.231),
                mention: Color::from_rgb(0.957, 0.263, 0.212),
                timestamp: Color::from_rgb(0.380, 0.380, 0.380),
                nickname: Color::from_rgb(0.392, 0.584, 0.929),
                action: Color::from_rgb(0.612, 0.153, 0.690),
                notice: Color::from_rgb(0.298, 0.686, 0.314),
                join_part: Color::from_rgb(0.380, 0.380, 0.380),
                mode_change: Color::from_rgb(1.0, 0.596, 0.0),
                
                outline: Color::from_rgb(0.380, 0.380, 0.380),
                outline_variant: Color::from_rgb(0.620, 0.620, 0.620),
                
                irc_colors: [
                    Color::from_rgb(1.0, 1.0, 1.0),
                    Color::from_rgb(0.122, 0.122, 0.122),
                    Color::from_rgb(0.392, 0.584, 0.929),
                    Color::from_rgb(0.298, 0.686, 0.314),
                    Color::from_rgb(0.957, 0.263, 0.212),
                    Color::from_rgb(1.0, 0.596, 0.0),
                    Color::from_rgb(0.612, 0.153, 0.690),
                    Color::from_rgb(1.0, 0.596, 0.0),
                    Color::from_rgb(1.0, 0.922, 0.231),
                    Color::from_rgb(0.298, 0.686, 0.314),
                    Color::from_rgb(0.0, 0.737, 0.831),
                    Color::from_rgb(0.0, 0.737, 0.831),
                    Color::from_rgb(0.392, 0.584, 0.929),
                    Color::from_rgb(0.612, 0.153, 0.690),
                    Color::from_rgb(0.380, 0.380, 0.380),
                    Color::from_rgb(0.620, 0.620, 0.620),
                ],
            },
            typography: Typography::default(),
            spacing: Spacing::default(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_dark()
    }
}

// Implement Iced theme traits for proper theming support
impl iced::widget::button::Catalog for Theme {
    type Class<'a> = iced::widget::button::Style;

    fn default<'a>() -> Self::Class<'a> {
        iced::widget::button::Style::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: iced::widget::button::Status) -> iced::widget::button::Style {
        let base = iced::widget::button::Style {
            background: Some(self.colors.primary.into()),
            text_color: self.colors.on_primary,
            border: iced::Border {
                color: self.colors.outline,
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow::default(),
        };

        match status {
            iced::widget::button::Status::Active => base,
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: Some(Color::from_rgb(
                    self.colors.primary.r * 0.9,
                    self.colors.primary.g * 0.9,
                    self.colors.primary.b * 0.9
                ).into()),
                ..base
            },
            iced::widget::button::Status::Pressed => iced::widget::button::Style {
                background: Some(Color::from_rgb(
                    self.colors.primary.r * 0.8,
                    self.colors.primary.g * 0.8,
                    self.colors.primary.b * 0.8
                ).into()),
                ..base
            },
            iced::widget::button::Status::Disabled => iced::widget::button::Style {
                background: Some(Color::from_rgb(0.3, 0.3, 0.3).into()),
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..base
            },
        }
    }
}

impl iced::widget::container::Catalog for Theme {
    type Class<'a> = iced::widget::container::Style;

    fn default<'a>() -> Self::Class<'a> {
        iced::widget::container::Style::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> iced::widget::container::Style {
        iced::widget::container::Style {
            text_color: Some(self.colors.on_surface),
            background: Some(self.colors.surface.into()),
            border: iced::Border {
                color: self.colors.outline,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: iced::Shadow::default(),
        }
    }
}

impl iced::widget::text_input::Catalog for Theme {
    type Class<'a> = iced::widget::text_input::Style;

    fn default<'a>() -> Self::Class<'a> {
        iced::widget::text_input::Style {
            background: iced::Background::Color(iced::Color::WHITE),
            border: iced::Border::default(),
            icon: iced::Color::BLACK,
            placeholder: iced::Color::from_rgb(0.6, 0.6, 0.6),
            value: iced::Color::BLACK,
            selection: iced::Color::from_rgb(0.0, 0.5, 1.0),
        }
    }

    fn style(&self, class: &Self::Class<'_>, status: iced::widget::text_input::Status) -> iced::widget::text_input::Style {
        let base = iced::widget::text_input::Style {
            background: self.colors.surface.into(),
            border: iced::Border {
                color: self.colors.outline,
                width: 1.0,
                radius: 4.0.into(),
            },
            icon: self.colors.on_surface,
            placeholder: Color::from_rgb(0.6, 0.6, 0.6),
            value: self.colors.on_surface,
            selection: Color::from_rgb(0.3, 0.5, 1.0),
        };

        match status {
            iced::widget::text_input::Status::Active => base,
            iced::widget::text_input::Status::Hovered => iced::widget::text_input::Style {
                border: iced::Border {
                    color: self.colors.primary,
                    ..base.border
                },
                ..base
            },
            iced::widget::text_input::Status::Focused => iced::widget::text_input::Style {
                border: iced::Border {
                    color: self.colors.primary,
                    width: 2.0,
                    ..base.border
                },
                ..base
            },
            iced::widget::text_input::Status::Disabled => iced::widget::text_input::Style {
                background: Color::from_rgb(0.2, 0.2, 0.2).into(),
                value: Color::from_rgb(0.5, 0.5, 0.5),
                ..base
            },
        }
    }
}

// Application theme implementation
impl iced::application::DefaultStyle for Theme {
    fn default_style(&self) -> iced::application::Appearance {
        iced::application::Appearance {
            background_color: self.colors.background,
            text_color: self.colors.on_background,
        }
    }
}