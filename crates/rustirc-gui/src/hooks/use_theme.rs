//! Hook for theme management and persistence

use crate::context::{ThemeState, ThemeType};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Theme management hook
#[allow(non_snake_case)]
pub fn use_theme() -> ThemeHook {
    let theme_state = use_context::<ThemeState>();
    
    ThemeHook { theme_state }
}

/// Theme hook interface
pub struct ThemeHook {
    pub theme_state: ThemeState,
}

impl ThemeHook {
    /// Get current theme
    pub fn current_theme(&self) -> ThemeType {
        self.theme_state.current_theme.read().clone()
    }

    /// Set theme
    pub fn set_theme(&self, theme: ThemeType) {
        self.theme_state.set_theme(theme);
        self.save_theme_preference(theme);
    }

    /// Cycle to next theme
    pub fn cycle_theme(&self) {
        let current = self.current_theme();
        let next = self.get_next_theme(current);
        self.set_theme(next);
    }

    /// Get all available themes
    pub fn get_available_themes(&self) -> Vec<ThemeInfo> {
        vec![
            ThemeInfo {
                theme_type: ThemeType::Dark,
                name: "Dark".to_string(),
                description: "Modern dark theme with Material Design colors".to_string(),
                is_dark: true,
            },
            ThemeInfo {
                theme_type: ThemeType::Light,
                name: "Light".to_string(),
                description: "Clean light theme with Material Design colors".to_string(),
                is_dark: false,
            },
            ThemeInfo {
                theme_type: ThemeType::Discord,
                name: "Discord".to_string(),
                description: "Discord-inspired dark theme".to_string(),
                is_dark: true,
            },
            ThemeInfo {
                theme_type: ThemeType::Nord,
                name: "Nord".to_string(),
                description: "Arctic-inspired color palette".to_string(),
                is_dark: true,
            },
            ThemeInfo {
                theme_type: ThemeType::MaterialDesign,
                name: "Material Design".to_string(),
                description: "Enhanced Material Design 3.0 theme".to_string(),
                is_dark: true,
            },
            ThemeInfo {
                theme_type: ThemeType::Dracula,
                name: "Dracula".to_string(),
                description: "Popular dark theme with vibrant colors".to_string(),
                is_dark: true,
            },
            ThemeInfo {
                theme_type: ThemeType::Catppuccin,
                name: "Catppuccin".to_string(),
                description: "Pastel theme with soothing colors".to_string(),
                is_dark: true,
            },
            ThemeInfo {
                theme_type: ThemeType::Terminal,
                name: "Terminal".to_string(),
                description: "Classic terminal-inspired theme".to_string(),
                is_dark: true,
            },
            ThemeInfo {
                theme_type: ThemeType::Slack,
                name: "Slack".to_string(),
                description: "Slack-inspired professional theme".to_string(),
                is_dark: false,
            },
        ]
    }

    /// Check if current theme is dark
    pub fn is_dark_theme(&self) -> bool {
        match self.current_theme() {
            ThemeType::Light | ThemeType::Slack => false,
            _ => true,
        }
    }

    /// Get theme info for current theme
    pub fn get_current_theme_info(&self) -> ThemeInfo {
        let current = self.current_theme();
        self.get_available_themes()
            .into_iter()
            .find(|info| info.theme_type == current)
            .unwrap_or_else(|| ThemeInfo {
                theme_type: current,
                name: format!("{:?}", current),
                description: "Custom theme".to_string(),
                is_dark: true,
            })
    }

    /// Apply custom CSS
    pub fn set_custom_css(&self, css: String) {
        self.theme_state.custom_css.set(css);
    }

    /// Get custom CSS
    pub fn get_custom_css(&self) -> String {
        self.theme_state.custom_css.read().clone()
    }

    fn get_next_theme(&self, current: ThemeType) -> ThemeType {
        match current {
            ThemeType::Dark => ThemeType::Light,
            ThemeType::Light => ThemeType::Discord,
            ThemeType::Discord => ThemeType::Nord,
            ThemeType::Nord => ThemeType::MaterialDesign,
            ThemeType::MaterialDesign => ThemeType::Dracula,
            ThemeType::Dracula => ThemeType::Catppuccin,
            ThemeType::Catppuccin => ThemeType::Terminal,
            ThemeType::Terminal => ThemeType::Slack,
            ThemeType::Slack => ThemeType::Dark,
        }
    }

