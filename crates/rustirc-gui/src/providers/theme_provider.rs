//! Theme management and CSS injection provider

use crate::context::{ThemeState, ThemeType};
use dioxus::prelude::*;

/// Theme provider for managing application theming
#[component]
pub fn ThemeProvider(children: Element) -> Element {
    use_context_provider(|| ThemeState::default());

    let theme_state = use_context::<ThemeState>();
    let current_theme = theme_state.current_theme.read();
    let custom_css = theme_state.custom_css.read();

    // Update document theme attribute and CSS variables
    use_effect(move || {
        update_theme_styles(*current_theme);
    });

    rsx! {
        // Inject Material Design CSS and custom theme styles
        head {
            // Material Design CSS (if using CDN)
            link {
                rel: "stylesheet",
                href: "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap"
            }

            // Custom theme CSS
            style {
                dangerous_inner_html: "{get_theme_css(*current_theme)}{custom_css}"
            }
        }

        {children}
    }
}

/// Update theme styles in the DOM
fn update_theme_styles(theme: ThemeType) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(html_element) = document.document_element() {
                let theme_name = match theme {
                    ThemeType::Dark => "dark",
                    ThemeType::Light => "light",
                    ThemeType::Discord => "discord",
                    ThemeType::Slack => "slack",
                    ThemeType::Terminal => "terminal",
                    ThemeType::Nord => "nord",
                    ThemeType::Dracula => "dracula",
                    ThemeType::MaterialDesign => "material",
                    ThemeType::Catppuccin => "catppuccin",
                };

                let _ = html_element.set_attribute("data-theme", theme_name);
            }
        }
    }
}

/// Get CSS for the specified theme
fn get_theme_css(theme: ThemeType) -> String {
    match theme {
        ThemeType::Dark => get_dark_theme_css(),
        ThemeType::Light => get_light_theme_css(),
        ThemeType::Discord => get_discord_theme_css(),
        ThemeType::Slack => get_slack_theme_css(),
        ThemeType::Terminal => get_terminal_theme_css(),
        ThemeType::Nord => get_nord_theme_css(),
        ThemeType::Dracula => get_dracula_theme_css(),
        ThemeType::MaterialDesign => get_material_design_theme_css(),
        ThemeType::Catppuccin => get_catppuccin_theme_css(),
    }
}

/// Dark theme CSS with Material Design principles
fn get_dark_theme_css() -> String {
    r#"
    :root[data-theme="dark"] {
        /* Material Design Dark Theme Colors */
        --bg-primary: #121212;
        --bg-secondary: #1e1e1e;
        --bg-tertiary: #2d2d2d;
        --bg-quaternary: #3d3d3d;
        
        --text-primary: #ffffff;
        --text-secondary: #e0e0e0;
        --text-muted: #9e9e9e;
        --text-disabled: #616161;
        
        --accent-primary: #bb86fc;
        --accent-secondary: #03dac6;
        --accent-tertiary: #cf6679;
        
        --border-color: #3d3d3d;
        --border-hover: #5d5d5d;
        
        --success: #4caf50;
        --warning: #ff9800;
        --error: #f44336;
        --info: #2196f3;
        
        /* Component-specific colors */
        --sidebar-bg: var(--bg-secondary);
        --message-bg: var(--bg-primary);
        --input-bg: var(--bg-tertiary);
        --button-bg: var(--accent-primary);
        --button-hover: #9575cd;
        
        /* Shadows */
        --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
        --shadow-md: 0 4px 8px rgba(0, 0, 0, 0.4);
        --shadow-lg: 0 8px 16px rgba(0, 0, 0, 0.5);
    }
    "#
    .to_string()
}

/// Light theme CSS with Material Design principles
fn get_light_theme_css() -> String {
    r#"
    :root[data-theme="light"] {
        /* Material Design Light Theme Colors */
        --bg-primary: #ffffff;
        --bg-secondary: #f5f5f5;
        --bg-tertiary: #eeeeee;
        --bg-quaternary: #e0e0e0;
        
        --text-primary: #212121;
        --text-secondary: #424242;
        --text-muted: #757575;
        --text-disabled: #9e9e9e;
        
        --accent-primary: #6200ea;
        --accent-secondary: #00bcd4;
        --accent-tertiary: #e91e63;
        
        --border-color: #e0e0e0;
        --border-hover: #bdbdbd;
        
        --success: #4caf50;
        --warning: #ff9800;
        --error: #f44336;
        --info: #2196f3;
        
        /* Component-specific colors */
        --sidebar-bg: var(--bg-secondary);
        --message-bg: var(--bg-primary);
        --input-bg: var(--bg-tertiary);
        --button-bg: var(--accent-primary);
        --button-hover: #7c4dff;
        
        /* Shadows */
        --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.1);
        --shadow-md: 0 4px 8px rgba(0, 0, 0, 0.15);
        --shadow-lg: 0 8px 16px rgba(0, 0, 0, 0.2);
    }
    "#
    .to_string()
}

