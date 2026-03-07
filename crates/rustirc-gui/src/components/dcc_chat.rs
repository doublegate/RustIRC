//! DCC chat tab component

use dioxus::prelude::*;

#[component]
pub fn DccChat(peer: String) -> Element {
    let mut messages: Signal<Vec<(String, String)>> = use_signal(Vec::new);
    let mut input = use_signal(String::new);

    rsx! {
        div {
            class: "flex flex-col h-full",

            // Header
            div {
                class: "flex items-center gap-2 p-2 bg-[var(--surface-color,#2d2d2d)] border-b border-[var(--border-color,#333)] text-sm",
                span { class: "font-bold text-[var(--text-color,#e0e0e0)]", "DCC Chat: {peer}" }
                span { class: "text-xs text-[var(--text-muted,#888)]", "(direct connection)" }
            }

            // Messages
            div {
                class: "flex-1 overflow-y-auto p-2 text-sm font-mono",
                for (sender, content) in messages.read().iter() {
                    {
                        let nick_display = format!("<{sender}>");
                        rsx! {
                            div {
                                class: "py-0.5",
                                span { class: "font-bold text-[var(--nick-color,#e06c75)]", "{nick_display} " }
                                span { class: "text-[var(--text-color,#e0e0e0)]", "{content}" }
                            }
                        }
                    }
                }
            }

            // Input
            div {
                class: "p-2 border-t border-[var(--border-color,#333)]",
                input {
                    class: "w-full bg-[var(--input-field-bg,#2d2d2d)] text-[var(--text-color,#e0e0e0)] px-3 py-1.5 rounded border border-[var(--border-color,#333)] text-sm font-mono",
                    r#type: "text",
                    placeholder: "Type a message...",
                    value: "{input}",
                    oninput: move |e: Event<FormData>| input.set(e.value()),
                    onkeydown: move |e: Event<KeyboardData>| {
                        if e.key() == Key::Enter {
                            let text = input.read().clone();
                            if !text.is_empty() {
                                messages.write().push(("me".to_string(), text));
                                input.set(String::new());
                            }
                        }
                    },
                }
            }
        }
    }
}
