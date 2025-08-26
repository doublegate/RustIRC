//! Material Design 3 Theme System for RustIRC
//! 
//! This module implements a comprehensive Material Design 3 theming system
//! with proper color tokens, typography scale, and accessibility compliance.

use iced::{color, Color, Font};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Material Design 3 theme with full color system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialTheme {
    pub scheme: ColorScheme,
    pub typography: TypographyScale,
    pub elevation: ElevationSystem,
    pub spacing: SpacingScale,
    pub motion: MotionSystem,
    pub shapes: ShapeSystem,
}

/// Complete Material Design 3 color scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
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

    // Tertiary colors
    pub tertiary: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,

    // Error colors
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,

    // Surface colors
    pub surface: Color,
    pub on_surface: Color,
    pub surface_variant: Color,
    pub on_surface_variant: Color,
    pub surface_dim: Color,
    pub surface_bright: Color,
    pub surface_container_lowest: Color,
    pub surface_container_low: Color,
    pub surface_container: Color,
    pub surface_container_high: Color,
    pub surface_container_highest: Color,

    // Background
    pub background: Color,
    pub on_background: Color,

    // Outline
    pub outline: Color,
    pub outline_variant: Color,

    // Surface tints
    pub surface_tint: Color,
    
    // Inverse colors
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub inverse_primary: Color,

    // State colors
    pub hover_overlay: Color,
    pub focus_overlay: Color,
    pub pressed_overlay: Color,
    pub selected_overlay: Color,
    pub disabled_overlay: Color,

    // IRC-specific semantic colors
    pub nick_colors: Vec<Color>,
    pub mention_highlight: Color,
    pub unread_indicator: Color,
    pub typing_indicator: Color,
    pub connection_good: Color,
    pub connection_poor: Color,
    pub connection_lost: Color,
}

/// Material Design 3 typography scale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyScale {
    // Display styles
    pub display_large: TypographyToken,
    pub display_medium: TypographyToken, 
    pub display_small: TypographyToken,

    // Headline styles
    pub headline_large: TypographyToken,
    pub headline_medium: TypographyToken,
    pub headline_small: TypographyToken,

    // Title styles
    pub title_large: TypographyToken,
    pub title_medium: TypographyToken,
    pub title_small: TypographyToken,

    // Label styles
    pub label_large: TypographyToken,
    pub label_medium: TypographyToken,
    pub label_small: TypographyToken,

    // Body styles
    pub body_large: TypographyToken,
    pub body_medium: TypographyToken,
    pub body_small: TypographyToken,

    // Code styles (IRC-specific)
    pub code_large: TypographyToken,
    pub code_medium: TypographyToken,
    pub code_small: TypographyToken,
}

/// Typography token with font properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyToken {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub line_height: f32,
    pub letter_spacing: f32,
}

/// Font weight enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Regular = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

/// Material Design 3 elevation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationSystem {
    pub level0: ElevationToken,
    pub level1: ElevationToken,
    pub level2: ElevationToken,
    pub level3: ElevationToken,
    pub level4: ElevationToken,
    pub level5: ElevationToken,
}

/// Elevation token with shadow and tint properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationToken {
    pub shadow_color: Color,
    pub shadow_blur: f32,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,
    pub surface_tint_opacity: f32,
}

/// Elevation levels for Material Design 3 components
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ElevationLevel {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
}

/// Spacing scale based on 4px grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingScale {
    pub xs: f32,   // 4px
    pub sm: f32,   // 8px  
    pub md: f32,   // 16px
    pub lg: f32,   // 24px
    pub xl: f32,   // 32px
    pub xxl: f32,  // 48px
    pub xxxl: f32, // 64px
}

/// Material Design 3 motion system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionSystem {
    pub duration_short1: u64,  // 50ms
    pub duration_short2: u64,  // 100ms
    pub duration_medium1: u64, // 250ms
    pub duration_medium2: u64, // 300ms
    pub duration_long1: u64,   // 400ms
    pub duration_long2: u64,   // 500ms

    // Easing curves
    pub easing_standard: String,     // cubic-bezier(0.2, 0.0, 0, 1.0)
    pub easing_deceleration: String, // cubic-bezier(0.0, 0.0, 0, 1.0)
    pub easing_acceleration: String, // cubic-bezier(0.3, 0.0, 1.0, 1.0)
}

/// Shape system with corner radius tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeSystem {
    pub corner_none: f32,       // 0px
    pub corner_extra_small: f32, // 4px
    pub corner_small: f32,      // 8px
    pub corner_medium: f32,     // 12px
    pub corner_large: f32,      // 16px
    pub corner_extra_large: f32, // 28px
    pub corner_full: f32,       // 9999px (fully rounded)
}

