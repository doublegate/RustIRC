//! Sidebar component for server and channel navigation

use crate::context::{ChannelInfo, ConnectionInfo, IrcState};
use dioxus::prelude::*;
use std::collections::HashMap;

/// Sidebar component showing servers and channels
#[component]
pub fn Sidebar() -> Element {
    let irc_state = use_context::<IrcState>();
    let connections = irc_state.connections.read();
    let current_server = irc_state.current_server.read();
    let current_channel = irc_state.current_channel.read();

    rsx! {
        div {
            class: "h-full flex flex-col",

            // Sidebar header
            div {
                class: "p-3 border-b border-[var(--border-color)] flex items-center justify-between",
                h2 {
                    class: "text-sm font-medium text-[var(--text-secondary)] uppercase tracking-wide",
                    "Servers"
                }
                button {
                    class: "irc-button text-xs px-2 py-1",
                    onclick: move |_| {
                        let ui_state = use_context::<crate::context::UiState>();
                        ui_state.show_dialog(crate::context::DialogType::Connect);
                    },
                    "+"
                }
            }

            // Server list
            div {
                class: "flex-1 overflow-auto custom-scrollbar",
                if connections.is_empty() {
                    div {
                        class: "p-4 text-center text-[var(--text-muted)]",
                        p { "No servers connected" }
                        p { class: "text-xs mt-2", "Click + to connect to a server" }
                    }
                } else {
                    for (server_id, connection) in connections.iter() {
                        ServerItem {
                            key: "{server_id}",
                            server_id: server_id.clone(),
                            connection: connection.clone(),
                            is_current: current_server.as_ref() == Some(server_id),
                            current_channel: current_channel.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Individual server item with channels
#[component]
fn ServerItem(
    server_id: String,
    connection: ConnectionInfo,
    is_current: bool,
    current_channel: Option<String>,
) -> Element {
    let mut expanded = use_signal(|| true);
    let irc_state = use_context::<IrcState>();

    let status_color = match connection.state {
        rustirc_core::ConnectionState::Connected => "text-[var(--success)]",
        rustirc_core::ConnectionState::Connecting => "text-[var(--warning)]",
        rustirc_core::ConnectionState::Disconnected => "text-[var(--error)]",
        rustirc_core::ConnectionState::Reconnecting => "text-[var(--info)]",
    };

    rsx! {
        div {
            class: "mb-1",

            // Server header
            div {
                class: if is_current {
                    "flex items-center p-2 mx-2 rounded irc-channel-active cursor-pointer"
                } else {
                    "flex items-center p-2 mx-2 rounded hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors"
                },
                onclick: move |_| {
                    irc_state.current_server.set(Some(server_id.clone()));
                    irc_state.current_channel.set(None);
                    irc_state.active_tab.set(server_id.clone());
                },
                oncontextmenu: move |evt| {
                    evt.prevent_default();
                    let ui_state = use_context::<crate::context::UiState>();
                    ui_state.show_context_menu(evt.data.client_coordinates().x as f32, evt.data.client_coordinates().y as f32);
                },

                // Expand/collapse button
                button {
                    class: "mr-2 text-xs w-4 h-4 flex items-center justify-center",
                    onclick: move |e| {
                        e.stop_propagation();
                        expanded.set(!expanded());
                    },
                    if expanded() { "▼" } else { "▶" }
                }

                // Connection status indicator
                div {
                    class: "w-2 h-2 rounded-full mr-2 {status_color}",
                    style: "background-color: currentColor;"
                }

                // Server name
                span {
                    class: "flex-1 text-sm truncate",
                    title: "{connection.server}:{connection.port}",
                    "{connection.server}"
                }

                // Channel count
                if !connection.channels.is_empty() {
                    span {
                        class: "text-xs text-[var(--text-muted)] ml-2",
                        "{connection.channels.len()}"
                    }
                }
            }

            // Channel list (collapsible)
            if expanded() {
                div {
                    class: "ml-8 space-y-1",
                    for (channel_name, channel_info) in connection.channels.iter() {
                        ChannelItem {
                            key: "{server_id}-{channel_name}",
                            server_id: server_id.clone(),
                            channel_name: channel_name.clone(),
                            channel_info: channel_info.clone(),
                            is_current: current_channel.as_ref() == Some(channel_name),
                        }
                    }
                }
            }
        }
    }
}

/// Individual channel item
#[component]
fn ChannelItem(
    server_id: String,
    channel_name: String,
    channel_info: ChannelInfo,
    is_current: bool,
) -> Element {
    let irc_state = use_context::<IrcState>();

    let channel_class = if is_current {
        "flex items-center p-1 px-3 mr-2 rounded irc-channel-active cursor-pointer"
    } else if channel_info.unread_count > 0 {
        "flex items-center p-1 px-3 mr-2 rounded hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors irc-channel-unread"
    } else {
        "flex items-center p-1 px-3 mr-2 rounded hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors"
    };

    rsx! {
        div {
            class: "{channel_class}",
            onclick: move |_| {
                irc_state.current_server.set(Some(server_id.clone()));
                irc_state.current_channel.set(Some(channel_name.clone()));
                irc_state.active_tab.set(format!("{}:{}", server_id, channel_name));

                // Clear unread count
                if let Some(connection) = irc_state.connections.write().get_mut(&server_id) {
                    if let Some(channel) = connection.channels.get_mut(&channel_name) {
                        channel.unread_count = 0;
                    }
                }
            },
            oncontextmenu: move |evt| {
                evt.prevent_default();
                let ui_state = use_context::<crate::context::UiState>();
                ui_state.show_context_menu(evt.data.client_coordinates().x as f32, evt.data.client_coordinates().y as f32);
            },

            // Channel prefix
            span {
                class: "text-[var(--text-muted)] mr-1",
                if channel_name.starts_with('#') { "#" }
                else if channel_name.starts_with('&') { "&" }
                else { "" }
            }

            // Channel name
            span {
                class: "flex-1 text-sm truncate",
                title: "{channel_name}",
                if channel_name.starts_with('#') || channel_name.starts_with('&') {
                    "{channel_name[1..]}"
                } else {
                    "{channel_name}"
                }
            }

            // Unread count
            if channel_info.unread_count > 0 {
                span {
                    class: "bg-[var(--accent-primary)] text-white text-xs rounded-full px-1.5 py-0.5 min-w-[18px] text-center",
                    style: "font-size: 10px;",
                    "{channel_info.unread_count}"
                }
            }

            // User count
            if !channel_info.users.is_empty() {
                span {
                    class: "text-xs text-[var(--text-muted)] ml-2",
                    "{channel_info.users.len()}"
                }
            }
        }
    }
}
