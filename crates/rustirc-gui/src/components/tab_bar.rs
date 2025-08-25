//! Tab bar component for managing multiple channels/conversations

use crate::context::{DialogType, IrcState, UiState};
use dioxus::prelude::*;

/// Tab bar component for switching between channels
#[component]
pub fn TabBar() -> Element {
    let irc_state = use_context::<IrcState>();
    let ui_state = use_context::<UiState>();
    let connections = irc_state.connections.read();
    let active_tab = irc_state.active_tab.read();

    // Build list of all open tabs
    let mut tabs = Vec::new();

    // Always have a welcome tab
    tabs.push(("welcome".to_string(), "Welcome".to_string(), 0));

    // Add server and channel tabs
    for (server_id, connection) in connections.iter() {
        // Add server tab
        tabs.push((server_id.clone(), connection.server.clone(), 0));

        // Add channel tabs
        for (channel_name, channel_info) in connection.channels.iter() {
            let tab_id = format!("{}:{}", server_id, channel_name);
            let display_name = if channel_name.starts_with('#') || channel_name.starts_with('&') {
                channel_name.clone()
            } else {
                channel_name.clone()
            };
            tabs.push((tab_id, display_name, channel_info.unread_count));
        }
    }

    rsx! {
        div {
            class: "flex items-center h-10 px-2 space-x-1 overflow-x-auto custom-scrollbar",

            // Tab items
            for (tab_id, display_name, unread_count) in tabs.iter() {
                TabItem {
                    key: "{tab_id}",
                    tab_id: tab_id.clone(),
                    display_name: display_name.clone(),
                    unread_count: *unread_count,
                    is_active: *active_tab == *tab_id,
                }
            }

            // Add tab button
            button {
                class: "ml-2 px-3 py-1 text-xs rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-muted)] transition-colors",
                onclick: move |_| {
                    ui_state.show_dialog(DialogType::Connect);
                },
                title: "Connect to new server",
                "+"
            }

            // Menu buttons on the right
            div {
                class: "ml-auto flex items-center space-x-2",

                // Settings button
                button {
                    class: "px-2 py-1 text-xs rounded hover:bg-[var(--bg-tertiary)] transition-colors",
                    onclick: move |_| {
                        ui_state.show_dialog(DialogType::Settings);
                    },
                    title: "Settings",
                    "⚙"
                }

                // About button
                button {
                    class: "px-2 py-1 text-xs rounded hover:bg-[var(--bg-tertiary)] transition-colors",
                    onclick: move |_| {
                        ui_state.show_dialog(DialogType::About);
                    },
                    title: "About",
                    "?"
                }
            }
        }
    }
}

/// Individual tab item
#[component]
fn TabItem(tab_id: String, display_name: String, unread_count: usize, is_active: bool) -> Element {
    let irc_state = use_context::<IrcState>();

    let tab_class = if is_active {
        "flex items-center px-3 py-1 text-sm rounded bg-[var(--accent-primary)] text-white cursor-pointer"
    } else if unread_count > 0 {
        "flex items-center px-3 py-1 text-sm rounded hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors irc-channel-unread"
    } else {
        "flex items-center px-3 py-1 text-sm rounded hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors"
    };

    rsx! {
        div {
            class: "relative group",

            div {
                class: "{tab_class}",
                onclick: move |_| {
                    irc_state.active_tab.set(tab_id.clone());

                    // Update current server and channel based on tab
                    if tab_id == "welcome" {
                        irc_state.current_server.set(None);
                        irc_state.current_channel.set(None);
                    } else if tab_id.contains(':') {
                        // Channel tab (server:channel)
                        let parts: Vec<&str> = tab_id.split(':').collect();
                        if parts.len() == 2 {
                            irc_state.current_server.set(Some(parts[0].to_string()));
                            irc_state.current_channel.set(Some(parts[1].to_string()));
                        }
                    } else {
                        // Server tab
                        irc_state.current_server.set(Some(tab_id.clone()));
                        irc_state.current_channel.set(None);
                    }
                },

                // Tab name
                span {
                    class: "truncate max-w-32",
                    title: "{display_name}",
                    "{display_name}"
                }

                // Unread indicator
                if unread_count > 0 {
                    span {
                        class: if is_active {
                            "ml-2 bg-white bg-opacity-20 text-xs rounded-full px-1.5 py-0.5 min-w-[18px] text-center"
                        } else {
                            "ml-2 bg-[var(--accent-primary)] text-white text-xs rounded-full px-1.5 py-0.5 min-w-[18px] text-center"
                        },
                        style: "font-size: 10px;",
                        "{unread_count}"
                    }
                }
            }

            // Close button (show on hover, except for welcome tab)
            if tab_id != "welcome" {
                button {
                    class: "absolute -top-1 -right-1 w-4 h-4 bg-[var(--error)] text-white text-xs rounded-full opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center",
                    onclick: move |e| {
                        e.stop_propagation();
                        close_tab(tab_id.clone());
                    },
                    title: "Close tab",
                    "×"
                }
            }
        }
    }
}

/// Close a tab (leave channel or disconnect server)
fn close_tab(tab_id: String) {
    let irc_state = use_context::<IrcState>();

    if tab_id.contains(':') {
        // Channel tab - leave the channel
        let parts: Vec<&str> = tab_id.split(':').collect();
        if parts.len() == 2 {
            let server_id = parts[0];
            let channel_name = parts[1];

            // Remove channel from state
            if let Some(connection) = irc_state.connections.write().get_mut(server_id) {
                connection.channels.remove(channel_name);

                // TODO: Send PART command to IRC server
                // if let Some(client) = &connection.client {
                //     client.part_channel(channel_name).await;
                // }
            }

            // Switch to server tab or welcome if this was the active tab
            if irc_state.active_tab.read().as_str() == tab_id {
                irc_state.active_tab.set(server_id.to_string());
                irc_state.current_channel.set(None);
            }
        }
    } else {
        // Server tab - disconnect from server
        irc_state.connections.write().remove(&tab_id);

        // Switch to welcome tab if this was the active tab
        if irc_state.active_tab.read().as_str() == tab_id {
            irc_state.active_tab.set("welcome".to_string());
            irc_state.current_server.set(None);
            irc_state.current_channel.set(None);
        }

        // TODO: Disconnect from IRC server
        // if let Some(client) = connection.client {
        //     client.disconnect().await;
        // }
    }
}
