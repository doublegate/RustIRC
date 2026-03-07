//! User list component showing users in the current channel

use crate::state::{AppState, TabType};
use dioxus::prelude::*;

#[component]
pub fn UserList() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();

    let current_tab = state.current_tab();

    // Only show user list for channel tabs
    let is_channel = current_tab
        .map(|t| matches!(t.tab_type, TabType::Channel { .. }))
        .unwrap_or(false);

    if !is_channel {
        return rsx! {
            div {
                class: "p-2 text-xs text-[var(--text-muted,#888)]",
                "No channel selected"
            }
        };
    }

    let tab = current_tab.unwrap();
    let mut users: Vec<_> = tab.users.values().collect();
    users.sort_by(|a, b| {
        b.privilege_level()
            .cmp(&a.privilege_level())
            .then(a.nickname.cmp(&b.nickname))
    });

    let user_count = users.len();

    rsx! {
        div {
            class: "flex flex-col p-2 text-sm",

            h3 {
                class: "text-xs uppercase tracking-wider text-[var(--text-muted,#888)] mb-2 font-semibold",
                "Users ({user_count})"
            }

            for user in users.iter() {
                {
                    let prefix = if user.has_mode('o') {
                        "@"
                    } else if user.has_mode('h') {
                        "%"
                    } else if user.has_mode('v') {
                        "+"
                    } else {
                        ""
                    };

                    let mode_class = if user.has_mode('o') {
                        "text-[var(--op-color,#e06c75)]"
                    } else if user.has_mode('h') {
                        "text-[var(--halfop-color,#d19a66)]"
                    } else if user.has_mode('v') {
                        "text-[var(--voice-color,#98c379)]"
                    } else {
                        "text-[var(--text-color,#e0e0e0)]"
                    };

                    let away_class = if user.is_away { "opacity-50" } else { "" };

                    rsx! {
                        div {
                            class: "flex items-center px-1 py-0.5 rounded cursor-default hover:bg-[var(--hover-bg,#2a2a2a)] {away_class}",

                            span {
                                class: "text-xs {mode_class}",
                                "{prefix}{user.nickname}"
                            }
                        }
                    }
                }
            }
        }
    }
}
