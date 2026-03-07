//! Plugin manager UI component

use dioxus::prelude::*;

#[derive(Clone, Debug)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}

#[component]
pub fn PluginManager() -> Element {
    let plugins = use_signal(|| {
        vec![
            PluginInfo {
                name: "Logger".to_string(),
                version: "1.0.0".to_string(),
                description: "Logs all messages to disk".to_string(),
                enabled: true,
            },
            PluginInfo {
                name: "Highlight".to_string(),
                version: "1.0.0".to_string(),
                description: "Highlights messages matching keywords".to_string(),
                enabled: true,
            },
        ]
    });

    rsx! {
        div {
            class: "flex flex-col h-full p-4",

            h2 {
                class: "text-lg font-bold text-[var(--text-color,#e0e0e0)] mb-4",
                "Plugin Manager"
            }

            div {
                class: "space-y-2",
                for plugin in plugins.read().iter() {
                    div {
                        class: "flex items-center justify-between p-3 bg-[var(--surface-color,#2d2d2d)] rounded border border-[var(--border-color,#333)]",

                        div {
                            class: "flex-1",
                            div {
                                class: "flex items-center gap-2",
                                span { class: "font-medium text-[var(--text-color,#e0e0e0)]", "{plugin.name}" }
                                span { class: "text-xs text-[var(--text-muted,#888)]", "v{plugin.version}" }
                            }
                            p {
                                class: "text-xs text-[var(--text-muted,#888)] mt-0.5",
                                "{plugin.description}"
                            }
                        }

                        div {
                            class: "flex items-center gap-2",
                            span {
                                class: if plugin.enabled { "text-xs text-green-500" } else { "text-xs text-red-500" },
                                if plugin.enabled { "Enabled" } else { "Disabled" }
                            }
                        }
                    }
                }
            }
        }
    }
}
