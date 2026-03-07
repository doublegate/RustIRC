//! About dialog

use dioxus::prelude::*;

#[component]
pub fn AboutDialog(on_close: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center",
            onclick: move |_| on_close.call(()),

            div {
                class: "bg-[var(--surface-color,#2d2d2d)] border border-[var(--border-color,#333)] rounded-lg shadow-xl w-[350px] p-6 text-center",
                onclick: |e| e.stop_propagation(),

                h2 {
                    class: "text-xl font-bold text-[var(--text-color,#e0e0e0)] mb-2",
                    "RustIRC"
                }

                p {
                    class: "text-sm text-[var(--text-muted,#888)] mb-4",
                    {format!("v{}", env!("CARGO_PKG_VERSION"))}
                }

                p {
                    class: "text-sm text-[var(--text-color,#e0e0e0)] mb-4",
                    "A modern IRC client combining the best of mIRC, HexChat, and WeeChat."
                }

                p {
                    class: "text-xs text-[var(--text-muted,#888)] mb-4",
                    "Built with Rust, Dioxus, and Tokio"
                }

                button {
                    class: "px-6 py-1.5 rounded text-sm bg-[var(--accent-color,#4ecdc4)] text-[var(--accent-text,#1a1a1a)] font-medium hover:opacity-90",
                    onclick: move |_| on_close.call(()),
                    "OK"
                }
            }
        }
    }
}