    fn save_theme_preference(&self, theme: ThemeType) {
        // For desktop Dioxus apps, theme preferences would be saved to the file system
        // or OS-specific settings storage (Registry on Windows, plist on macOS, etc.)
        if let Ok(theme_json) = serde_json::to_string(&theme) {
            // This would save to a config file or app settings
            println!("Saving theme preference: {}", theme_json);
        }
    }
}

/// Theme information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub theme_type: ThemeType,
    pub name: String,
    pub description: String,
    pub is_dark: bool,
}

/// Hook for theme persistence
#[allow(non_snake_case)]
pub fn use_theme_persistence() {
    let theme_hook = use_theme();
    
    // Load theme from localStorage on mount
    use_effect(move || {
        load_saved_theme(&theme_hook);
        
        move || {
            // Cleanup if needed
        }
    });
}

/// Load theme from app settings (desktop version)
fn load_saved_theme(theme_hook: &ThemeHook) {
    // For desktop Dioxus apps, theme would be loaded from config file or app settings
    // This is a placeholder implementation that defaults to Dark theme
    let default_theme = ThemeType::Dark;
    theme_hook.set_theme(default_theme);
}

/// Hook for system theme detection (desktop version)
#[allow(non_snake_case)]
pub fn use_system_theme_detection() -> SystemThemeHook {
    let mut system_prefers_dark = use_signal(|| detect_system_dark_mode());
    
    // Set up system theme detection for desktop
    use_effect(move || {
        // For desktop apps, system theme detection would use OS-specific APIs
        // This is a simplified implementation
        system_prefers_dark.set(detect_system_dark_mode());
        
        move || {
            // Desktop cleanup if needed
        }
    });
    
    SystemThemeHook { system_prefers_dark }
}

/// System theme detection hook interface
pub struct SystemThemeHook {
    pub system_prefers_dark: Signal<bool>,
}

impl SystemThemeHook {
    /// Check if system prefers dark mode
    pub fn prefers_dark(&self) -> bool {
        self.system_prefers_dark()
    }

    /// Get recommended theme based on system preference
    pub fn get_recommended_theme(&self) -> ThemeType {
        if self.prefers_dark() {
            ThemeType::Dark
        } else {
            ThemeType::Light
        }
    }

    /// Auto-apply system theme
    pub fn apply_system_theme(&self) {
        let theme_hook = use_theme();
        let recommended = self.get_recommended_theme();
        theme_hook.set_theme(recommended);
    }
}

/// Detect system dark mode preference (desktop version)
fn detect_system_dark_mode() -> bool {
    // For desktop apps, this would check OS theme settings
    // On Linux: check GTK theme, GNOME/KDE settings
    // On macOS: check NSApp.effectiveAppearance
    // On Windows: check Registry HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize
    
    // This is a simplified implementation that defaults to dark mode
    true // Most developers prefer dark themes
}

/// Hook for automatic theme switching based on time
#[allow(non_snake_case)]
pub fn use_auto_theme_switching(enabled: bool, dark_start_hour: u8, light_start_hour: u8) {
    let theme_hook = use_theme();
    
    use_effect(move || {
        if !enabled {
            return move || {};
        }
        
        let theme_hook = theme_hook.clone();
        
        spawn(async move {
            loop {
                // Check every hour
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
                
                let now = chrono::Local::now();
                let current_hour = now.hour() as u8;
                
                let should_be_dark = if dark_start_hour < light_start_hour {
                    // Normal case: dark_start_hour=20, light_start_hour=6
                    current_hour >= dark_start_hour || current_hour < light_start_hour
                } else {
                    // Edge case across midnight: dark_start_hour=6, light_start_hour=20
                    current_hour >= dark_start_hour && current_hour < light_start_hour
                };
                
                let target_theme = if should_be_dark {
                    ThemeType::Dark
                } else {
                    ThemeType::Light
                };
                
                if theme_hook.current_theme() != target_theme {
                    theme_hook.set_theme(target_theme);
                }
            }
        });
        
        move || {
            // Cleanup if needed
        }
    });
}