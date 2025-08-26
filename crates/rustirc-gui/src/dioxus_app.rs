//! Main Dioxus application component
//!
//! This is the new Dioxus v0.6 implementation replacing the Iced GUI.
//! Features:
//! - React-like component architecture with hooks
//! - Modern CSS Grid layout with Tailwind CSS  
//! - Context API for global state management
//! - Hot reloading for development
//! - Cross-platform desktop support

use crate::components::{
    input_area::InputArea, message_view::MessageView, sidebar::Sidebar, status_bar::StatusBar,
    tab_bar::TabBar, user_list::UserList,
};
use crate::context::{ContextProvider, DialogType, IrcState, ThemeState, ThemeType, UiState};
use crate::dialogs::{AboutDialog, ConnectDialog, SettingsDialog};
use dioxus::prelude::*;

/// Main application entry point
pub fn launch_app() -> anyhow::Result<()> {
    // Launch the Dioxus desktop application
    dioxus::launch(App);
    Ok(())
}

/// Root application component
#[component]
fn App() -> Element {
    rsx! {
        ContextProvider {
            div {
                class: "irc-window font-mono",
                id: "app-root",
                AppShell {}
                DialogProvider {}
                ContextMenuProvider {}
            }
        }
    }
}

/// Main application shell with layout
#[component]
fn AppShell() -> Element {
    let ui_state = use_context::<UiState>();
    let user_list_visible = ui_state.user_list_visible.read();
    let sidebar_width = ui_state.sidebar_width.read();

    rsx! {
        div {
            class: if *user_list_visible {
                "h-full grid grid-cols-[{sidebar_width}px_1fr_200px] grid-rows-irc-layout"
            } else {
                "h-full grid grid-cols-irc-layout-no-userlist grid-rows-irc-layout"
            },

            // Tab bar (spans full width)
            header {
                class: "col-span-full border-b border-[var(--border-color)] irc-panel",
                TabBar {}
            }

            // Sidebar with server/channel list
            aside {
                class: "border-r border-[var(--border-color)] overflow-auto custom-scrollbar irc-panel",
                Sidebar {}
            }

            // Main content area
            main {
                class: "flex flex-col overflow-hidden",

                // Message view (expandable)
                div {
                    class: "flex-1 overflow-hidden",
                    MessageView {}
                }

                // Input area (fixed at bottom)
                div {
                    class: "border-t border-[var(--border-color)] irc-panel",
                    InputArea {}
                }
            }

            // User list (conditionally rendered)
            if *user_list_visible {
                aside {
                    class: "border-l border-[var(--border-color)] overflow-auto custom-scrollbar irc-panel",
                    UserList {}
                }
            }

            // Status bar (spans full width)
            footer {
                class: "col-span-full border-t border-[var(--border-color)] irc-panel",
                StatusBar {}
            }
        }
    }
}

/// Dialog provider for modal dialogs
#[component]
fn DialogProvider() -> Element {
    let ui_state = use_context::<UiState>();
    let active_dialogs = ui_state.active_dialogs.read();

    rsx! {
        for dialog in active_dialogs.iter() {
            match dialog {
                DialogType::Connect => rsx! { ConnectDialog {} },
                DialogType::Settings => rsx! { SettingsDialog {} },
                DialogType::About => rsx! { AboutDialog {} },
                DialogType::ChannelList => rsx! {
                    div { class: "modal-backdrop", "Channel List Coming Soon..." }
                },
                DialogType::UserInfo(username) => rsx! {
                    div { class: "modal-backdrop", "User Info for {username} Coming Soon..." }
                },
            }
        }
    }
}

/// Context menu provider for right-click menus
#[component]
fn ContextMenuProvider() -> Element {
    let ui_state = use_context::<UiState>();
    let context_menu_pos = ui_state.context_menu_position.read();

    rsx! {
        if let Some((x, y)) = *context_menu_pos {
            div {
                class: "fixed context-menu z-50",
                style: "left: {x}px; top: {y}px;",

                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        let ui_state = use_context::<UiState>();
                        ui_state.show_dialog(DialogType::Connect);
                        ui_state.hide_context_menu();
                    },
                    "Connect to Server"
                }
                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        let ui_state = use_context::<UiState>();
                        ui_state.show_dialog(DialogType::Settings);
                        ui_state.hide_context_menu();
                    },
                    "Settings"
                }
                div {
                    class: "context-menu-item",
                    onclick: move |_| {
                        let ui_state = use_context::<UiState>();
                        ui_state.show_dialog(DialogType::ChannelList);
                        ui_state.hide_context_menu();
                    },
                    "Channel List"
                }
                hr { class: "border-[var(--border-color)] my-1" }
                div {
                    class: "context-menu-item text-[var(--text-muted)]",
                    onclick: move |_| {
                        let ui_state = use_context::<UiState>();
                        ui_state.hide_context_menu();
                    },
                    "Close Menu"
                }
            }
        }
    }
}

/// Keyboard shortcut handler component
#[component]
fn KeyboardShortcuts() -> Element {
    let ui_state = use_context::<UiState>();
    let _theme_state = use_context::<ThemeState>();

    // Handle global keyboard shortcuts (desktop version)
    use_effect(move || {
        // In desktop Dioxus apps, keyboard shortcuts are handled by the main window
        // or through component event handlers. Global shortcuts would be registered
        // with the desktop application framework.

        let _ui_state = ui_state.clone();

        // Desktop keyboard shortcut setup would happen here
        move || {
            // Desktop cleanup if needed
        }
    });

    rsx! {
        // This component doesn't render anything visible
        span { hidden: true }
    }
}

/// Theme provider component that injects CSS
#[component]
fn ThemeProvider() -> Element {
    let theme_state = use_context::<ThemeState>();
    let current_theme = theme_state.current_theme.read();
    let custom_css = theme_state.custom_css.read();

    // Update document theme attribute
    use_effect(move || {
        let document = web_sys::window().unwrap().document().unwrap();
        let html_element = document.document_element().unwrap();

        let theme_name = match *current_theme {
            ThemeType::Dark => "dark",
            ThemeType::Light => "light",
            ThemeType::Discord => "discord",
            ThemeType::Nord => "nord",
            _ => "dark",
        };

        html_element
            .set_attribute("data-theme", theme_name)
            .unwrap();
    });

    rsx! {
        // Inject custom CSS
        if !custom_css.is_empty() {
            style {
                dangerous_inner_html: "{custom_css}"
            }
        }

        // Include Tailwind CSS
        link {
            rel: "stylesheet",
            href: "/assets/style.css"
        }
    }
}
