//! Input area component for typing messages and commands

use crate::hooks::IrcActions;
use crate::state::{AppState, TabType};
use dioxus::prelude::*;

#[component]
pub fn InputArea() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let actions = use_context::<IrcActions>();
    let mut input_text = use_signal(String::new);
    let mut history: Signal<Vec<String>> = use_signal(Vec::new);
    let mut history_index: Signal<Option<usize>> = use_signal(|| None);

    let state = app_state.read();
    let has_tab = state.current_tab_id.is_some();

    let current_target = state.current_tab().map(|tab| match &tab.tab_type {
        TabType::Channel { channel } => channel.clone(),
        TabType::PrivateMessage { nick } => nick.clone(),
        TabType::Server => tab.server_id.clone().unwrap_or_default(),
    });

    let current_server = state.current_tab().and_then(|tab| tab.server_id.clone());

    let target_display = current_target.clone().unwrap_or_default();

    rsx! {
        div {
            class: "flex items-center gap-2 p-2 bg-[var(--input-bg,#1e1e1e)] border-t border-[var(--border-color,#333)]",

            if !target_display.is_empty() {
                span {
                    class: "text-xs text-[var(--text-muted,#888)] flex-shrink-0",
                    "{target_display}"
                }
            }

            input {
                class: "flex-1 bg-[var(--input-field-bg,#2d2d2d)] text-[var(--text-color,#e0e0e0)] px-3 py-1.5 rounded border border-[var(--border-color,#333)] focus:border-[var(--accent-color,#4ecdc4)] focus:outline-none text-sm font-mono",
                r#type: "text",
                placeholder: if has_tab { "Type a message..." } else { "Connect to a server first" },
                disabled: !has_tab,
                value: "{input_text}",
                oninput: move |e: Event<FormData>| input_text.set(e.value()),
                onkeydown: move |e: Event<KeyboardData>| {
                    match e.key() {
                        Key::Enter => {
                            let text = input_text.read().clone();
                            if text.is_empty() {
                                return;
                            }

                            history.write().push(text.clone());
                            history_index.set(None);

                            if let (Some(ref server_id), Some(ref target)) = (&current_server, &current_target) {
                                if text.starts_with('/') {
                                    handle_command(&actions, server_id, target, &text);
                                } else {
                                    actions.send_message(server_id, target, &text);
                                }
                            }

                            input_text.set(String::new());
                        }
                        Key::ArrowUp => {
                            let hist = history.read();
                            if hist.is_empty() {
                                return;
                            }
                            let current_idx = *history_index.read();
                            let idx = match current_idx {
                                Some(i) if i > 0 => i - 1,
                                None => hist.len() - 1,
                                Some(i) => i,
                            };
                            if let Some(entry) = hist.get(idx).cloned() {
                                drop(hist);
                                history_index.set(Some(idx));
                                input_text.set(entry);
                            }
                        }
                        Key::ArrowDown => {
                            let hist = history.read();
                            let current_idx = *history_index.read();
                            if let Some(idx) = current_idx {
                                if idx + 1 < hist.len() {
                                    let new_idx = idx + 1;
                                    if let Some(entry) = hist.get(new_idx).cloned() {
                                        drop(hist);
                                        history_index.set(Some(new_idx));
                                        input_text.set(entry);
                                    }
                                } else {
                                    drop(hist);
                                    history_index.set(None);
                                    input_text.set(String::new());
                                }
                            }
                        }
                        _ => {}
                    }
                },
            }
        }
    }
}

fn handle_command(actions: &IrcActions, server_id: &str, target: &str, input: &str) {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let args = parts.get(1).unwrap_or(&"");

    match command.as_str() {
        "/join" | "/j" => {
            let channel = if args.starts_with('#') || args.starts_with('&') {
                args.to_string()
            } else {
                format!("#{args}")
            };
            actions.join_channel(server_id, &channel);
        }
        "/part" | "/leave" => {
            let channel = if args.is_empty() {
                target.to_string()
            } else {
                args.to_string()
            };
            actions.leave_channel(server_id, &channel);
        }
        "/msg" => {
            if let Some((nick, message)) = args.split_once(' ') {
                actions.send_message(server_id, nick, message);
            }
        }
        "/quit" | "/q" => {
            actions.disconnect(server_id);
        }
        _ => {
            tracing::warn!("Unknown command: {}", command);
        }
    }
}