impl Default for MaterialTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl MaterialTheme {
    /// Create a Material Design 3 dark theme
    pub fn dark() -> Self {
        Self {
            scheme: ColorScheme::dark(),
            typography: TypographyScale::default(),
            elevation: ElevationSystem::default(),
            spacing: SpacingScale::default(),
            motion: MotionSystem::default(),
            shapes: ShapeSystem::default(),
        }
    }

    /// Create a Material Design 3 light theme
    pub fn light() -> Self {
        Self {
            scheme: ColorScheme::light(),
            typography: TypographyScale::default(),
            elevation: ElevationSystem::default(),
            spacing: SpacingScale::default(),
            motion: MotionSystem::default(),
            shapes: ShapeSystem::default(),
        }
    }

    /// Create theme from seed color (dynamic color generation)
    pub fn from_seed_color(seed: Color, is_dark: bool) -> Self {
        let scheme = if is_dark {
            ColorScheme::from_seed_dark(seed)
        } else {
            ColorScheme::from_seed_light(seed)
        };

        Self {
            scheme,
            typography: TypographyScale::default(),
            elevation: ElevationSystem::default(),
            spacing: SpacingScale::default(),
            motion: MotionSystem::default(),
            shapes: ShapeSystem::default(),
        }
    }

    /// Get contrast ratio between two colors
    pub fn contrast_ratio(color1: Color, color2: Color) -> f32 {
        let l1 = Self::relative_luminance(color1);
        let l2 = Self::relative_luminance(color2);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Calculate relative luminance of a color (for WCAG compliance)
    fn relative_luminance(color: Color) -> f32 {
        let to_linear = |c: f32| {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };

        0.2126 * to_linear(color.r) +
        0.7152 * to_linear(color.g) +
        0.0722 * to_linear(color.b)
    }

    /// Check if color combination meets WCAG AA standard (4.5:1 contrast)
    pub fn meets_wcag_aa(foreground: Color, background: Color) -> bool {
        Self::contrast_ratio(foreground, background) >= 4.5
    }

    /// Check if color combination meets WCAG AAA standard (7:1 contrast)
    pub fn meets_wcag_aaa(foreground: Color, background: Color) -> bool {
        Self::contrast_ratio(foreground, background) >= 7.0
    }
}

impl ColorScheme {
    /// Create Material Design 3 dark color scheme
    pub fn dark() -> Self {
        Self {
            // Primary colors (Material Blue)
            primary: color!(0x6750A4),
            on_primary: color!(0xFFFFFF),
            primary_container: color!(0x4F378B),
            on_primary_container: color!(0xE9DDFF),

            // Secondary colors
            secondary: color!(0x625B71),
            on_secondary: color!(0xFFFFFF),
            secondary_container: color!(0x48454F),
            on_secondary_container: color!(0xE8DEF8),

            // Tertiary colors
            tertiary: color!(0x7D5260),
            on_tertiary: color!(0xFFFFFF),
            tertiary_container: color!(0x63394A),
            on_tertiary_container: color!(0xFFD8E4),

            // Error colors
            error: color!(0xF2B8B5),
            on_error: color!(0x601410),
            error_container: color!(0x8C1D18),
            on_error_container: color!(0xF9DEDC),

            // Surface colors
            surface: color!(0x141218),
            on_surface: color!(0xE6E0E9),
            surface_variant: color!(0x49454F),
            on_surface_variant: color!(0xCAC4D0),
            surface_dim: color!(0x141218),
            surface_bright: color!(0x3B383E),
            surface_container_lowest: color!(0x0F0D13),
            surface_container_low: color!(0x1D1B20),
            surface_container: color!(0x211F26),
            surface_container_high: color!(0x2B2930),
            surface_container_highest: color!(0x36343B),

            // Background
            background: color!(0x141218),
            on_background: color!(0xE6E0E9),

            // Outline
            outline: color!(0x938F99),
            outline_variant: color!(0x49454F),

            // Surface tint
            surface_tint: color!(0x6750A4),

            // Inverse colors
            inverse_surface: color!(0xE6E0E9),
            inverse_on_surface: color!(0x322F35),
            inverse_primary: color!(0x6750A4),

            // State colors (with proper opacity)
            hover_overlay: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            focus_overlay: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
            pressed_overlay: Color::from_rgba(1.0, 1.0, 1.0, 0.12),
            selected_overlay: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            disabled_overlay: Color::from_rgba(1.0, 1.0, 1.0, 0.38),

            // IRC-specific colors
            nick_colors: vec![
                color!(0xFF5722), // Deep Orange
                color!(0x4CAF50), // Green
                color!(0x2196F3), // Blue
                color!(0xFF9800), // Orange
                color!(0x9C27B0), // Purple
                color!(0x00BCD4), // Cyan
                color!(0xF44336), // Red
                color!(0x8BC34A), // Light Green
                color!(0x3F51B5), // Indigo
                color!(0xFFEB3B), // Yellow
            ],
            mention_highlight: color!(0xFFEB3B),
            unread_indicator: color!(0xFF5722),
            typing_indicator: color!(0x4CAF50),
            connection_good: color!(0x4CAF50),
            connection_poor: color!(0xFF9800),
            connection_lost: color!(0xF44336),
        }
    }

