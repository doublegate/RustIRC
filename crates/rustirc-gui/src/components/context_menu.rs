//! Context menu component for right-click actions

use crate::context::{IrcState, UiState};
use dioxus::prelude::*;

/// Context menu provider component
#[component]
pub fn ContextMenuProvider() -> Element {
    let ui_state = use_context::<UiState>();
    let context_menu = ui_state.context_menu.read();

    if let Some((x, y)) = context_menu.as_ref() {
        rsx! {
            div {
                class: "fixed inset-0 z-40",
                onclick: move |_| {
                    ui_state.hide_context_menu();
                },

                div {
                    class: "context-menu",
                    style: "left: {x}px; top: {y}px;",
                    onclick: move |e| {
                        e.stop_propagation();
                    },

                    ContextMenuContent {}
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}

/// Context menu content based on current context
#[component]
fn ContextMenuContent() -> Element {
    let ui_state = use_context::<UiState>();
    let irc_state = use_context::<IrcState>();
    let current_server = irc_state.current_server.read();
    let current_channel = irc_state.current_channel.read();

    rsx! {
        div {
            class: "context-menu-content",

            // Server actions (when in server context)
            if current_server.is_some() && current_channel.is_none() {
                ServerContextMenu {}
            }

            // Channel actions (when in channel context)
            if current_server.is_some() && current_channel.is_some() {
                ChannelContextMenu {}
            }

            // General actions (always available)
            GeneralContextMenu {}
        }
    }
}

/// Server-specific context menu items
#[component]
fn ServerContextMenu() -> Element {
    let ui_state = use_context::<UiState>();
    let irc_state = use_context::<IrcState>();
    let current_server = irc_state.current_server.read();

    rsx! {
        div {
            class: "context-menu-item",
            onclick: move |_| {
                ui_state.show_dialog(crate::context::DialogType::JoinChannel);
                ui_state.hide_context_menu();
            },
            "Join Channel..."
        }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                ui_state.show_dialog(crate::context::DialogType::ChannelList);
                ui_state.hide_context_menu();
            },
            "Channel List..."
        }

        hr { class: "border-[var(--border-color)] my-1" }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                // TODO: Implement server info
                ui_state.hide_context_menu();
            },
            "Server Info"
        }

        div {
            class: "context-menu-item text-[var(--warning)]",
            onclick: move |_| {
                if let Some(server_id) = current_server.as_ref() {
                    irc_state.disconnect_server(server_id.clone());
                }
                ui_state.hide_context_menu();
            },
            "Disconnect"
        }
    }
}

/// Channel-specific context menu items
#[component]
fn ChannelContextMenu() -> Element {
    let ui_state = use_context::<UiState>();
    let irc_state = use_context::<IrcState>();
    let current_channel = irc_state.current_channel.read();

    rsx! {
        div {
            class: "context-menu-item",
            onclick: move |_| {
                // TODO: Implement channel topic edit
                ui_state.hide_context_menu();
            },
            "Edit Topic"
        }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                // TODO: Implement channel modes
                ui_state.hide_context_menu();
            },
            "Channel Modes"
        }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                // TODO: Implement ban list
                ui_state.hide_context_menu();
            },
            "Ban List"
        }

        hr { class: "border-[var(--border-color)] my-1" }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                // TODO: Implement invite user
                ui_state.hide_context_menu();
            },
            "Invite User..."
        }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                // TODO: Implement clear messages
                ui_state.hide_context_menu();
            },
            "Clear Messages"
        }

        hr { class: "border-[var(--border-color)] my-1" }

        div {
            class: "context-menu-item text-[var(--warning)]",
            onclick: move |_| {
                if let Some(channel) = current_channel.as_ref() {
                    irc_state.part_channel(channel.clone());
                }
                ui_state.hide_context_menu();
            },
            "Leave Channel"
        }
    }
}

/// General context menu items (always available)
#[component]
fn GeneralContextMenu() -> Element {
    let ui_state = use_context::<UiState>();

    rsx! {
        div {
            class: "context-menu-item",
            onclick: move |_| {
                ui_state.show_dialog(crate::context::DialogType::Connect);
                ui_state.hide_context_menu();
            },
            "New Connection..."
        }

        hr { class: "border-[var(--border-color)] my-1" }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                ui_state.show_dialog(crate::context::DialogType::Settings);
                ui_state.hide_context_menu();
            },
            "Settings"
        }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                ui_state.show_dialog(crate::context::DialogType::Preferences);
                ui_state.hide_context_menu();
            },
            "Preferences"
        }

        hr { class: "border-[var(--border-color)] my-1" }

        div {
            class: "context-menu-item",
            onclick: move |_| {
                ui_state.show_dialog(crate::context::DialogType::About);
                ui_state.hide_context_menu();
            },
            "About RustIRC"
        }
    }
}

/// User-specific context menu for user list items
#[component]
pub fn UserContextMenu(username: String, x: f32, y: f32) -> Element {
    let ui_state = use_context::<UiState>();
    let irc_state = use_context::<IrcState>();

    rsx! {
        div {
            class: "fixed inset-0 z-40",
            onclick: move |_| {
                ui_state.hide_context_menu();
            },

            div {
                class: "context-menu",
                style: "left: {x}px; top: {y}px;",
                onclick: move |e| {
                    e.stop_propagation();
                },

                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        // TODO: Open private message with user
                        ui_state.hide_context_menu();
                    },
                    "Send Private Message"
                }

                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        ui_state.show_dialog(crate::context::DialogType::UserInfo(username.clone()));
                        ui_state.hide_context_menu();
                    },
                    "User Info"
                }

                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        // TODO: Implement WHOIS query
                        ui_state.hide_context_menu();
                    },
                    "WHOIS"
                }

                hr { class: "border-[var(--border-color)] my-1" }

                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        // TODO: Add to friends/ignore list
                        ui_state.hide_context_menu();
                    },
                    "Add to Friends"
                }

                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        // TODO: Ignore user
                        ui_state.hide_context_menu();
                    },
                    "Ignore User"
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
}

/// Message context menu for right-clicking on messages
#[component]
pub fn MessageContextMenu(message_id: String, sender: Option<String>, x: f32, y: f32) -> Element {
    let ui_state = use_context::<UiState>();

    rsx! {
        div {
            class: "fixed inset-0 z-40",
            onclick: move |_| {
                ui_state.hide_context_menu();
            },

            div {
                class: "context-menu",
                style: "left: {x}px; top: {y}px;",
                onclick: move |e| {
                    e.stop_propagation();
                },

                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        // TODO: Copy message text
                        ui_state.hide_context_menu();
                    },
                    "Copy Message"
                }

                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        // TODO: Reply to message
                        ui_state.hide_context_menu();
                    },
                    "Reply"
                }

                if let Some(username) = &sender {
                    hr { class: "border-[var(--border-color)] my-1" }

                    div {
                        class: "context-menu-item",
                        onclick: move |_| {
                            // TODO: Open private message with sender
                            ui_state.hide_context_menu();
                        },
                        "Message {username}"
                    }

                    div {
                        class: "context-menu-item",
                        onclick: move |_| {
                            ui_state.show_dialog(crate::context::DialogType::UserInfo(username.clone()));
                            ui_state.hide_context_menu();
                        },
                        "User Info"
                    }
                }

                hr { class: "border-[var(--border-color)] my-1" }

                div {
                    class: "context-menu-item text-[var(--error)]",
                    onclick: move |_| {
                        // TODO: Report message
                        ui_state.hide_context_menu();
                    },
                    "Report Message"
                }
            }
        }
    }
}
