//! Context providers for Dioxus app state
//!
//! Sets up Signal-based context providers for AppState, ThemeType, and IrcActions.

use crate::hooks::{use_theme::ThemeType, IrcActions};
use crate::state::AppState;
use dioxus::prelude::*;

/// Initialize all context providers. Call once from the root App component.
pub fn provide_app_context() -> (Signal<AppState>, Signal<ThemeType>) {
    let app_state = use_context_provider(|| Signal::new(AppState::new()));
    let theme = use_context_provider(|| Signal::new(ThemeType::Dark));

    // Provide IrcActions as context
    let actions = IrcActions::new(app_state);
    use_context_provider(|| actions);

    (app_state, theme)
}
