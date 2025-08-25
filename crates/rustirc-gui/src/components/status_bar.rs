//! Status bar component showing connection and app status

use crate::context::{IrcState, ThemeState, ThemeType};
use dioxus::prelude::*;

/// Status bar component at the bottom of the application
#[component]
pub fn StatusBar() -> Element {
    let irc_state = use_context::<IrcState>();
    let theme_state = use_context::<ThemeState>();
    let current_server = irc_state.current_server.read();
    let current_channel = irc_state.current_channel.read();
    let connections = irc_state.connections.read();
    let current_theme = theme_state.current_theme.read();

    // Get connection status for current server
    let connection_status = if let Some(server_id) = current_server.as_ref() {
        if let Some(connection) = connections.get(server_id) {
            Some((
                connection.server.clone(),
                connection.state.clone(),
                connection.nickname.clone(),
            ))
        } else {
            None
        }
    } else {
        None
    };

    rsx! {
        div {
            class: "h-6 px-3 flex items-center justify-between text-xs bg-[var(--bg-secondary)] border-t border-[var(--border-color)]",

            // Left side - Connection status
            div {
                class: "flex items-center space-x-4",

                // Connection indicator
                match connection_status {
                    Some((server, state, nickname)) => rsx! {
                        div {
                            class: "flex items-center space-x-2",

                            // Status dot
                            div {
                                class: match state {
                                    rustirc_core::ConnectionState::Connected => "w-2 h-2 rounded-full bg-[var(--success)]",
                                    rustirc_core::ConnectionState::Connecting => "w-2 h-2 rounded-full bg-[var(--warning)]",
                                    rustirc_core::ConnectionState::Reconnecting => "w-2 h-2 rounded-full bg-[var(--info)] animate-pulse",
                                    rustirc_core::ConnectionState::Disconnected => "w-2 h-2 rounded-full bg-[var(--error)]",
                                }
                            }

                            // Connection info
                            span {
                                class: "text-[var(--text-secondary)]",
                                "{nickname} @ {server} ({state:?})"
                            }
                        }
                    },
                    None => rsx! {
                        div {
                            class: "flex items-center space-x-2 text-[var(--text-muted)]",
                            div { class: "w-2 h-2 rounded-full bg-[var(--text-muted)]" }
                            span { "Not connected" }
                        }
                    }
                }

                // Current channel indicator
                {
                    match current_channel.as_ref() {
                        Some(channel) => rsx! {
                            div {
                                class: "text-[var(--text-secondary)]",
                                "#{channel}"
                            }
                        },
                        None => rsx! { span {} }
                    }
                }
            }

            // Right side - App status and controls
            div {
                class: "flex items-center space-x-4",

                // Connection count
                div {
                    class: "text-[var(--text-muted)]",
                    "{connections.len()} connection{if connections.len() == 1 { \"\" } else { \"s\" }}"
                }

                // Theme switcher
                ThemeSelector { current_theme: *current_theme }

                // Version info
                div {
                    class: "text-[var(--text-muted)]",
                    "RustIRC v0.3.7"
                }
            }
        }
    }
}

/// Theme selector dropdown component
#[component]
fn ThemeSelector(current_theme: ThemeType) -> Element {
    let mut show_dropdown = use_signal(|| false);
    let theme_state = use_context::<ThemeState>();

    let themes = vec![
        (ThemeType::Dark, "Dark"),
        (ThemeType::Light, "Light"),
        (ThemeType::Discord, "Discord"),
        (ThemeType::Nord, "Nord"),
        (ThemeType::Dracula, "Dracula"),
        (ThemeType::MaterialDesign, "Material"),
        (ThemeType::Catppuccin, "Catppuccin"),
    ];

    let current_theme_name = themes
        .iter()
        .find(|(theme, _)| *theme == current_theme)
        .map(|(_, name)| *name)
        .unwrap_or("Unknown");

    rsx! {
        div {
            class: "relative",

            // Theme selector button
            button {
                class: "px-2 py-1 rounded hover:bg-[var(--bg-tertiary)] transition-colors text-[var(--text-secondary)] flex items-center space-x-1",
                onclick: move |_| {
                    show_dropdown.set(!show_dropdown());
                },

                span { "🎨" }
                span { "{current_theme_name}" }
                span {
                    class: "text-xs",
                    if show_dropdown() { "▲" } else { "▼" }
                }
            }

            // Dropdown menu
            if show_dropdown() {
                div {
                    class: "absolute bottom-full right-0 mb-1 context-menu",

                    for (theme, name) in themes.iter() {
                        button {
                            class: if *theme == current_theme {
                                "context-menu-item bg-[var(--accent-primary)] text-white w-full text-left"
                            } else {
                                "context-menu-item w-full text-left"
                            },
                            onclick: move |_| {
                                theme_state.set_theme(*theme);
                                show_dropdown.set(false);
                            },
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}

/// Connection status indicator component  
#[component]
pub fn ConnectionStatus() -> Element {
    let irc_state = use_context::<IrcState>();
    let connections = irc_state.connections.read();

    let (connected_count, total_count) = count_connections(&*connections);

    rsx! {
        div {
            class: "flex items-center space-x-2 text-xs",

            // Connection indicator
            div {
                class: if connected_count == total_count && total_count > 0 {
                    "w-2 h-2 rounded-full bg-[var(--success)]"
                } else if connected_count > 0 {
                    "w-2 h-2 rounded-full bg-[var(--warning)]"
                } else {
                    "w-2 h-2 rounded-full bg-[var(--error)]"
                }
            }

            // Connection text
            span {
                class: "text-[var(--text-secondary)]",
                if total_count == 0 {
                    "No servers"
                } else {
                    "{connected_count}/{total_count} servers"
                }
            }
        }
    }
}

/// Network lag indicator component
#[component]
pub fn NetworkStatus() -> Element {
    // TODO: Implement actual network monitoring
    let mut last_ping = use_signal(|| 0u32); // Milliseconds

    // Simulate network status updates
    use_future(move || async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            // TODO: Actual ping to IRC server
            last_ping.set(fastrand::u32(20..200));
        }
    });

    let ping = last_ping();
    let status_class = if ping < 50 {
        "text-[var(--success)]"
    } else if ping < 150 {
        "text-[var(--warning)]"
    } else {
        "text-[var(--error)]"
    };

    rsx! {
        div {
            class: "flex items-center space-x-1 text-xs {status_class}",
            span { "📶" }
            span { "{ping}ms" }
        }
    }
}

/// Count connected vs total connections
fn count_connections(
    connections: &std::collections::HashMap<String, crate::context::ConnectionInfo>,
) -> (usize, usize) {
    let total = connections.len();
    let connected = connections
        .values()
        .filter(|conn| matches!(conn.state, rustirc_core::ConnectionState::Connected))
        .count();

    (connected, total)
}
