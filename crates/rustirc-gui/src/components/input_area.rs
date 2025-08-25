//! Input area component for sending messages and IRC commands

use crate::context::{IrcState, MessageType};
use dioxus::prelude::*;

/// Input area component for typing and sending messages
#[component]
pub fn InputArea() -> Element {
    let mut input_text = use_signal(|| String::new());
    let mut command_history = use_signal(|| Vec::<String>::new());
    let mut history_index = use_signal(|| 0usize);
    let irc_state = use_context::<IrcState>();
    let active_tab = irc_state.active_tab.read();
    let current_server = irc_state.current_server.read();
    let current_channel = irc_state.current_channel.read();
    
    let is_connected = current_server.is_some();
    let placeholder = if *active_tab == "welcome" {
        "Welcome to RustIRC! Connect to a server to start chatting."
    } else if current_channel.is_some() {
        "Type a message..."
    } else {
        "Type a command or message..."
    };

    rsx! {
        div { 
            class: "p-3 border-t border-[var(--border-color)]",
            
            div { 
                class: "flex items-center space-x-3",
                
                // Main input field
                input {
                    class: "flex-1 irc-input",
                    r#type: "text",
                    placeholder: "{placeholder}",
                    disabled: !is_connected && *active_tab != "welcome",
                    value: "{input_text}",
                    
                    oninput: move |evt| {
                        input_text.set(evt.value());
                    },
                    
                    onkeydown: move |evt| {
                        match evt.key().as_str() {
                            "Enter" => {
                                if !input_text().trim().is_empty() {
                                    send_message(&input_text(), &irc_state, &current_server, &current_channel);
                                    
                                    // Add to command history
                                    command_history.write().push(input_text());
                                    if command_history().len() > 100 {
                                        command_history.write().remove(0);
                                    }
                                    
                                    input_text.set(String::new());
                                    history_index.set(command_history().len());
                                }
                            },
                            "ArrowUp" => {
                                if !command_history().is_empty() && history_index() > 0 {
                                    let new_index = history_index() - 1;
                                    history_index.set(new_index);
                                    input_text.set(command_history()[new_index].clone());
                                }
                                evt.prevent_default();
                            },
                            "ArrowDown" => {
                                if !command_history().is_empty() {
                                    if history_index() < command_history().len() - 1 {
                                        let new_index = history_index() + 1;
                                        history_index.set(new_index);
                                        input_text.set(command_history()[new_index].clone());
                                    } else {
                                        history_index.set(command_history().len());
                                        input_text.set(String::new());
                                    }
                                }
                                evt.prevent_default();
                            },
                            "Tab" => {
                                // TODO: Implement tab completion for nicknames and commands
                                evt.prevent_default();
                            },
                            _ => {}
                        }
                    },
                }
                
                // Send button
                button {
                    class: "irc-button px-4",
                    disabled: input_text().trim().is_empty() || (!is_connected && *active_tab != "welcome"),
                    onclick: move |_| {
                        if !input_text().trim().is_empty() {
                            send_message(&input_text(), &irc_state, &current_server, &current_channel);
                            input_text.set(String::new());
                        }
                    },
                    "Send"
                }
                
                // Formatting tools (future enhancement)
                div { 
                    class: "flex items-center space-x-1 text-sm text-[var(--text-muted)]",
                    
                    button {
                        class: "px-2 py-1 rounded hover:bg-[var(--bg-tertiary)] transition-colors",
                        title: "Bold (Ctrl+B)",
                        onclick: move |_| {
                            insert_formatting(&mut input_text, "**", "**");
                        },
                        "B"
                    }
                    
                    button {
                        class: "px-2 py-1 rounded hover:bg-[var(--bg-tertiary)] transition-colors",
                        title: "Italic (Ctrl+I)",
                        onclick: move |_| {
                            insert_formatting(&mut input_text, "*", "*");
                        },
                        "I"
                    }
                    
                    button {
                        class: "px-2 py-1 rounded hover:bg-[var(--bg-tertiary)] transition-colors",
                        title: "Upload file",
                        onclick: move |_| {
                            // TODO: Implement file upload
                        },
                        "📎"
                    }
                }
            }
            
            // Status/typing indicators could go here
            if is_connected {
                div { 
                    class: "text-xs text-[var(--text-muted)] mt-2 flex items-center justify-between",
                    
                    div { 
                        if let Some(server_id) = current_server.as_ref() {
                            if let Some(channel) = current_channel.as_ref() {
                                span { "Connected to {server_id} in {channel}" }
                            } else {
                                span { "Connected to {server_id}" }
                            }
                        }
                    }
                    
                    div { 
                        class: "text-right",
                        span { "{input_text().len()}/512" }
                    }
                }
            }
        }
    }
}

/// Send a message or execute a command
fn send_message(
    text: &str,
    irc_state: &IrcState, 
    current_server: &Option<String>,
    current_channel: &Option<String>
) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    
    // Handle IRC commands
    if text.starts_with('/') {
        execute_irc_command(text, irc_state, current_server, current_channel);
    } else {
        // Send regular message
        send_regular_message(text, irc_state, current_server, current_channel);
    }
}

