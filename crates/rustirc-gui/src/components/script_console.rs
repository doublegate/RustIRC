//! Script console component for Lua scripting interaction

use dioxus::prelude::*;

#[component]
pub fn ScriptConsole() -> Element {
    let mut output: Signal<Vec<String>> = use_signal(Vec::new);
    let mut input = use_signal(String::new);

    rsx! {
        div {
            class: "flex flex-col h-full",

            div {
                class: "flex items-center gap-2 p-2 bg-[var(--surface-color,#2d2d2d)] border-b border-[var(--border-color,#333)] text-sm",
                span { class: "font-bold text-[var(--text-color,#e0e0e0)]", "Script Console" }
                span { class: "text-xs text-[var(--text-muted,#888)]", "(Lua)" }
            }

            div {
                class: "flex-1 overflow-y-auto p-2 text-xs font-mono bg-[var(--bg-color,#1a1a1a)]",
                for line in output.read().iter() {
                    div {
                        class: "text-[var(--text-color,#e0e0e0)] py-0.5",
                        "{line}"
                    }
                }
            }

            div {
                class: "flex items-center gap-2 p-2 border-t border-[var(--border-color,#333)]",
                span { class: "text-xs text-[var(--accent-color,#4ecdc4)]", ">" }
                input {
                    class: "flex-1 bg-[var(--input-field-bg,#2d2d2d)] text-[var(--text-color,#e0e0e0)] px-2 py-1 rounded border border-[var(--border-color,#333)] text-xs font-mono",
                    r#type: "text",
                    placeholder: "Enter Lua command...",
                    value: "{input}",
                    oninput: move |e: Event<FormData>| input.set(e.value()),
                    onkeydown: move |e: Event<KeyboardData>| {
                        if e.key() == Key::Enter {
                            let cmd = input.read().clone();
                            if !cmd.is_empty() {
                                output.write().push(format!("> {cmd}"));
                                output.write().push("(script execution not connected)".to_string());
                                input.set(String::new());
                            }
                        }
                    },
                }
            }
        }
    }
}
