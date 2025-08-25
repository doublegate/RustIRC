//! Message view component for displaying IRC messages

use crate::context::{IrcState, UiState, ChatMessage, MessageType};
use dioxus::prelude::*;
use chrono::{DateTime, Utc};

/// Message view component for displaying chat messages
#[component]
pub fn MessageView() -> Element {
    let irc_state = use_context::<IrcState>();
    let ui_state = use_context::<UiState>();
    let active_tab = irc_state.active_tab.read();
    let connections = irc_state.connections.read();
    let system_messages_visible = ui_state.system_messages_visible.read();
    let joins_parts_visible = ui_state.joins_parts_visible.read();
    
    // Get messages for the current channel/server
    let messages = get_messages_for_tab(&*active_tab, &*connections);
    
    // Filter messages based on preferences  
    let filtered_messages: Vec<&ChatMessage> = messages.iter()
        .filter(|msg| should_show_message(msg, *system_messages_visible, *joins_parts_visible))
        .collect();

    rsx! {
        div { 
            class: "h-full flex flex-col",
            
            // Messages container (scrollable)
            div { 
                class: "flex-1 overflow-y-auto custom-scrollbar",
                id: "messages-container",
                
                if *active_tab == "welcome" {
                    WelcomeView {}
                } else if filtered_messages.is_empty() {
                    EmptyView { tab: active_tab.clone() }
                } else {
                    div { 
                        class: "p-2 space-y-1",
                        for message in filtered_messages.iter() {
                            MessageItem {
                                key: "{message.id}",
                                message: (*message).clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Welcome screen for the default tab
#[component]
fn WelcomeView() -> Element {
    let ui_state = use_context::<UiState>();
    
    rsx! {
        div { 
            class: "h-full flex flex-col items-center justify-center text-center p-8",
            
            div { 
                class: "max-w-md mx-auto space-y-6",
                
                // Logo/Title
                div { 
                    class: "space-y-2",
                    h1 { 
                        class: "text-3xl font-bold text-[var(--text-primary)]",
                        "RustIRC"
                    }
                    p { 
                        class: "text-[var(--text-secondary)]",
                        "Modern IRC client built with Dioxus"
                    }
                }
                
                // Quick actions
                div { 
                    class: "space-y-3",
                    button {
                        class: "irc-button w-full py-3 text-base",
                        onclick: move |_| {
                            ui_state.show_dialog(crate::context::DialogType::Connect);
                        },
                        "Connect to IRC Server"
                    }
                    
                    button {
                        class: "w-full py-2 px-4 rounded border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors",
                        onclick: move |_| {
                            ui_state.show_dialog(crate::context::DialogType::Settings);
                        },
                        "Settings"
                    }
                }
                
                // Help text
                div { 
                    class: "text-sm text-[var(--text-muted)] space-y-2",
                    p { "Keyboard shortcuts:" }
                    ul { 
                        class: "text-left space-y-1 max-w-48 mx-auto",
                        li { "Ctrl+K - Connect to server" }
                        li { "Ctrl+, - Settings" }
                        li { "F1 - About" }
                        li { "Esc - Close dialogs" }
                    }
                }
            }
        }
    }
}

/// Empty state when no messages in current channel
#[component] 
fn EmptyView(tab: String) -> Element {
    rsx! {
        div { 
            class: "h-full flex items-center justify-center text-center p-8",
            div { 
                class: "text-[var(--text-muted)] space-y-2",
                p { 
                    class: "text-lg",
                    if tab.contains(':') {
                        "No messages in this channel yet"
                    } else {
                        "No messages from this server yet"  
                    }
                }
                p { 
                    class: "text-sm",
                    "Start typing a message to begin the conversation"
                }
            }
        }
    }
}

/// Individual message item
#[component]
fn MessageItem(message: ChatMessage) -> Element {
    let message_class = match message.message_type {
        MessageType::Normal => "irc-message",
        MessageType::Action => "irc-message irc-message-action",
        MessageType::System | MessageType::Join | MessageType::Part | MessageType::Quit | MessageType::Nick | MessageType::Topic => "irc-message irc-message-system",
        MessageType::Error => "irc-message irc-message-error",
    };
    
    let timestamp = message.timestamp.format("%H:%M:%S");

    rsx! {
        div { 
            class: "{message_class}",
            
            div { 
                class: "flex items-start space-x-3",
                
                // Timestamp
                span { 
                    class: "text-xs text-[var(--text-muted)] mt-0.5 w-16 flex-shrink-0",
                    "{timestamp}"
                }
                
                // Message content
                div { 
                    class: "flex-1 min-w-0",
                    
                    match message.message_type {
                        MessageType::Normal => rsx! {
                            div { 
                                class: "flex items-baseline space-x-2",
                                if let Some(sender) = &message.sender {
                                    span { 
                                        class: "font-medium text-[var(--accent-primary)] cursor-pointer hover:underline",
                                        onclick: move |_| {
                                            let ui_state = use_context::<UiState>();
                                            ui_state.show_dialog(crate::context::DialogType::UserInfo(sender.clone()));
                                        },
                                        "{sender}"
                                    }
                                }
                                div { 
                                    class: "break-words",
                                    dangerous_inner_html: "{format_message_content(&message.content)}"
                                }
                            }
                        },
                        MessageType::Action => rsx! {
                            div { 
                                class: "italic",
                                if let Some(sender) = &message.sender {
                                    span { "* {sender} {message.content}" }
                                } else {
                                    span { "* {message.content}" }
                                }
                            }
                        },
                        _ => rsx! {
                            div { 
                                class: "text-sm",
                                dangerous_inner_html: "{format_message_content(&message.content)}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Get messages for the current tab
fn get_messages_for_tab(tab_id: &str, connections: &std::collections::HashMap<String, crate::context::ConnectionInfo>) -> Vec<ChatMessage> {
    if tab_id == "welcome" {
        return Vec::new();
    }
    
    if tab_id.contains(':') {
        // Channel tab
        let parts: Vec<&str> = tab_id.split(':').collect();
        if parts.len() == 2 {
            let server_id = parts[0];
            let channel_name = parts[1];
            
            if let Some(connection) = connections.get(server_id) {
                if let Some(channel) = connection.channels.get(channel_name) {
                    return channel.messages.clone();
                }
            }
        }
    } else {
        // Server tab - return system messages or server notices
        if let Some(connection) = connections.get(tab_id) {
            // For now, return empty - could add server-level messages later
            return Vec::new();
        }
    }
    
    Vec::new()
}

/// Determine if a message should be shown based on filter preferences  
fn should_show_message(message: &ChatMessage, system_visible: bool, joins_parts_visible: bool) -> bool {
    match message.message_type {
        MessageType::Normal | MessageType::Action | MessageType::Error => true,
        MessageType::Join | MessageType::Part | MessageType::Quit => joins_parts_visible,
        MessageType::System | MessageType::Nick | MessageType::Topic => system_visible,
    }
}

/// Format message content with IRC formatting and links
fn format_message_content(content: &str) -> String {
    let mut formatted = content.to_string();
    
    // Basic URL detection and linking
    let url_regex = regex::Regex::new(r"https?://[^\s]+").unwrap();
    formatted = url_regex.replace_all(&formatted, |caps: &regex::Captures| {
        format!("<a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\" class=\"text-[var(--accent-primary)] hover:underline\">{}</a>", &caps[0], &caps[0])
    }).to_string();
    
    // Basic IRC formatting (simplified)
    // Bold: **text** or \x02text\x02
    formatted = formatted.replace("**", "<strong class=\"irc-bold\">").replace("**", "</strong>");
    
    // Italic: *text* or \x1dtext\x1d  
    formatted = formatted.replace("*", "<em class=\"irc-italic\">").replace("*", "</em>");
    
    // TODO: Add more IRC formatting support (colors, underline, etc.)
    
    formatted
}