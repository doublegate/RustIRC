//! Message search bar component

use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn SearchBar(on_close: EventHandler<()>) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut query = use_signal(String::new);
    let mut results: Signal<Vec<SearchResult>> = use_signal(Vec::new);

    rsx! {
        div {
            class: "flex items-center gap-2 p-2 bg-[var(--surface-color,#2d2d2d)] border-b border-[var(--border-color,#333)]",

            input {
                class: "flex-1 bg-[var(--input-field-bg,#1e1e1e)] text-[var(--text-color,#e0e0e0)] px-3 py-1 rounded border border-[var(--border-color,#333)] text-sm",
                r#type: "text",
                placeholder: "Search messages...",
                value: "{query}",
                oninput: move |e: Event<FormData>| {
                    let q = e.value();
                    query.set(q.clone());
                    if q.len() >= 2 {
                        let state = app_state.read();
                        let mut found = Vec::new();
                        if let Some(tab) = state.current_tab() {
                            for msg in tab.messages.iter() {
                                if msg.content.to_lowercase().contains(&q.to_lowercase()) {
                                    found.push(SearchResult {
                                        message_id: msg.id,
                                        sender: msg.sender.clone(),
                                        preview: msg.content.clone(),
                                    });
                                }
                            }
                        }
                        results.set(found);
                    } else {
                        results.set(Vec::new());
                    }
                },
            }

            span {
                class: "text-xs text-[var(--text-muted,#888)]",
                {format!("{} results", results.read().len())}
            }

            button {
                class: "text-xs text-[var(--text-muted,#888)] hover:text-[var(--text-color,#e0e0e0)]",
                onclick: move |_| on_close.call(()),
                "Close"
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub message_id: usize,
    pub sender: String,
    pub preview: String,
}
