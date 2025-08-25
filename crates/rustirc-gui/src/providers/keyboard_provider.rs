//! Keyboard shortcuts and input handling provider

use crate::context::{DialogType, ThemeType, UiState};
use dioxus::prelude::*;

/// Keyboard provider for global shortcut handling
#[component]
pub fn KeyboardProvider(children: Element) -> Element {
    let ui_state = use_context::<UiState>();
    let theme_state = use_context::<crate::context::ThemeState>();
    
    // Set up global keyboard shortcuts for desktop
    use_effect(move || {
        setup_desktop_keyboard_shortcuts(ui_state, theme_state)
    });

    rsx! { {children} }
}

/// Set up desktop keyboard shortcuts using Dioxus event handling
fn setup_desktop_keyboard_shortcuts(
    ui_state: UiState, 
    theme_state: crate::context::ThemeState
) -> impl FnOnce() {
    // In desktop Dioxus apps, keyboard events are handled by component event handlers
    // This function sets up global key bindings and creates a cleanup function
    
    // Store global keyboard state for shortcuts
    let _keyboard_state = GlobalKeyboardState::new(ui_state, theme_state);
    
    // Return cleanup function (currently no-op for desktop)
    move || {
        // Desktop keyboard cleanup if needed
    }
}

/// Global keyboard state for managing shortcuts
struct GlobalKeyboardState {
    ui_state: UiState,
    theme_state: crate::context::ThemeState,
}

impl GlobalKeyboardState {
    fn new(ui_state: UiState, theme_state: crate::context::ThemeState) -> Self {
        Self { ui_state, theme_state }
    }
    
    /// Handle global keyboard shortcuts
    pub fn handle_keydown(&self, ctrl_or_cmd: bool, shift: bool, alt: bool, key: &str) -> bool {
        match (ctrl_or_cmd, shift, alt, key) {
            // Connection shortcuts
            (true, false, false, "k") => {
                self.ui_state.show_dialog(DialogType::Connect);
                true
            }
            
            // Settings shortcuts  
            (true, false, false, ",") => {
                self.ui_state.show_dialog(DialogType::Settings);
                true
            }
            
            // Tab management shortcuts
            (true, false, false, "t") => {
                // TODO: New tab functionality
                true
            }
            (true, false, false, "w") => {
                // TODO: Close current tab
                true
            }
            
            // Theme shortcuts
            (true, true, false, "t") => {
                cycle_theme(&self.theme_state);
                true
            }
            
            // View shortcuts
            (true, false, false, "1") => {
                self.ui_state.user_list_visible.set(!self.ui_state.user_list_visible());
                true
            }
            (true, false, false, "2") => {
                // TODO: Toggle sidebar
                true
            }
            
            // Search shortcuts
            (true, false, false, "f") => {
                // TODO: Search in current channel
                true
            }
            (true, true, false, "f") => {
                // TODO: Global search
                true
            }
            
            // Help shortcuts
            (false, false, false, "F1") => {
                self.ui_state.show_dialog(DialogType::About);
                true
            }
            
            // Developer shortcuts (for debug builds)
            #[cfg(debug_assertions)]
            (true, true, false, "d") => {
                toggle_developer_tools();
                true
            }
            
            // Escape key - close everything
            (false, false, false, "Escape") => {
                self.ui_state.active_dialogs.write().clear();
                self.ui_state.hide_context_menu();
                true
            }
            
            // Number shortcuts for tab switching (Alt+1-9)
            (false, false, true, n) if n.chars().next().map_or(false, |c| c.is_ascii_digit()) => {
                // TODO: Switch to tab N
                true
            }
            
            _ => false // Not handled
        }
    }
}

/// Cycle through available themes
fn cycle_theme(theme_state: &crate::context::ThemeState) {
    let current_theme = theme_state.current_theme.read();
    
    let next_theme = match *current_theme {
        ThemeType::Dark => ThemeType::Light,
        ThemeType::Light => ThemeType::Discord,
        ThemeType::Discord => ThemeType::Nord,
        ThemeType::Nord => ThemeType::MaterialDesign,
        ThemeType::MaterialDesign => ThemeType::Dracula,
        ThemeType::Dracula => ThemeType::Catppuccin,
        ThemeType::Catppuccin => ThemeType::Terminal,
        ThemeType::Terminal => ThemeType::Slack,
        ThemeType::Slack => ThemeType::Dark,
    };
    
    theme_state.set_theme(next_theme);
}

/// Toggle developer tools (debug builds only)
#[cfg(debug_assertions)]
fn toggle_developer_tools() {
    // For desktop Dioxus apps, we can log debug information
    println!("Developer tools toggle requested");
    
    // Could also inject debug information or toggle debug overlays
    // This is mainly for development convenience
}

/// Hook for component-specific keyboard shortcuts (desktop version)
#[allow(non_snake_case)]
pub fn use_keyboard_shortcuts() {
    // In desktop Dioxus apps, keyboard shortcuts are typically handled
    // by the component's onkeydown event handlers rather than global listeners
    use_effect(move || {
        // Desktop keyboard shortcut setup
        move || {
            // Desktop keyboard cleanup if needed
        }
    });
}

/// Hook for input field shortcuts (e.g., IRC formatting)
#[allow(non_snake_case)]
pub fn use_input_shortcuts(input_ref: Signal<String>) -> impl Fn(bool, bool, bool, &str) {
    move |ctrl_or_cmd: bool, _shift: bool, _alt: bool, key: &str| {
        if ctrl_or_cmd {
            match key {
                "b" => {
                    insert_irc_formatting(&input_ref, "\u{0002}", "\u{0002}"); // Bold
                }
                "i" => {
                    insert_irc_formatting(&input_ref, "\u{001d}", "\u{001d}"); // Italic
                }
                "u" => {
                    insert_irc_formatting(&input_ref, "\u{001f}", "\u{001f}"); // Underline
                }
                "k" => {
                    insert_irc_formatting(&input_ref, "\u{0003}", ""); // Color code
                }
                _ => {}
            }
        }
    }
}

/// Insert IRC formatting codes around selection or at cursor
fn insert_irc_formatting(input_ref: &Signal<String>, start_code: &str, end_code: &str) {
    let current_text = input_ref.read();
    
    // For now, just append the formatting codes
    // TODO: Handle text selection and cursor position properly
    let new_text = format!("{}{}{}", current_text, start_code, end_code);
    input_ref.set(new_text);
}