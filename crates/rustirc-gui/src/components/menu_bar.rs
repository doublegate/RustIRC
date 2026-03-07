//! Menu bar component

use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn MenuBar() -> Element {
    let mut show_connect = use_signal(|| false);
    let mut show_preferences = use_signal(|| false);
    let mut show_about = use_signal(|| false);
    let mut active_menu: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div {
            class: "flex items-center bg-[var(--menubar-bg,#252525)] text-[var(--text-color,#e0e0e0)] text-sm border-b border-[var(--border-color,#333)] h-7 select-none",

            MenuButton {
                label: "File",
                active_menu: active_menu,
                items: vec![
                    MenuItem { label: "Connect...".to_string(), action: MenuAction::Connect },
                    MenuItem { label: "Disconnect".to_string(), action: MenuAction::Disconnect },
                    MenuItem { label: "separator".to_string(), action: MenuAction::None },
                    MenuItem { label: "Preferences...".to_string(), action: MenuAction::Preferences },
                    MenuItem { label: "separator".to_string(), action: MenuAction::None },
                    MenuItem { label: "Quit".to_string(), action: MenuAction::Quit },
                ],
                on_action: move |action: MenuAction| {
                    active_menu.set(None);
                    match action {
                        MenuAction::Connect => show_connect.set(true),
                        MenuAction::Preferences => show_preferences.set(true),
                        MenuAction::Quit => std::process::exit(0),
                        _ => {}
                    }
                },
            }

            MenuButton {
                label: "View",
                active_menu: active_menu,
                items: vec![
                    MenuItem { label: "Toggle Server Tree".to_string(), action: MenuAction::ToggleServerTree },
                    MenuItem { label: "Toggle User List".to_string(), action: MenuAction::ToggleUserList },
                ],
                on_action: move |action: MenuAction| {
                    active_menu.set(None);
                    let mut state = use_context::<Signal<AppState>>();
                    match action {
                        MenuAction::ToggleServerTree => {
                            let current = state.read().ui_state.show_server_tree;
                            state.write().ui_state.show_server_tree = !current;
                        }
                        MenuAction::ToggleUserList => {
                            let current = state.read().ui_state.show_user_list;
                            state.write().ui_state.show_user_list = !current;
                        }
                        _ => {}
                    }
                },
            }

            MenuButton {
                label: "Help",
                active_menu: active_menu,
                items: vec![
                    MenuItem { label: "About RustIRC".to_string(), action: MenuAction::About },
                ],
                on_action: move |action: MenuAction| {
                    active_menu.set(None);
                    if let MenuAction::About = action {
                        show_about.set(true);
                    }
                },
            }
        }

        if *show_connect.read() {
            super::dialogs::connect::ConnectDialog {
                on_close: move |_| show_connect.set(false),
            }
        }

        if *show_preferences.read() {
            super::dialogs::preferences::PreferencesDialog {
                on_close: move |_| show_preferences.set(false),
            }
        }

        if *show_about.read() {
            super::dialogs::about::AboutDialog {
                on_close: move |_| show_about.set(false),
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
}

#[derive(Clone, PartialEq)]
pub enum MenuAction {
    None,
    Connect,
    Disconnect,
    Preferences,
    Quit,
    ToggleServerTree,
    ToggleUserList,
    About,
}

#[component]
fn MenuButton(
    label: String,
    active_menu: Signal<Option<String>>,
    items: Vec<MenuItem>,
    on_action: EventHandler<MenuAction>,
) -> Element {
    let is_open = active_menu.read().as_ref() == Some(&label);
    let label_click = label.clone();
    let btn_class = if is_open {
        "px-3 py-0.5 hover:bg-[var(--hover-bg,#333)] bg-[var(--hover-bg,#333)]"
    } else {
        "px-3 py-0.5 hover:bg-[var(--hover-bg,#333)]"
    };

    rsx! {
        div {
            class: "relative",

            button {
                class: "{btn_class}",
                onclick: move |_| {
                    let mut menu = active_menu;
                    if menu.read().as_ref() == Some(&label_click) {
                        menu.set(None);
                    } else {
                        menu.set(Some(label_click.clone()));
                    }
                },
                "{label}"
            }

            if is_open {
                div {
                    class: "absolute top-full left-0 z-50 min-w-[180px] bg-[var(--menu-bg,#2d2d2d)] border border-[var(--border-color,#333)] shadow-lg py-1",

                    for item in items.iter() {
                        if item.label == "separator" {
                            div { class: "border-t border-[var(--border-color,#333)] my-1" }
                        } else {
                            {
                                let action = item.action.clone();
                                rsx! {
                                    button {
                                        class: "block w-full text-left px-4 py-1 hover:bg-[var(--hover-bg,#333)] text-[var(--text-color,#e0e0e0)]",
                                        onclick: move |_| on_action.call(action.clone()),
                                        "{item.label}"
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
