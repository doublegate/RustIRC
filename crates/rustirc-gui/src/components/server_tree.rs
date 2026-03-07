//! Server tree component showing connected servers and their channels

use crate::hooks::IrcActions;
use crate::state::AppState;
use dioxus::prelude::*;
use rustirc_core::connection::ConnectionState;

#[component]
pub fn ServerTree() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let actions = use_context::<IrcActions>();
    let state = app_state.read();

    rsx! {
        div {
            class: "flex flex-col p-2 text-sm text-[var(--text-color,#e0e0e0)]",

            h3 {
                class: "text-xs uppercase tracking-wider text-[var(--text-muted,#888)] mb-2 font-semibold",
                "Servers"
            }

            if state.servers.is_empty() {
                div {
                    class: "text-[var(--text-muted,#888)] italic text-xs px-2",
                    "No servers connected"
                }
            }

            for (server_id, server) in state.servers.iter() {
                {
                    let is_connected = server.connection_state == ConnectionState::Connected;
                    let status_color = if is_connected { "bg-green-500" } else { "bg-red-500" };
                    let server_tab_id = format!("server:{server_id}");
                    let is_active = state.current_tab_id.as_ref() == Some(&server_tab_id);
                    let server_class = if is_active {
                        "flex items-center gap-1.5 px-2 py-1 rounded cursor-pointer bg-[var(--selection-bg,#37373d)]"
                    } else {
                        "flex items-center gap-1.5 px-2 py-1 rounded cursor-pointer hover:bg-[var(--hover-bg,#2a2a2a)]"
                    };

                    rsx! {
                        div {
                            class: "mb-1",

                            div {
                                class: "{server_class}",
                                onclick: {
                                    let tab_id = server_tab_id.clone();
                                    move |_| actions.switch_tab(&tab_id)
                                },

                                span { class: "w-2 h-2 rounded-full {status_color}" }
                                span { class: "font-medium truncate", "{server.name}" }
                            }

                            for (channel_name, _channel_info) in server.channels.iter() {
                                {
                                    let tab_id = format!("{server_id}:{channel_name}");
                                    let is_channel_active = state.current_tab_id.as_ref() == Some(&tab_id);
                                    let tab = state.tabs.get(&tab_id);
                                    let has_activity = tab.map(|t| t.has_activity).unwrap_or(false);
                                    let ch_class = if is_channel_active {
                                        "flex items-center gap-1 pl-6 pr-2 py-0.5 rounded cursor-pointer text-xs bg-[var(--selection-bg,#37373d)] text-[var(--text-color,#e0e0e0)]"
                                    } else {
                                        "flex items-center gap-1 pl-6 pr-2 py-0.5 rounded cursor-pointer text-xs text-[var(--text-muted,#aaa)] hover:bg-[var(--hover-bg,#2a2a2a)]"
                                    };

                                    rsx! {
                                        div {
                                            class: "{ch_class}",
                                            onclick: {
                                                let tab_id = tab_id.clone();
                                                move |_| actions.switch_tab(&tab_id)
                                            },

                                            if has_activity {
                                                span { class: "w-1.5 h-1.5 rounded-full bg-[var(--activity-color,#4ecdc4)]" }
                                            }
                                            span { "# {channel_name}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