    /// Create Material Design 3 light color scheme
    pub fn light() -> Self {
        Self {
            // Primary colors
            primary: color!(0x6750A4),
            on_primary: color!(0xFFFFFF),
            primary_container: color!(0xE9DDFF),
            on_primary_container: color!(0x4F378B),

            // Secondary colors
            secondary: color!(0x625B71),
            on_secondary: color!(0xFFFFFF),
            secondary_container: color!(0xE8DEF8),
            on_secondary_container: color!(0x48454F),

            // Tertiary colors
            tertiary: color!(0x7D5260),
            on_tertiary: color!(0xFFFFFF),
            tertiary_container: color!(0xFFD8E4),
            on_tertiary_container: color!(0x63394A),

            // Error colors
            error: color!(0xBA1A1A),
            on_error: color!(0xFFFFFF),
            error_container: color!(0xFDDAD6),
            on_error_container: color!(0x410E0B),

            // Surface colors
            surface: color!(0xFEF7FF),
            on_surface: color!(0x1D1B20),
            surface_variant: color!(0xE7E0EB),
            on_surface_variant: color!(0x49454E),
            surface_dim: color!(0xDED8E1),
            surface_bright: color!(0xFEF7FF),
            surface_container_lowest: color!(0xFFFFFF),
            surface_container_low: color!(0xF7F2FA),
            surface_container: color!(0xF3EDF7),
            surface_container_high: color!(0xECE6F0),
            surface_container_highest: color!(0xE6E0E9),

            // Background
            background: color!(0xFEF7FF),
            on_background: color!(0x1D1B20),

            // Outline
            outline: color!(0x79757F),
            outline_variant: color!(0xCAC4D0),

            // Surface tint
            surface_tint: color!(0x6750A4),

            // Inverse colors
            inverse_surface: color!(0x322F35),
            inverse_on_surface: color!(0xF5EFF7),
            inverse_primary: color!(0xD0BCFF),

            // State colors
            hover_overlay: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
            focus_overlay: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
            pressed_overlay: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
            selected_overlay: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
            disabled_overlay: Color::from_rgba(0.0, 0.0, 0.0, 0.38),

            // IRC-specific colors (adjusted for light mode)
            nick_colors: vec![
                color!(0xD32F2F), // Red 700
                color!(0x388E3C), // Green 700
                color!(0x1976D2), // Blue 700
                color!(0xF57C00), // Orange 700
                color!(0x7B1FA2), // Purple 700
                color!(0x0097A7), // Cyan 700
                color!(0xC62828), // Red 800
                color!(0x689F38), // Light Green 700
                color!(0x303F9F), // Indigo 700
                color!(0xF9A825), // Yellow 700
            ],
            mention_highlight: color!(0xFFF59D),
            unread_indicator: color!(0xD32F2F),
            typing_indicator: color!(0x388E3C),
            connection_good: color!(0x4CAF50),
            connection_poor: color!(0xFF9800),
            connection_lost: color!(0xF44336),
        }
    }

    /// Generate color scheme from seed color (simplified version)
    pub fn from_seed_dark(seed: Color) -> Self {
        // This is a simplified implementation
        // In a full implementation, you would use the Material Color Utilities
        // to generate a complete palette from the seed color
        let mut dark_scheme = Self::dark();
        dark_scheme.primary = seed;
        // Adjust other colors based on the seed...
        dark_scheme
    }

    /// Generate light color scheme from seed color
    pub fn from_seed_light(seed: Color) -> Self {
        let mut light_scheme = Self::light();
        light_scheme.primary = seed;
        // Adjust other colors based on the seed...
        light_scheme
    }
}

impl Default for TypographyScale {
    fn default() -> Self {
        Self {
            // Display styles
            display_large: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 57.0,
                font_weight: FontWeight::Regular,
                line_height: 64.0,
                letter_spacing: -0.25,
            },
            display_medium: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 45.0,
                font_weight: FontWeight::Regular,
                line_height: 52.0,
                letter_spacing: 0.0,
            },
            display_small: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 36.0,
                font_weight: FontWeight::Regular,
                line_height: 44.0,
                letter_spacing: 0.0,
            },

