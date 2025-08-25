//! Dialog components for various user interactions

use crate::context::{IrcState, UiState, ThemeState, ThemeType, DialogType};
use dioxus::prelude::*;

/// Main dialog dispatcher component
#[component]
pub fn DialogProvider() -> Element {
    let ui_state = use_context::<UiState>();
    let current_dialog = ui_state.current_dialog.read();
    
    if let Some(dialog_type) = current_dialog.as_ref() {
        rsx! {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black bg-opacity-50",
                onclick: move |_| {
                    ui_state.hide_dialog();
                },
                
                div {
                    class: "bg-[var(--bg-primary)] rounded-lg shadow-xl max-w-md w-full max-h-[90vh] overflow-hidden",
                    onclick: move |e| {
                        e.stop_propagation();
                    },
                    
                    match dialog_type {
                        DialogType::Connect => rsx! { ConnectDialog {} },
                        DialogType::Settings => rsx! { SettingsDialog {} },
                        DialogType::About => rsx! { AboutDialog {} },
                        DialogType::UserInfo(username) => rsx! { UserInfoDialog { username: username.clone() } },
                        DialogType::ChannelList => rsx! { ChannelListDialog {} },
                        DialogType::JoinChannel => rsx! { JoinChannelDialog {} },
                        DialogType::Preferences => rsx! { PreferencesDialog {} },
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}

/// Connect to server dialog
#[component]
fn ConnectDialog() -> Element {
    let ui_state = use_context::<UiState>();
    let irc_state = use_context::<IrcState>();
    let mut server = use_signal(|| "irc.libera.chat".to_string());
    let mut port = use_signal(|| "6697".to_string());
    let mut nickname = use_signal(|| "RustUser".to_string());
    let mut username = use_signal(|| "rustuser".to_string());
    let mut realname = use_signal(|| "RustIRC User".to_string());
    let mut use_tls = use_signal(|| true);
    let mut auto_join = use_signal(|| "#rust".to_string());

    rsx! {
        div {
            class: "flex flex-col h-full",
            
            // Header
            div {
                class: "p-4 border-b border-[var(--border-color)]",
                h2 {
                    class: "text-lg font-semibold text-[var(--text-primary)]",
                    "Connect to IRC Server"
                }
            }
            
            // Content
            div {
                class: "p-4 space-y-4 overflow-y-auto flex-1",
                
                // Server settings
                div {
                    class: "space-y-2",
                    label {
                        class: "block text-sm font-medium text-[var(--text-secondary)]",
                        "Server"
                    }
                    input {
                        class: "irc-input w-full",
                        r#type: "text",
                        value: "{server}",
                        placeholder: "irc.libera.chat",
                        oninput: move |evt| server.set(evt.value()),
                    }
                }
                
                div {
                    class: "grid grid-cols-2 gap-2",
                    div {
                        label {
                            class: "block text-sm font-medium text-[var(--text-secondary)]",
                            "Port"
                        }
                        input {
                            class: "irc-input w-full",
                            r#type: "number",
                            value: "{port}",
                            oninput: move |evt| port.set(evt.value()),
                        }
                    }
                    div {
                        class: "flex items-center mt-6",
                        input {
                            id: "use_tls",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: use_tls(),
                            onchange: move |evt| use_tls.set(evt.checked()),
                        }
                        label {
                            r#for: "use_tls",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Use TLS/SSL"
                        }
                    }
                }
                
                // User settings
                div {
                    class: "space-y-2",
                    label {
                        class: "block text-sm font-medium text-[var(--text-secondary)]",
                        "Nickname"
                    }
                    input {
                        class: "irc-input w-full",
                        r#type: "text",
                        value: "{nickname}",
                        placeholder: "YourNick",
                        oninput: move |evt| nickname.set(evt.value()),
                    }
                }
                
                div {
                    class: "space-y-2",
                    label {
                        class: "block text-sm font-medium text-[var(--text-secondary)]",
                        "Username"
                    }
                    input {
                        class: "irc-input w-full",
                        r#type: "text",
                        value: "{username}",
                        placeholder: "username",
                        oninput: move |evt| username.set(evt.value()),
                    }
                }
                
                div {
                    class: "space-y-2",
                    label {
                        class: "block text-sm font-medium text-[var(--text-secondary)]",
                        "Real Name"
                    }
                    input {
                        class: "irc-input w-full",
                        r#type: "text",
                        value: "{realname}",
                        placeholder: "Your Real Name",
                        oninput: move |evt| realname.set(evt.value()),
                    }
                }
                
                div {
                    class: "space-y-2",
                    label {
                        class: "block text-sm font-medium text-[var(--text-secondary)]",
                        "Auto-join Channels"
                    }
                    input {
                        class: "irc-input w-full",
                        r#type: "text",
                        value: "{auto_join}",
                        placeholder: "#rust,#programming",
                        oninput: move |evt| auto_join.set(evt.value()),
                    }
                    p {
                        class: "text-xs text-[var(--text-muted)]",
                        "Separate multiple channels with commas"
                    }
                }
            }
            
            // Footer
            div {
                class: "p-4 border-t border-[var(--border-color)] flex justify-end space-x-2",
                
                button {
                    class: "px-4 py-2 rounded border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors",
                    onclick: move |_| {
                        ui_state.hide_dialog();
                    },
                    "Cancel"
                }
                
                button {
                    class: "irc-button px-4 py-2",
                    onclick: move |_| {
                        let port_num: u16 = port().parse().unwrap_or(6667);
                        irc_state.connect_server(
                            server(),
                            port_num,
                            nickname(),
                            username(),
                            realname(),
                            use_tls(),
                            auto_join().split(',').map(|s| s.trim().to_string()).collect()
                        );
                        ui_state.hide_dialog();
                    },
                    "Connect"
                }
            }
        }
    }
}

/// Settings dialog
#[component]
fn SettingsDialog() -> Element {
    let ui_state = use_context::<UiState>();
    let theme_state = use_context::<ThemeState>();
    let current_theme = theme_state.current_theme.read();
    
    rsx! {
        div {
            class: "flex flex-col h-full max-h-[600px]",
            
            // Header
            div {
                class: "p-4 border-b border-[var(--border-color)]",
                h2 {
                    class: "text-lg font-semibold text-[var(--text-primary)]",
                    "Settings"
                }
            }
            
            // Content
            div {
                class: "p-4 space-y-6 overflow-y-auto flex-1",
                
                // Appearance settings
                div {
                    class: "space-y-3",
                    h3 {
                        class: "font-medium text-[var(--text-primary)] border-b border-[var(--border-color)] pb-1",
                        "Appearance"
                    }
                    
                    div {
                        class: "space-y-2",
                        label {
                            class: "block text-sm font-medium text-[var(--text-secondary)]",
                            "Theme"
                        }
                        select {
                            class: "irc-input w-full",
                            value: "{current_theme:?}",
                            onchange: move |evt| {
                                if let Ok(theme) = evt.value().parse::<ThemeType>() {
                                    theme_state.set_theme(theme);
                                }
                            },
                            
                            option { value: "Dark", "Dark" }
                            option { value: "Light", "Light" }
                            option { value: "Discord", "Discord" }
                            option { value: "Nord", "Nord" }
                            option { value: "Dracula", "Dracula" }
                            option { value: "MaterialDesign", "Material Design" }
                            option { value: "Catppuccin", "Catppuccin" }
                        }
                    }
                }
                
                // Chat settings
                div {
                    class: "space-y-3",
                    h3 {
                        class: "font-medium text-[var(--text-primary)] border-b border-[var(--border-color)] pb-1",
                        "Chat"
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "show_timestamps",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: true,
                        }
                        label {
                            r#for: "show_timestamps",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Show timestamps"
                        }
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "compact_mode",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: false,
                        }
                        label {
                            r#for: "compact_mode",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Compact message view"
                        }
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "show_join_part",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: true,
                        }
                        label {
                            r#for: "show_join_part",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Show join/part messages"
                        }
                    }
                }
                
                // Notification settings
                div {
                    class: "space-y-3",
                    h3 {
                        class: "font-medium text-[var(--text-primary)] border-b border-[var(--border-color)] pb-1",
                        "Notifications"
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "desktop_notifications",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: true,
                        }
                        label {
                            r#for: "desktop_notifications",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Desktop notifications"
                        }
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "sound_notifications",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: false,
                        }
                        label {
                            r#for: "sound_notifications",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Sound notifications"
                        }
                    }
                }
            }
            
            // Footer
            div {
                class: "p-4 border-t border-[var(--border-color)] flex justify-end space-x-2",
                
                button {
                    class: "px-4 py-2 rounded border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors",
                    onclick: move |_| {
                        ui_state.hide_dialog();
                    },
                    "Cancel"
                }
                
                button {
                    class: "irc-button px-4 py-2",
                    onclick: move |_| {
                        // TODO: Save settings
                        ui_state.hide_dialog();
                    },
                    "Save"
                }
            }
        }
    }
}

