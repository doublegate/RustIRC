//! Theme management hook
//!
//! Manages theme state via CSS custom properties using a `data-theme` attribute.

use dioxus::prelude::*;

/// Available theme types matching the original 22 themes
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ThemeType {
    Dark,
    Light,
    Monokai,
    SolarizedDark,
    SolarizedLight,
    Dracula,
    Nord,
    GruvboxDark,
    GruvboxLight,
    OneDark,
    OneLight,
    TokyoNight,
    Catppuccin,
    Palenight,
    MaterialDark,
    MaterialLight,
    ArcDark,
    Cobalt2,
    Synthwave84,
    NightOwl,
    Ayu,
    Rosepine,
}

impl ThemeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Monokai => "monokai",
            Self::SolarizedDark => "solarized-dark",
            Self::SolarizedLight => "solarized-light",
            Self::Dracula => "dracula",
            Self::Nord => "nord",
            Self::GruvboxDark => "gruvbox-dark",
            Self::GruvboxLight => "gruvbox-light",
            Self::OneDark => "one-dark",
            Self::OneLight => "one-light",
            Self::TokyoNight => "tokyo-night",
            Self::Catppuccin => "catppuccin",
            Self::Palenight => "palenight",
            Self::MaterialDark => "material-dark",
            Self::MaterialLight => "material-light",
            Self::ArcDark => "arc-dark",
            Self::Cobalt2 => "cobalt2",
            Self::Synthwave84 => "synthwave84",
            Self::NightOwl => "night-owl",
            Self::Ayu => "ayu",
            Self::Rosepine => "rosepine",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::Monokai => "Monokai",
            Self::SolarizedDark => "Solarized Dark",
            Self::SolarizedLight => "Solarized Light",
            Self::Dracula => "Dracula",
            Self::Nord => "Nord",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::GruvboxLight => "Gruvbox Light",
            Self::OneDark => "One Dark",
            Self::OneLight => "One Light",
            Self::TokyoNight => "Tokyo Night",
            Self::Catppuccin => "Catppuccin",
            Self::Palenight => "Palenight",
            Self::MaterialDark => "Material Dark",
            Self::MaterialLight => "Material Light",
            Self::ArcDark => "Arc Dark",
            Self::Cobalt2 => "Cobalt2",
            Self::Synthwave84 => "Synthwave '84",
            Self::NightOwl => "Night Owl",
            Self::Ayu => "Ayu",
            Self::Rosepine => "Rose Pine",
        }
    }

    pub fn all() -> &'static [ThemeType] {
        &[
            Self::Dark,
            Self::Light,
            Self::Monokai,
            Self::SolarizedDark,
            Self::SolarizedLight,
            Self::Dracula,
            Self::Nord,
            Self::GruvboxDark,
            Self::GruvboxLight,
            Self::OneDark,
            Self::OneLight,
            Self::TokyoNight,
            Self::Catppuccin,
            Self::Palenight,
            Self::MaterialDark,
            Self::MaterialLight,
            Self::ArcDark,
            Self::Cobalt2,
            Self::Synthwave84,
            Self::NightOwl,
            Self::Ayu,
            Self::Rosepine,
        ]
    }
}

impl std::str::FromStr for ThemeType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "light" => Self::Light,
            "monokai" => Self::Monokai,
            "solarized-dark" | "solarized dark" => Self::SolarizedDark,
            "solarized-light" | "solarized light" => Self::SolarizedLight,
            "dracula" => Self::Dracula,
            "nord" => Self::Nord,
            "gruvbox-dark" | "gruvbox dark" => Self::GruvboxDark,
            "gruvbox-light" | "gruvbox light" => Self::GruvboxLight,
            "one-dark" | "one dark" => Self::OneDark,
            "one-light" | "one light" => Self::OneLight,
            "tokyo-night" | "tokyo night" => Self::TokyoNight,
            "catppuccin" => Self::Catppuccin,
            "palenight" => Self::Palenight,
            "material-dark" | "material dark" => Self::MaterialDark,
            "material-light" | "material light" => Self::MaterialLight,
            "arc-dark" | "arc dark" => Self::ArcDark,
            "cobalt2" => Self::Cobalt2,
            "synthwave84" | "synthwave '84" => Self::Synthwave84,
            "night-owl" | "night owl" => Self::NightOwl,
            "ayu" => Self::Ayu,
            "rosepine" | "rose pine" | "rose-pine" => Self::Rosepine,
            _ => Self::Dark,
        })
    }
}

/// Hook for theme management
pub fn use_theme() -> Signal<ThemeType> {
    use_context::<Signal<ThemeType>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_all_returns_22_themes() {
        assert_eq!(ThemeType::all().len(), 22);
    }

    #[test]
    fn test_theme_from_str_known() {
        assert_eq!("dark".parse::<ThemeType>().unwrap(), ThemeType::Dark);
        assert_eq!("light".parse::<ThemeType>().unwrap(), ThemeType::Light);
        assert_eq!("dracula".parse::<ThemeType>().unwrap(), ThemeType::Dracula);
        assert_eq!("nord".parse::<ThemeType>().unwrap(), ThemeType::Nord);
        assert_eq!(
            "tokyo-night".parse::<ThemeType>().unwrap(),
            ThemeType::TokyoNight
        );
        assert_eq!(
            "catppuccin".parse::<ThemeType>().unwrap(),
            ThemeType::Catppuccin
        );
    }

    #[test]
    fn test_theme_from_str_unknown_defaults_to_dark() {
        assert_eq!("nonexistent".parse::<ThemeType>().unwrap(), ThemeType::Dark);
    }

    #[test]
    fn test_theme_from_str_case_insensitive() {
        assert_eq!("DARK".parse::<ThemeType>().unwrap(), ThemeType::Dark);
        assert_eq!("Dracula".parse::<ThemeType>().unwrap(), ThemeType::Dracula);
        assert_eq!("NORD".parse::<ThemeType>().unwrap(), ThemeType::Nord);
    }

    #[test]
    fn test_theme_as_str_roundtrip() {
        for theme in ThemeType::all() {
            let s = theme.as_str();
            assert!(!s.is_empty());
            assert_eq!(&s.parse::<ThemeType>().unwrap(), theme);
        }
    }

    #[test]
    fn test_theme_display_name_nonempty() {
        for theme in ThemeType::all() {
            assert!(!theme.display_name().is_empty());
        }
    }
}