            // Headline styles
            headline_large: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 32.0,
                font_weight: FontWeight::Regular,
                line_height: 40.0,
                letter_spacing: 0.0,
            },
            headline_medium: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 28.0,
                font_weight: FontWeight::Regular,
                line_height: 36.0,
                letter_spacing: 0.0,
            },
            headline_small: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 24.0,
                font_weight: FontWeight::Regular,
                line_height: 32.0,
                letter_spacing: 0.0,
            },

            // Title styles
            title_large: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 22.0,
                font_weight: FontWeight::Regular,
                line_height: 28.0,
                letter_spacing: 0.0,
            },
            title_medium: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 16.0,
                font_weight: FontWeight::Medium,
                line_height: 24.0,
                letter_spacing: 0.15,
            },
            title_small: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 14.0,
                font_weight: FontWeight::Medium,
                line_height: 20.0,
                letter_spacing: 0.1,
            },

            // Label styles  
            label_large: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 14.0,
                font_weight: FontWeight::Medium,
                line_height: 20.0,
                letter_spacing: 0.1,
            },
            label_medium: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Medium,
                line_height: 16.0,
                letter_spacing: 0.5,
            },
            label_small: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 11.0,
                font_weight: FontWeight::Medium,
                line_height: 16.0,
                letter_spacing: 0.5,
            },

            // Body styles
            body_large: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 16.0,
                font_weight: FontWeight::Regular,
                line_height: 24.0,
                letter_spacing: 0.5,
            },
            body_medium: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 14.0,
                font_weight: FontWeight::Regular,
                line_height: 20.0,
                letter_spacing: 0.25,
            },
            body_small: TypographyToken {
                font_family: "Inter".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Regular,
                line_height: 16.0,
                letter_spacing: 0.4,
            },

            // Code styles (monospace)
            code_large: TypographyToken {
                font_family: "JetBrains Mono".to_string(),
                font_size: 16.0,
                font_weight: FontWeight::Regular,
                line_height: 24.0,
                letter_spacing: 0.0,
            },
            code_medium: TypographyToken {
                font_family: "JetBrains Mono".to_string(),
                font_size: 14.0,
                font_weight: FontWeight::Regular,
                line_height: 20.0,
                letter_spacing: 0.0,
            },
            code_small: TypographyToken {
                font_family: "JetBrains Mono".to_string(),
                font_size: 12.0,
                font_weight: FontWeight::Regular,
                line_height: 16.0,
                letter_spacing: 0.0,
            },
        }
    }
}

impl Default for ElevationSystem {
    fn default() -> Self {
        Self {
            level0: ElevationToken {
                shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
                shadow_blur: 0.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 0.0,
                surface_tint_opacity: 0.0,
            },
            level1: ElevationToken {
                shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
                shadow_blur: 3.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 1.0,
                surface_tint_opacity: 0.05,
            },
            level2: ElevationToken {
                shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                shadow_blur: 6.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                surface_tint_opacity: 0.08,
            },
            level3: ElevationToken {
                shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                shadow_blur: 12.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 4.0,
                surface_tint_opacity: 0.11,
            },
            level4: ElevationToken {
                shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                shadow_blur: 16.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 6.0,
                surface_tint_opacity: 0.12,
            },
            level5: ElevationToken {
                shadow_color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                shadow_blur: 24.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 8.0,
                surface_tint_opacity: 0.14,
            },
        }
    }
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
            xxxl: 64.0,
        }
    }
}

impl Default for MotionSystem {
    fn default() -> Self {
        Self {
            duration_short1: 50,
            duration_short2: 100,
            duration_medium1: 250,
            duration_medium2: 300,
            duration_long1: 400,
            duration_long2: 500,
            easing_standard: "cubic-bezier(0.2, 0.0, 0, 1.0)".to_string(),
            easing_deceleration: "cubic-bezier(0.0, 0.0, 0, 1.0)".to_string(),
            easing_acceleration: "cubic-bezier(0.3, 0.0, 1.0, 1.0)".to_string(),
        }
    }
}

impl Default for ShapeSystem {
    fn default() -> Self {
        Self {
            corner_none: 0.0,
            corner_extra_small: 4.0,
            corner_small: 8.0,
            corner_medium: 12.0,
            corner_large: 16.0,
            corner_extra_large: 28.0,
            corner_full: 9999.0,
        }
    }
}