/// About dialog
#[component]
fn AboutDialog() -> Element {
    let ui_state = use_context::<UiState>();
    
    rsx! {
        div {
            class: "flex flex-col h-full",
            
            // Header
            div {
                class: "p-4 border-b border-[var(--border-color)]",
                h2 {
                    class: "text-lg font-semibold text-[var(--text-primary)]",
                    "About RustIRC"
                }
            }
            
            // Content
            div {
                class: "p-6 space-y-4 text-center",
                
                div {
                    class: "space-y-2",
                    h3 {
                        class: "text-2xl font-bold text-[var(--accent-primary)]",
                        "RustIRC"
                    }
                    p {
                        class: "text-[var(--text-secondary)]",
                        "Version 0.3.7"
                    }
                    p {
                        class: "text-sm text-[var(--text-muted)]",
                        "Modern IRC client built with Rust and Dioxus"
                    }
                }
                
                div {
                    class: "space-y-2",
                    p {
                        class: "text-sm text-[var(--text-secondary)]",
                        "Combining the best features of mIRC, HexChat, and WeeChat"
                    }
                    p {
                        class: "text-sm text-[var(--text-secondary)]",
                        "Built with ❤️ using Rust, Dioxus, and modern web technologies"
                    }
                }
                
                div {
                    class: "pt-4 space-y-2 text-xs text-[var(--text-muted)]",
                    p { "© 2025 RustIRC Project" }
                    p { "Licensed under MIT License" }
                }
            }
            
            // Footer
            div {
                class: "p-4 border-t border-[var(--border-color)] flex justify-center",
                
                button {
                    class: "irc-button px-6 py-2",
                    onclick: move |_| {
                        ui_state.hide_dialog();
                    },
                    "Close"
                }
            }
        }
    }
}