/// Execute an IRC command (e.g., /join, /part, /quit)
fn execute_irc_command(
    command: &str,
    irc_state: &IrcState,
    current_server: &Option<String>, 
    current_channel: &Option<String>
) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    
    let cmd = parts[0].to_lowercase();
    
    match cmd.as_str() {
        "/join" | "/j" => {
            if parts.len() >= 2 {
                let channel = parts[1];
                if let Some(server_id) = current_server {
                    irc_state.join_channel(channel.to_string());
                    // TODO: Send JOIN command to IRC server
                }
            } else {
                add_system_message(irc_state, current_server, "Usage: /join <channel>", MessageType::Error);
            }
        },
        "/part" | "/leave" => {
            if let Some(channel) = current_channel {
                if let Some(server_id) = current_server {
                    // TODO: Send PART command to IRC server
                    add_system_message(irc_state, current_server, &format!("Left {}", channel), MessageType::System);
                }
            } else {
                add_system_message(irc_state, current_server, "Not in a channel", MessageType::Error);
            }
        },
        "/quit" => {
            let quit_message = if parts.len() > 1 { 
                parts[1..].join(" ")
            } else { 
                "RustIRC".to_string()
            };
            
            // TODO: Send QUIT command and disconnect
            add_system_message(irc_state, current_server, &format!("Quit: {}", quit_message), MessageType::System);
        },
        "/nick" => {
            if parts.len() >= 2 {
                let new_nick = parts[1];
                // TODO: Send NICK command to IRC server
                add_system_message(irc_state, current_server, &format!("Changing nickname to {}", new_nick), MessageType::System);
            } else {
                add_system_message(irc_state, current_server, "Usage: /nick <nickname>", MessageType::Error);
            }
        },
        "/msg" | "/query" => {
            if parts.len() >= 3 {
                let target = parts[1];
                let message = parts[2..].join(" ");
                // TODO: Send PRIVMSG to target
                add_system_message(irc_state, current_server, &format!("-> {}: {}", target, message), MessageType::System);
            } else {
                add_system_message(irc_state, current_server, "Usage: /msg <nick> <message>", MessageType::Error);
            }
        },
        "/me" => {
            if parts.len() >= 2 {
                let action = parts[1..].join(" ");
                send_action_message(&action, irc_state, current_server, current_channel);
            } else {
                add_system_message(irc_state, current_server, "Usage: /me <action>", MessageType::Error);
            }
        },
        "/help" => {
            let help_text = r#"Available commands:
/join <channel> - Join a channel
/part - Leave current channel  
/quit [message] - Quit IRC
/nick <nickname> - Change nickname
/msg <nick> <message> - Send private message
/me <action> - Send action message
/help - Show this help"#;
            add_system_message(irc_state, current_server, help_text, MessageType::System);
        },
        _ => {
            add_system_message(irc_state, current_server, &format!("Unknown command: {}", cmd), MessageType::Error);
        }
    }
}

/// Send a regular chat message
fn send_regular_message(
    text: &str,
    irc_state: &IrcState,
    current_server: &Option<String>,
    current_channel: &Option<String>
) {
    if let (Some(server_id), Some(channel)) = (current_server, current_channel) {
        // Add message to local state (will be echoed back from server in real implementation)
        irc_state.add_message(
            server_id.clone(),
            channel.clone(),
            Some("YourNick".to_string()), // TODO: Get actual nickname
            text.to_string(),
            MessageType::Normal
        );
        
        // TODO: Send PRIVMSG to IRC server
    } else if let Some(server_id) = current_server {
        add_system_message(irc_state, current_server, "Not in a channel. Use /join <channel> to join a channel.", MessageType::Error);
    }
}

/// Send an action message (/me command)
fn send_action_message(
    action: &str,
    irc_state: &IrcState,
    current_server: &Option<String>,
    current_channel: &Option<String>
) {
    if let (Some(server_id), Some(channel)) = (current_server, current_channel) {
        irc_state.add_message(
            server_id.clone(),
            channel.clone(),
            Some("YourNick".to_string()), // TODO: Get actual nickname
            action.to_string(),
            MessageType::Action
        );
        
        // TODO: Send CTCP ACTION to IRC server
    }
}

/// Add a system message to the current context
fn add_system_message(
    irc_state: &IrcState,
    current_server: &Option<String>,
    message: &str,
    msg_type: MessageType
) {
    if let Some(server_id) = current_server {
        // Add to server messages or current channel
        let target = if let Some(channel) = &irc_state.current_channel.read().as_ref() {
            channel.to_string()
        } else {
            server_id.clone() // Server-level message
        };
        
        irc_state.add_message(
            server_id.clone(),
            target,
            None, // System message has no sender
            message.to_string(),
            msg_type
        );
    }
}

/// Insert formatting around selected text or at cursor
fn insert_formatting(input_text: &mut Signal<String>, start_marker: &str, end_marker: &str) {
    let current = input_text();
    
    // For now, just append the markers - could enhance with selection support
    let new_text = format!("{}{}{}{}", current, start_marker, end_marker, "");
    input_text.set(new_text);
    
    // TODO: Handle text selection and cursor positioning
}