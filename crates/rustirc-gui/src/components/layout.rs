//! Main 3-pane layout component
//!
//! Provides the primary application layout with server tree, message view, and user list.

use crate::hooks::use_theme::ThemeType;
use crate::state::AppState;
use dioxus::prelude::*;

use super::input_area::InputArea;
use super::menu_bar::MenuBar;
use super::message_view::MessageView;
use super::server_tree::ServerTree;
use super::status_bar::StatusBar;
use super::tab_bar::TabBar;
use super::user_list::UserList;

#[component]
pub fn MainLayout() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let theme = use_context::<Signal<ThemeType>>();
    let state = app_state.read();

    let theme_attr = theme.read().as_str().to_string();
    let show_server_tree = state.ui_state.show_server_tree;
    let show_user_list = state.ui_state.show_user_list;

    rsx! {
        div {
            class: "flex flex-col h-screen w-screen overflow-hidden",
            "data-theme": "{theme_attr}",

            // Menu bar
            MenuBar {}

            // Tab bar
            TabBar {}

            // Main content area: 3-pane layout
            div {
                class: "flex flex-1 overflow-hidden",

                // Left pane: Server tree
                if show_server_tree {
                    div {
                        class: "w-52 flex-shrink-0 overflow-y-auto border-r border-[var(--border-color,#333)]",
                        ServerTree {}
                    }
                }

                // Center pane: Messages + Input
                div {
                    class: "flex flex-col flex-1 overflow-hidden",
                    div {
                        class: "flex-1 overflow-y-auto",
                        MessageView {}
                    }
                    InputArea {}
                }

                // Right pane: User list
                if show_user_list {
                    div {
                        class: "w-40 flex-shrink-0 overflow-y-auto border-l border-[var(--border-color,#333)]",
                        UserList {}
                    }
                }
            }

            // Status bar
            StatusBar {}
        }
    }
}