/// User info dialog
#[component]
fn UserInfoDialog(username: String) -> Element {
    let ui_state = use_context::<UiState>();
    
    rsx! {
        div {
            class: "flex flex-col h-full",
            
            // Header
            div {
                class: "p-4 border-b border-[var(--border-color)]",
                h2 {
                    class: "text-lg font-semibold text-[var(--text-primary)]",
                    "User Information"
                }
            }
            
            // Content
            div {
                class: "p-4 space-y-4",
                
                div {
                    class: "text-center space-y-2",
                    div {
                        class: "w-16 h-16 bg-[var(--accent-primary)] rounded-full flex items-center justify-center mx-auto",
                        span {
                            class: "text-white text-xl font-bold",
                            "{username.chars().next().unwrap_or('?').to_uppercase()}"
                        }
                    }
                    h3 {
                        class: "text-xl font-semibold text-[var(--text-primary)]",
                        "{username}"
                    }
                }
                
                div {
                    class: "space-y-3",
                    div {
                        class: "flex justify-between",
                        span {
                            class: "text-[var(--text-secondary)]",
                            "Status:"
                        }
                        span {
                            class: "text-[var(--success)]",
                            "Online"
                        }
                    }
                    
                    div {
                        class: "flex justify-between",
                        span {
                            class: "text-[var(--text-secondary)]",
                            "Hostname:"
                        }
                        span {
                            class: "text-[var(--text-primary)] text-sm font-mono",
                            "user.example.com"
                        }
                    }
                    
                    div {
                        class: "flex justify-between",
                        span {
                            class: "text-[var(--text-secondary)]",
                            "Channels:"
                        }
                        span {
                            class: "text-[var(--text-primary)]",
                            "#rust, #programming"
                        }
                    }
                    
                    div {
                        class: "flex justify-between",
                        span {
                            class: "text-[var(--text-secondary)]",
                            "Idle time:"
                        }
                        span {
                            class: "text-[var(--text-primary)]",
                            "5 minutes"
                        }
                    }
                }
                
                div {
                    class: "pt-4 space-y-2",
                    button {
                        class: "w-full irc-button py-2",
                        "Send Private Message"
                    }
                    button {
                        class: "w-full px-4 py-2 rounded border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors",
                        "View WHOIS Info"
                    }
                }
            }
            
            // Footer
            div {
                class: "p-4 border-t border-[var(--border-color)] flex justify-end",
                
                button {
                    class: "px-4 py-2 rounded border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors",
                    onclick: move |_| {
                        ui_state.hide_dialog();
                    },
                    "Close"
                }
            }
        }
    }
}

