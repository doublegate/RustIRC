//! UI state management provider

use crate::context::UiState;
use dioxus::prelude::*;

/// UI provider for managing interface state and preferences
#[component]
pub fn UiProvider(children: Element) -> Element {
    use_context_provider(|| UiState::default());
    
    let ui_state = use_context::<UiState>();
    
    // Set up UI event handlers for desktop
    use_effect(move || {
        setup_desktop_ui_handlers(ui_state);
        move || {
            // Desktop cleanup if needed
        }
    });

    rsx! { {children} }
}

/// Set up UI event handlers for desktop window resize, etc.
fn setup_desktop_ui_handlers(ui_state: UiState) {
    // For desktop Dioxus apps, window resize and events are handled by the desktop runtime
    // This function sets up the initial UI state
    let _ui_state = ui_state.clone();
    
    // Desktop apps typically start with optimal layout settings
    // The window manager or Dioxus runtime handles resize events
}

/// Hook for responsive layout adjustments (desktop version)
#[allow(non_snake_case)]
pub fn use_responsive_layout() -> bool {
    let mut is_mobile = use_signal(|| false);
    
    use_effect(move || {
        // For desktop apps, we typically don't have mobile layouts
        // This could be based on window size if needed
        is_mobile.set(false);
        
        move || {
            // Desktop cleanup if needed
        }
    });
    
    is_mobile()
}

/// Hook for managing sidebar collapse state
#[allow(non_snake_case)]
pub fn use_sidebar_state() -> (bool, Signal<bool>) {
    let mut collapsed = use_signal(|| false);
    let ui_state = use_context::<UiState>();
    
    // Sync with responsive layout
    use_effect(move || {
        if use_responsive_layout() {
            collapsed.set(true);
        }
        move || {
            // Desktop cleanup if needed
        }
    });
    
    // Return the signal itself so callers can use .set() directly
    (collapsed(), collapsed)
}

/// Hook for managing theme persistence
#[allow(non_snake_case)]  
pub fn use_theme_persistence() {
    let theme_state = use_context::<crate::context::ThemeState>();
    
    // Load theme from localStorage on mount
    use_effect(move || {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten()) 
        {
            if let Ok(Some(theme_str)) = storage.get_item("rustirc-theme") {
                if let Ok(theme) = serde_json::from_str::<crate::context::ThemeType>(&theme_str) {
                    theme_state.set_theme(theme);
                }
            }
        }
    });
    
    // Save theme to localStorage when it changes
    use_effect(move || {
        let current_theme = theme_state.current_theme.read();
        
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
        {
            if let Ok(theme_json) = serde_json::to_string(&*current_theme) {
                let _ = storage.set_item("rustirc-theme", &theme_json);
            }
        }
    });
}