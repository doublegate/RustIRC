//! User list component showing channel members

use crate::context::{IrcState, UserInfo};
use dioxus::prelude::*;
use std::collections::HashMap;

/// User list component for showing channel members
#[component]
pub fn UserList() -> Element {
    let irc_state = use_context::<IrcState>();
    let current_server = irc_state.current_server.read();
    let current_channel = irc_state.current_channel.read();
    let connections = irc_state.connections.read();

    // Get users for current channel
    let users = get_channel_users(&*current_server, &*current_channel, &*connections);

    rsx! {
        div {
            class: "h-full flex flex-col",

            // Header
            div {
                class: "p-3 border-b border-[var(--border-color)] flex items-center justify-between",
                h3 {
                    class: "text-sm font-medium text-[var(--text-secondary)] uppercase tracking-wide",
                    "Users ({users.len()})"
                }

                // User list options
                div {
                    class: "flex items-center space-x-1",
                    button {
                        class: "text-xs px-2 py-1 rounded hover:bg-[var(--bg-tertiary)] transition-colors",
                        title: "Sort users",
                        "↕"
                    }
                }
            }

            // User list
            div {
                class: "flex-1 overflow-y-auto custom-scrollbar",

                if users.is_empty() {
                    div {
                        class: "p-4 text-center text-[var(--text-muted)] text-sm",
                        if current_channel.is_some() {
                            p { "No users in this channel" }
                        } else {
                            p { "Join a channel to see users" }
                        }
                    }
                } else {
                    div {
                        class: "py-2",
                        for user in users.iter() {
                            UserItem {
                                key: "{user.nickname}",
                                user: user.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Individual user item in the list
#[component]
fn UserItem(user: UserInfo) -> Element {
    let ui_state = use_context::<crate::context::UiState>();

    // Determine user styling based on modes and status
    let user_class = if user.modes.contains(&'o') {
        "flex items-center px-3 py-1 hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors irc-user-op"
    } else if user.modes.contains(&'v') {
        "flex items-center px-3 py-1 hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors irc-user-voice"
    } else if user.away {
        "flex items-center px-3 py-1 hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors irc-user-away"
    } else {
        "flex items-center px-3 py-1 hover:bg-[var(--bg-tertiary)] cursor-pointer transition-colors"
    };

    let mode_prefix = get_mode_prefix(&user.modes);

    rsx! {
        div {
            class: "{user_class}",
            onclick: move |_| {
                ui_state.show_dialog(crate::context::DialogType::UserInfo(user.nickname.clone()));
            },
            oncontextmenu: move |evt| {
                evt.prevent_default();
                ui_state.show_context_menu(evt.data.client_coordinates().x as f32, evt.data.client_coordinates().y as f32);
            },

            // Mode indicator
            span {
                class: "w-4 text-center text-xs font-bold flex-shrink-0",
                "{mode_prefix}"
            }

            // Nickname
            span {
                class: "flex-1 text-sm truncate",
                title: if let Some(realname) = &user.realname {
                    format!("{} ({})", user.nickname, realname)
                } else {
                    user.nickname.clone()
                },
                "{user.nickname}"
            }

            // Away indicator
            if user.away {
                span {
                    class: "text-xs text-[var(--text-muted)] ml-1",
                    title: "Away",
                    "💤"
                }
            }
        }
    }
}

/// Get users for the specified channel
fn get_channel_users(
    server_id: &Option<String>,
    channel_name: &Option<String>,
    connections: &HashMap<String, crate::context::ConnectionInfo>,
) -> Vec<UserInfo> {
    if let (Some(server_id), Some(channel_name)) = (server_id, channel_name) {
        if let Some(connection) = connections.get(server_id) {
            if let Some(channel) = connection.channels.get(channel_name) {
                let mut users: Vec<UserInfo> = channel.users.values().cloned().collect();

                // Sort users by mode (ops first, then voiced, then regular)
                users.sort_by(|a, b| {
                    let a_priority = get_user_priority(a);
                    let b_priority = get_user_priority(b);

                    if a_priority != b_priority {
                        a_priority.cmp(&b_priority)
                    } else {
                        a.nickname.to_lowercase().cmp(&b.nickname.to_lowercase())
                    }
                });

                return users;
            }
        }
    }

    Vec::new()
}

/// Get priority for sorting users (lower number = higher priority)
fn get_user_priority(user: &UserInfo) -> u8 {
    if user.modes.contains(&'o') {
        0 // Operators first
    } else if user.modes.contains(&'v') {
        1 // Voiced users second
    } else {
        2 // Regular users last
    }
}

/// Get the mode prefix symbol for display
fn get_mode_prefix(modes: &std::collections::HashSet<char>) -> &'static str {
    if modes.contains(&'o') {
        "@" // Operator
    } else if modes.contains(&'v') {
        "+" // Voice
    } else {
        "" // Regular user
    }
}

/// User context menu component (could be enhanced)
#[component]
fn UserContextMenu(user: UserInfo, x: f32, y: f32) -> Element {
    let ui_state = use_context::<crate::context::UiState>();

    rsx! {
        div {
            class: "fixed context-menu z-50",
            style: "left: {x}px; top: {y}px;",

            div {
                class: "context-menu-item",
                onclick: move |_| {
                    // TODO: Open private message with user
                    ui_state.hide_context_menu();
                },
                "Send Message"
            }
            div {
                class: "context-menu-item",
                onclick: move |_| {
                    ui_state.show_dialog(crate::context::DialogType::UserInfo(user.nickname.clone()));
                    ui_state.hide_context_menu();
                },
                "User Info"
            }
            div {
                class: "context-menu-item",
                onclick: move |_| {
                    // TODO: Add to friends list
                    ui_state.hide_context_menu();
                },
                "Add Friend"
            }

            hr { class: "border-[var(--border-color)] my-1" }

            // Moderator actions (if user has permissions)
            div {
                class: "context-menu-item text-[var(--warning)]",
                onclick: move |_| {
                    // TODO: Kick user
                    ui_state.hide_context_menu();
                },
                "Kick"
            }
            div {
                class: "context-menu-item text-[var(--error)]",
                onclick: move |_| {
                    // TODO: Ban user
                    ui_state.hide_context_menu();
                },
                "Ban"
            }
        }
    }
}