/// Channel list dialog
#[component]
fn ChannelListDialog() -> Element {
    let ui_state = use_context::<UiState>();
    let mut search = use_signal(|| String::new());
    
    // Mock channel data
    let channels = vec![
        ("#rust", 1234, "The Rust programming language"),
        ("#programming", 567, "General programming discussion"),
        ("#linux", 890, "Linux operating system"),
        ("#javascript", 432, "JavaScript development"),
        ("#python", 321, "Python programming"),
    ];
    
    rsx! {
        div {
            class: "flex flex-col h-full max-h-[600px]",
            
            // Header
            div {
                class: "p-4 border-b border-[var(--border-color)]",
                h2 {
                    class: "text-lg font-semibold text-[var(--text-primary)]",
                    "Channel List"
                }
            }
            
            // Search
            div {
                class: "p-4 border-b border-[var(--border-color)]",
                input {
                    class: "irc-input w-full",
                    r#type: "text",
                    placeholder: "Search channels...",
                    value: "{search}",
                    oninput: move |evt| search.set(evt.value()),
                }
            }
            
            // Channel list
            div {
                class: "flex-1 overflow-y-auto",
                for (channel, users, topic) in channels {
                    div {
                        class: "p-3 border-b border-[var(--border-color)] hover:bg-[var(--bg-secondary)] cursor-pointer transition-colors",
                        onclick: move |_| {
                            // TODO: Join channel
                            ui_state.hide_dialog();
                        },
                        
                        div {
                            class: "flex items-center justify-between",
                            div {
                                class: "flex-1 min-w-0",
                                h4 {
                                    class: "font-medium text-[var(--text-primary)] truncate",
                                    "{channel}"
                                }
                                p {
                                    class: "text-sm text-[var(--text-secondary)] truncate",
                                    "{topic}"
                                }
                            }
                            div {
                                class: "text-xs text-[var(--text-muted)] ml-2",
                                "{users} users"
                            }
                        }
                    }
                }
            }
            
            // Footer
            div {
                class: "p-4 border-t border-[var(--border-color)] flex justify-end",
                
                button {
                    class: "px-4 py-2 rounded border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors",
                    onclick: move |_| {
                        ui_state.hide_dialog();
                    },
                    "Close"
                }
            }
        }
    }
}

