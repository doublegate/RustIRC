//! Connect to server dialog

use crate::hooks::IrcActions;
use dioxus::prelude::*;

#[component]
pub fn ConnectDialog(on_close: EventHandler<()>) -> Element {
    let actions = use_context::<IrcActions>();
    let mut server_address = use_signal(|| "irc.libera.chat".to_string());
    let mut port = use_signal(|| "6697".to_string());
    let mut nickname = use_signal(|| "RustIRC_User".to_string());
    let mut use_tls = use_signal(|| true);

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center",
            onclick: move |_| on_close.call(()),

            // Dialog
            div {
                class: "bg-[var(--surface-color,#2d2d2d)] border border-[var(--border-color,#333)] rounded-lg shadow-xl w-[400px] p-6",
                onclick: |e| e.stop_propagation(),

                h2 {
                    class: "text-lg font-bold text-[var(--text-color,#e0e0e0)] mb-4",
                    "Connect to Server"
                }

                // Server address
                div {
                    class: "mb-3",
                    label {
                        class: "block text-xs text-[var(--text-muted,#888)] mb-1",
                        "Server"
                    }
                    input {
                        class: "w-full bg-[var(--input-field-bg,#1e1e1e)] text-[var(--text-color,#e0e0e0)] px-3 py-1.5 rounded border border-[var(--border-color,#333)] text-sm",
                        r#type: "text",
                        value: "{server_address}",
                        oninput: move |e| server_address.set(e.value()),
                    }
                }

                // Port
                div {
                    class: "mb-3",
                    label {
                        class: "block text-xs text-[var(--text-muted,#888)] mb-1",
                        "Port"
                    }
                    input {
                        class: "w-full bg-[var(--input-field-bg,#1e1e1e)] text-[var(--text-color,#e0e0e0)] px-3 py-1.5 rounded border border-[var(--border-color,#333)] text-sm",
                        r#type: "text",
                        value: "{port}",
                        oninput: move |e| port.set(e.value()),
                    }
                }

                // Nickname
                div {
                    class: "mb-3",
                    label {
                        class: "block text-xs text-[var(--text-muted,#888)] mb-1",
                        "Nickname"
                    }
                    input {
                        class: "w-full bg-[var(--input-field-bg,#1e1e1e)] text-[var(--text-color,#e0e0e0)] px-3 py-1.5 rounded border border-[var(--border-color,#333)] text-sm",
                        r#type: "text",
                        value: "{nickname}",
                        oninput: move |e| nickname.set(e.value()),
                    }
                }

                // TLS checkbox
                div {
                    class: "mb-4 flex items-center gap-2",
                    input {
                        class: "accent-[var(--accent-color,#4ecdc4)]",
                        r#type: "checkbox",
                        checked: *use_tls.read(),
                        onchange: move |e| use_tls.set(e.checked()),
                    }
                    label {
                        class: "text-sm text-[var(--text-color,#e0e0e0)]",
                        "Use TLS"
                    }
                }

                // Buttons
                div {
                    class: "flex justify-end gap-2",
                    button {
                        class: "px-4 py-1.5 rounded text-sm text-[var(--text-muted,#888)] hover:bg-[var(--hover-bg,#333)]",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "px-4 py-1.5 rounded text-sm bg-[var(--accent-color,#4ecdc4)] text-[var(--accent-text,#1a1a1a)] font-medium hover:opacity-90",
                        onclick: move |_| {
                            let addr = server_address.read().clone();
                            let port_str = port.read().clone();
                            let server_id = format!("{addr}:{port_str}");
                            actions.connect(&server_id, &addr);
                            on_close.call(());
                        },
                        "Connect"
                    }
                }
            }
        }
    }
}
