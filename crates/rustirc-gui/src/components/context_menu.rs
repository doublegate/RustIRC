//! Context menu component for right-click actions

use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ContextMenuEntry {
    pub label: String,
    pub action: String,
}

#[component]
pub fn ContextMenu(
    x: f64,
    y: f64,
    entries: Vec<ContextMenuEntry>,
    on_action: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        // Overlay to capture clicks outside
        div {
            class: "fixed inset-0 z-40",
            onclick: move |_| on_close.call(()),
        }

        // Menu
        div {
            class: "fixed z-50 min-w-[150px] bg-[var(--menu-bg,#2d2d2d)] border border-[var(--border-color,#333)] shadow-lg rounded py-1 text-sm",
            style: "left: {x}px; top: {y}px;",

            for entry in entries.iter() {
                {
                    let action = entry.action.clone();
                    rsx! {
                        button {
                            class: "block w-full text-left px-4 py-1 hover:bg-[var(--hover-bg,#333)] text-[var(--text-color,#e0e0e0)]",
                            onclick: move |_| on_action.call(action.clone()),
                            "{entry.label}"
                        }
                    }
                }
            }
        }
    }
}