/// Join channel dialog
#[component]
fn JoinChannelDialog() -> Element {
    let ui_state = use_context::<UiState>();
    let irc_state = use_context::<IrcState>();
    let mut channel_name = use_signal(|| String::new());
    let mut channel_key = use_signal(|| String::new());
    
    rsx! {
        div {
            class: "flex flex-col h-full",
            
            // Header
            div {
                class: "p-4 border-b border-[var(--border-color)]",
                h2 {
                    class: "text-lg font-semibold text-[var(--text-primary)]",
                    "Join Channel"
                }
            }
            
            // Content
            div {
                class: "p-4 space-y-4",
                
                div {
                    class: "space-y-2",
                    label {
                        class: "block text-sm font-medium text-[var(--text-secondary)]",
                        "Channel Name"
                    }
                    input {
                        class: "irc-input w-full",
                        r#type: "text",
                        placeholder: "#channel-name",
                        value: "{channel_name}",
                        oninput: move |evt| {
                            let mut value = evt.value();
                            if !value.starts_with('#') && !value.is_empty() {
                                value = format!("#{}", value);
                            }
                            channel_name.set(value);
                        },
                    }
                }
                
                div {
                    class: "space-y-2",
                    label {
                        class: "block text-sm font-medium text-[var(--text-secondary)]",
                        "Channel Key (optional)"
                    }
                    input {
                        class: "irc-input w-full",
                        r#type: "password",
                        placeholder: "Enter channel password if required",
                        value: "{channel_key}",
                        oninput: move |evt| channel_key.set(evt.value()),
                    }
                    p {
                        class: "text-xs text-[var(--text-muted)]",
                        "Only required for password-protected channels"
                    }
                }
            }
            
            // Footer
            div {
                class: "p-4 border-t border-[var(--border-color)] flex justify-end space-x-2",
                
                button {
                    class: "px-4 py-2 rounded border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors",
                    onclick: move |_| {
                        ui_state.hide_dialog();
                    },
                    "Cancel"
                }
                
                button {
                    class: "irc-button px-4 py-2",
                    disabled: channel_name().trim().is_empty(),
                    onclick: move |_| {
                        if !channel_name().trim().is_empty() {
                            irc_state.join_channel(channel_name());
                            ui_state.hide_dialog();
                        }
                    },
                    "Join Channel"
                }
            }
        }
    }
}

/// Preferences dialog (extended settings)
#[component]
fn PreferencesDialog() -> Element {
    let ui_state = use_context::<UiState>();
    
    rsx! {
        div {
            class: "flex flex-col h-full max-h-[700px]",
            
            // Header
            div {
                class: "p-4 border-b border-[var(--border-color)]",
                h2 {
                    class: "text-lg font-semibold text-[var(--text-primary)]",
                    "Preferences"
                }
            }
            
            // Content with tabs (simplified for now)
            div {
                class: "p-4 space-y-6 overflow-y-auto flex-1",
                
                // Interface preferences
                div {
                    class: "space-y-3",
                    h3 {
                        class: "font-medium text-[var(--text-primary)] border-b border-[var(--border-color)] pb-1",
                        "Interface"
                    }
                    
                    div {
                        class: "space-y-2",
                        label {
                            class: "block text-sm font-medium text-[var(--text-secondary)]",
                            "Font Size"
                        }
                        select {
                            class: "irc-input w-full",
                            option { value: "small", "Small" }
                            option { value: "medium", selected: true, "Medium" }
                            option { value: "large", "Large" }
                        }
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "show_user_list",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: true,
                        }
                        label {
                            r#for: "show_user_list",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Show user list by default"
                        }
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "show_topic",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: true,
                        }
                        label {
                            r#for: "show_topic",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Show channel topic"
                        }
                    }
                }
                
                // Advanced settings
                div {
                    class: "space-y-3",
                    h3 {
                        class: "font-medium text-[var(--text-primary)] border-b border-[var(--border-color)] pb-1",
                        "Advanced"
                    }
                    
                    div {
                        class: "space-y-2",
                        label {
                            class: "block text-sm font-medium text-[var(--text-secondary)]",
                            "Message Buffer Size"
                        }
                        input {
                            class: "irc-input w-full",
                            r#type: "number",
                            value: "1000",
                            min: "100",
                            max: "10000",
                        }
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "auto_reconnect",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: true,
                        }
                        label {
                            r#for: "auto_reconnect",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Auto-reconnect on disconnect"
                        }
                    }
                    
                    div {
                        class: "flex items-center",
                        input {
                            id: "logging_enabled",
                            r#type: "checkbox",
                            class: "mr-2",
                            checked: false,
                        }
                        label {
                            r#for: "logging_enabled",
                            class: "text-sm text-[var(--text-secondary)]",
                            "Enable chat logging"
                        }
                    }
                }
            }
            
            // Footer
            div {
                class: "p-4 border-t border-[var(--border-color)] flex justify-end space-x-2",
                
                button {
                    class: "px-4 py-2 rounded border border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors",
                    onclick: move |_| {
                        ui_state.hide_dialog();
                    },
                    "Cancel"
                }
                
                button {
                    class: "irc-button px-4 py-2",
                    onclick: move |_| {
                        // TODO: Save preferences
                        ui_state.hide_dialog();
                    },
                    "Apply"
                }
            }
        }
    }
}