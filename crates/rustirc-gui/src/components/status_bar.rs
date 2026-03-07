//! Status bar component showing connection info

use crate::state::AppState;
use dioxus::prelude::*;
use rustirc_core::connection::ConnectionState;

#[component]
pub fn StatusBar() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();

    let server_count = state.servers.len();
    let connected_count = state
        .servers
        .values()
        .filter(|s| s.connection_state == ConnectionState::Connected)
        .count();

    let current_server = state
        .current_tab_id
        .as_ref()
        .and_then(|tab_id| state.tabs.get(tab_id).and_then(|tab| tab.server_id.clone()))
        .unwrap_or_else(|| "No server".to_string());

    let current_nick = state
        .current_tab_id
        .as_ref()
        .and_then(|tab_id| {
            state
                .tabs
                .get(tab_id)
                .and_then(|tab| tab.server_id.as_ref())
                .and_then(|sid| state.servers.get(sid))
                .map(|s| s.nickname.clone())
        })
        .unwrap_or_default();

    rsx! {
        div {
            class: "flex items-center justify-between px-3 py-1 bg-[var(--statusbar-bg,#1a1a1a)] text-[var(--text-muted,#888)] text-xs border-t border-[var(--border-color,#333)]",

            // Left: connection info
            div {
                class: "flex items-center gap-3",
                span {
                    class: "flex items-center gap-1",
                    span {
                        class: if connected_count > 0 { "w-2 h-2 rounded-full bg-green-500" } else { "w-2 h-2 rounded-full bg-red-500" },
                    }
                    "{connected_count}/{server_count} servers"
                }
                if !current_nick.is_empty() {
                    span { "Nick: {current_nick}" }
                }
            }

            // Center: current server
            div { "{current_server}" }

            // Right: version
            div { {"RustIRC v".to_string() + env!("CARGO_PKG_VERSION")} }
        }
    }
}
