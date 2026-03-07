//! Preferences dialog

use crate::hooks::use_theme::ThemeType;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn PreferencesDialog(on_close: EventHandler<()>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut theme = use_context::<Signal<ThemeType>>();
    let state = app_state.read();

    let mut show_timestamps = use_signal(|| state.settings.show_timestamps);
    let mut show_join_part = use_signal(|| state.settings.show_join_part);
    let mut compact_mode = use_signal(|| state.settings.compact_mode);
    let mut nick_colors = use_signal(|| state.settings.nick_colors);
    let mut font_size = use_signal(|| state.settings.font_size.to_string());

    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 z-50 flex items-center justify-center",
            onclick: move |_| on_close.call(()),

            div {
                class: "bg-[var(--surface-color,#2d2d2d)] border border-[var(--border-color,#333)] rounded-lg shadow-xl w-[450px] p-6 max-h-[80vh] overflow-y-auto",
                onclick: |e| e.stop_propagation(),

                h2 {
                    class: "text-lg font-bold text-[var(--text-color,#e0e0e0)] mb-4",
                    "Preferences"
                }

                // Theme
                div {
                    class: "mb-4",
                    label {
                        class: "block text-xs text-[var(--text-muted,#888)] mb-1",
                        "Theme"
                    }
                    select {
                        class: "w-full bg-[var(--input-field-bg,#1e1e1e)] text-[var(--text-color,#e0e0e0)] px-3 py-1.5 rounded border border-[var(--border-color,#333)] text-sm",
                        onchange: move |e| {
                            let new_theme: ThemeType = e.value().parse().unwrap_or(ThemeType::Dark);
                            theme.set(new_theme);
                        },
                        for t in ThemeType::all().iter() {
                            option {
                                value: "{t.as_str()}",
                                selected: *theme.read() == *t,
                                "{t.display_name()}"
                            }
                        }
                    }
                }

                // Font size
                div {
                    class: "mb-4",
                    label {
                        class: "block text-xs text-[var(--text-muted,#888)] mb-1",
                        "Font Size"
                    }
                    input {
                        class: "w-full bg-[var(--input-field-bg,#1e1e1e)] text-[var(--text-color,#e0e0e0)] px-3 py-1.5 rounded border border-[var(--border-color,#333)] text-sm",
                        r#type: "number",
                        min: "8",
                        max: "24",
                        value: "{font_size}",
                        oninput: move |e| font_size.set(e.value()),
                    }
                }

                // Checkboxes
                div { class: "space-y-2 mb-4",
                    label { class: "flex items-center gap-2 text-sm text-[var(--text-color,#e0e0e0)]",
                        input {
                            r#type: "checkbox",
                            checked: *show_timestamps.read(),
                            onchange: move |e| show_timestamps.set(e.checked()),
                        }
                        "Show timestamps"
                    }
                    label { class: "flex items-center gap-2 text-sm text-[var(--text-color,#e0e0e0)]",
                        input {
                            r#type: "checkbox",
                            checked: *show_join_part.read(),
                            onchange: move |e| show_join_part.set(e.checked()),
                        }
                        "Show join/part messages"
                    }
                    label { class: "flex items-center gap-2 text-sm text-[var(--text-color,#e0e0e0)]",
                        input {
                            r#type: "checkbox",
                            checked: *compact_mode.read(),
                            onchange: move |e| compact_mode.set(e.checked()),
                        }
                        "Compact mode"
                    }
                    label { class: "flex items-center gap-2 text-sm text-[var(--text-color,#e0e0e0)]",
                        input {
                            r#type: "checkbox",
                            checked: *nick_colors.read(),
                            onchange: move |e| nick_colors.set(e.checked()),
                        }
                        "Colorize nicknames"
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
                            let mut state = app_state.write();
                            state.settings.show_timestamps = *show_timestamps.read();
                            state.settings.show_join_part = *show_join_part.read();
                            state.settings.compact_mode = *compact_mode.read();
                            state.settings.nick_colors = *nick_colors.read();
                            if let Ok(size) = font_size.read().parse::<f32>() {
                                state.settings.font_size = size;
                            }
                            let _ = state.settings.save();
                            on_close.call(());
                        },
                        "Save"
                    }
                }
            }
        }
    }
}