/// Discord theme CSS
fn get_discord_theme_css() -> String {
    r#"
    :root[data-theme="discord"] {
        --bg-primary: #36393f;
        --bg-secondary: #2f3136;
        --bg-tertiary: #40444b;
        --bg-quaternary: #4f545c;
        
        --text-primary: #dcddde;
        --text-secondary: #b9bbbe;
        --text-muted: #8e9297;
        --text-disabled: #72767d;
        
        --accent-primary: #5865f2;
        --accent-secondary: #57f287;
        --accent-tertiary: #ed4245;
        
        --border-color: #202225;
        --border-hover: #4f545c;
        
        --success: #57f287;
        --warning: #fee75c;
        --error: #ed4245;
        --info: #5865f2;
        
        --sidebar-bg: var(--bg-secondary);
        --message-bg: var(--bg-primary);
        --input-bg: var(--bg-tertiary);
        --button-bg: var(--accent-primary);
        --button-hover: #4752c4;
        
        --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
        --shadow-md: 0 4px 8px rgba(0, 0, 0, 0.4);
        --shadow-lg: 0 8px 16px rgba(0, 0, 0, 0.5);
    }
    "#
    .to_string()
}

/// Nord theme CSS  
fn get_nord_theme_css() -> String {
    r#"
    :root[data-theme="nord"] {
        --bg-primary: #2e3440;
        --bg-secondary: #3b4252;
        --bg-tertiary: #434c5e;
        --bg-quaternary: #4c566a;
        
        --text-primary: #eceff4;
        --text-secondary: #e5e9f0;
        --text-muted: #d8dee9;
        --text-disabled: #4c566a;
        
        --accent-primary: #88c0d0;
        --accent-secondary: #8fbcbb;
        --accent-tertiary: #81a1c1;
        
        --border-color: #3b4252;
        --border-hover: #434c5e;
        
        --success: #a3be8c;
        --warning: #ebcb8b;
        --error: #bf616a;
        --info: #5e81ac;
        
        --sidebar-bg: var(--bg-secondary);
        --message-bg: var(--bg-primary);
        --input-bg: var(--bg-tertiary);
        --button-bg: var(--accent-primary);
        --button-hover: #7ab8c7;
        
        --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.3);
        --shadow-md: 0 4px 8px rgba(0, 0, 0, 0.4);
        --shadow-lg: 0 8px 16px rgba(0, 0, 0, 0.5);
    }
    "#
    .to_string()
}

/// Material Design theme CSS (enhanced)
fn get_material_design_theme_css() -> String {
    r#"
    :root[data-theme="material"] {
        /* Material Design 3.0 Colors */
        --bg-primary: #1a1c1e;
        --bg-secondary: #21262a;
        --bg-tertiary: #2b3136;
        --bg-quaternary: #353c42;
        
        --text-primary: #e3e3e3;
        --text-secondary: #c7c7c7;
        --text-muted: #9ca1a7;
        --text-disabled: #6c757d;
        
        --accent-primary: #bb86fc;
        --accent-secondary: #03dac6;
        --accent-tertiary: #cf6679;
        
        --border-color: #2b3136;
        --border-hover: #353c42;
        
        --success: #00c853;
        --warning: #ffb300;
        --error: #d32f2f;
        --info: #1976d2;
        
        --sidebar-bg: var(--bg-secondary);
        --message-bg: var(--bg-primary);
        --input-bg: var(--bg-tertiary);
        --button-bg: var(--accent-primary);
        --button-hover: #9575cd;
        
        /* Material Design elevation shadows */
        --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.2), 0 1px 2px rgba(0, 0, 0, 0.12);
        --shadow-md: 0 3px 6px rgba(0, 0, 0, 0.15), 0 2px 4px rgba(0, 0, 0, 0.12);
        --shadow-lg: 0 10px 20px rgba(0, 0, 0, 0.15), 0 3px 6px rgba(0, 0, 0, 0.10);
        
        /* Material Design motion curves */
        --motion-standard: cubic-bezier(0.2, 0, 0, 1);
        --motion-emphasized: cubic-bezier(0.05, 0.7, 0.1, 1);
    }
    "#
    .to_string()
}

// Placeholder implementations for other themes
fn get_slack_theme_css() -> String {
    get_dark_theme_css()
}
fn get_terminal_theme_css() -> String {
    get_dark_theme_css()
}
fn get_dracula_theme_css() -> String {
    get_dark_theme_css()
}
fn get_catppuccin_theme_css() -> String {
    get_dark_theme_css()
}
