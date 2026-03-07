//! Message view component for displaying IRC messages
//!
//! Renders IRC messages with formatting codes converted to styled HTML spans.

use crate::formatting;
use crate::state::{AppState, DisplayMessage, MessageType};
use dioxus::prelude::*;

#[component]
pub fn MessageView() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let state = app_state.read();

    let current_tab = state.current_tab();

    let Some(tab) = current_tab else {
        return rsx! {
            div {
                class: "flex items-center justify-center h-full text-[var(--text-muted,#888)]",
                "Welcome to RustIRC. Connect to a server to get started."
            }
        };
    };

    let show_timestamps = state.settings.show_timestamps;
    let timestamp_format = state.settings.timestamp_format.clone();
    let messages: Vec<DisplayMessage> = tab.messages.iter().cloned().collect();

    rsx! {
        div {
            class: "flex flex-col p-2 text-sm font-mono",
            id: "message-view",

            for msg in messages.iter() {
                {render_message(msg, show_timestamps, &timestamp_format)}
            }
        }
    }
}

fn render_message(msg: &DisplayMessage, show_timestamps: bool, timestamp_format: &str) -> Element {
    let type_class = match msg.message_type {
        MessageType::Join | MessageType::Part | MessageType::Quit => {
            "text-[var(--join-part-color,#666)]"
        }
        MessageType::Action => "text-[var(--action-color,#d19a66)]",
        MessageType::Notice => "text-[var(--notice-color,#61afef)]",
        MessageType::System | MessageType::Mode | MessageType::Topic => {
            "text-[var(--system-color,#888)]"
        }
        MessageType::Nick => "text-[var(--nick-change-color,#c678dd)]",
        _ => "",
    };

    let highlight_class = if msg.is_highlight {
        "bg-[var(--highlight-bg,rgba(255,107,107,0.1))]"
    } else {
        ""
    };

    let timestamp_str = if show_timestamps {
        let datetime: chrono::DateTime<chrono::Local> = msg.timestamp.into();
        datetime.format(timestamp_format).to_string()
    } else {
        String::new()
    };

    let spans = formatting::parse_irc_text(&msg.content);

    rsx! {
        div {
            class: "flex gap-2 py-0.5 leading-5 {type_class} {highlight_class}",

            // Timestamp
            if show_timestamps {
                span {
                    class: "text-[var(--timestamp-color,#555)] flex-shrink-0 select-none",
                    "[{timestamp_str}]"
                }
            }

            // Sender
            if msg.message_type == MessageType::Action {
                span {
                    class: "text-[var(--action-color,#d19a66)]",
                    "* {msg.sender}"
                }
            } else if msg.message_type != MessageType::System {
                {
                    let sender_class = if msg.is_own_message {
                        "font-bold flex-shrink-0 text-[var(--own-nick-color,#61afef)]"
                    } else {
                        "font-bold flex-shrink-0 text-[var(--nick-color,#e06c75)]"
                    };
                    let nick_display = format!("<{}>", msg.sender);
                    rsx! {
                        span {
                            class: "{sender_class}",
                            "{nick_display}"
                        }
                    }
                }
            } else {
                span {
                    class: "text-[var(--system-color,#888)] flex-shrink-0",
                    "***"
                }
            }

            // Message content with IRC formatting
            span {
                class: "break-words min-w-0",
                for span in spans.iter() {
                    {render_span(span)}
                }
            }
        }
    }
}

fn render_span(span: &formatting::TextSpan) -> Element {
    let style = span.to_css_style();
    let classes = span.to_css_classes();

    if span.is_url {
        let url = span.url_target.clone().unwrap_or_default();
        rsx! {
            a {
                class: "text-[var(--link-color,#0088ff)] underline hover:opacity-80 {classes}",
                style: "{style}",
                href: "{url}",
                target: "_blank",
                onclick: move |e| {
                    e.prevent_default();
                    let _ = open::that(&url);
                },
                "{span.text}"
            }
        }
    } else if style.is_empty() && classes.is_empty() {
        rsx! { "{span.text}" }
    } else {
        rsx! {
            span {
                class: "{classes}",
                style: "{style}",
                "{span.text}"
            }
        }
    }
}
