//! Tab bar component for channel/server navigation

use crate::hooks::IrcActions;
use crate::state::{ActivityLevel, AppState};
use dioxus::prelude::*;

#[component]
pub fn TabBar() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let actions = use_context::<IrcActions>();
    let state = app_state.read();

    let current_tab_id = state.current_tab_id.clone();

    rsx! {
        div {
            class: "flex items-center bg-[var(--tab-bg,#1e1e1e)] border-b border-[var(--border-color,#333)] h-9 overflow-x-auto",

            for tab_id in state.tab_order.iter() {
                {
                    let tab = state.tabs.get(tab_id);
                    let is_active = current_tab_id.as_ref() == Some(tab_id);
                    let tab_name = tab.map(|t| t.name.clone()).unwrap_or_default();
                    let has_activity = tab.map(|t| t.has_activity).unwrap_or(false);
                    let has_highlight = tab.map(|t| t.has_highlight).unwrap_or(false);
                    let activity_level = tab.map(|t| t.activity.clone()).unwrap_or(ActivityLevel::None);

                    let active_class = if is_active {
                        "bg-[var(--surface-color,#2d2d2d)] text-[var(--text-color,#e0e0e0)]"
                    } else {
                        "text-[var(--text-muted,#888)] hover:bg-[var(--hover-bg,#333)]"
                    };

                    let activity_class = match activity_level {
                        ActivityLevel::Highlight | ActivityLevel::Mention => "text-[var(--highlight-color,#ff6b6b)]",
                        ActivityLevel::Activity if !is_active => "text-[var(--activity-color,#4ecdc4)]",
                        _ => "",
                    };

                    let tab_id_click = tab_id.clone();
                    let tab_id_close = tab_id.clone();

                    rsx! {
                        div {
                            class: "flex items-center px-3 py-1 cursor-pointer select-none whitespace-nowrap text-sm border-r border-[var(--border-color,#333)] {active_class} {activity_class}",
                            onclick: move |_| actions.switch_tab(&tab_id_click),

                            // Activity indicator dot
                            if has_activity && !is_active {
                                span { class: "w-2 h-2 rounded-full bg-[var(--activity-color,#4ecdc4)] mr-1.5" }
                            }
                            if has_highlight {
                                span { class: "w-2 h-2 rounded-full bg-[var(--highlight-color,#ff6b6b)] mr-1.5" }
                            }

                            span { "{tab_name}" }

                            // Close button
                            button {
                                class: "ml-2 text-xs opacity-50 hover:opacity-100",
                                onclick: move |e: Event<MouseData>| {
                                    e.stop_propagation();
                                    actions.close_tab(&tab_id_close);
                                },
                                "x"
                            }
                        }
                    }
                }
            }
        }
    }
}
