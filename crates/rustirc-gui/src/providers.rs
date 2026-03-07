//! Context providers for Dioxus app state
//!
//! Sets up Signal-based context providers for AppState, ThemeType, IrcActions,
//! and the real IrcClient event bridge.

use crate::bridge::ChannelEventHandler;
use crate::hooks::{use_irc_event_handler, use_theme::ThemeType, IrcActions};
use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_core::spawn_forever;

/// Initialize all context providers and start the IRC event bridge.
/// Call once from the root App component.
pub fn provide_app_context() -> (Signal<AppState>, Signal<ThemeType>) {
    let app_state = use_context_provider(|| Signal::new(AppState::new()));
    let theme = use_context_provider(|| Signal::new(ThemeType::Dark));

    // Create a channel for bridging EventBus events to Dioxus
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

    // Register the channel bridge handler with the core EventBus
    let client = crate::irc_client();
    let event_bus = client.event_bus();
    let handler = ChannelEventHandler::new(event_tx);
    spawn_forever(async move {
        event_bus.register(handler).await;
    });

    // Store the event receiver so the coroutine can take ownership
    let event_rx_signal = use_context_provider(|| Signal::new(Some(event_rx)));

    // Provide IrcActions (Copy type) as context
    let actions = IrcActions::new(app_state);
    use_context_provider(|| actions);

    // Start the event handler coroutine
    use_irc_event_handler(app_state, event_rx_signal);

    (app_state, theme)
}
