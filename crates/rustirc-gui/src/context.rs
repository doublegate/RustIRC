//! Global state management using Dioxus Context API

use dioxus::prelude::*;
use rustirc_core::{ConnectionState, IrcClient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Connection information for a single IRC server
#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub id: String,
    pub server: String,
    pub port: u16,
    pub nickname: String,
    pub real_name: String,
    pub state: ConnectionState,
    pub channels: HashMap<String, ChannelInfo>,
    pub client: Option<IrcClient>,
}

/// Channel information
#[derive(Clone, Debug)]
pub struct ChannelInfo {
    pub name: String,
    pub topic: Option<String>,
    pub users: HashMap<String, UserInfo>,
    pub messages: Vec<ChatMessage>,
    pub unread_count: usize,
    pub joined: bool,
}

/// User information in a channel
#[derive(Clone, Debug)]
pub struct UserInfo {
    pub nickname: String,
    pub modes: HashSet<char>, // Channel modes (op, voice, etc.)
    pub away: bool,
    pub realname: Option<String>,
}

/// Chat message
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub sender: Option<String>,
    pub content: String,
    pub message_type: MessageType,
    pub formatted: bool,
}

/// Message types for different IRC events
#[derive(Clone, Debug, PartialEq)]
pub enum MessageType {
    Normal,
    Action,
    Join,
    Part,
    Quit,
    Nick,
    Topic,
    System,
    Error,
}

/// Theme types available in the application
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum ThemeType {
    Dark,
    Light,
    Discord,
    Slack,
    Terminal,
    Nord,
    Dracula,
    MaterialDesign,
    Catppuccin,
}

/// Dialog types that can be displayed
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum DialogType {
    Connect,
    Settings,
    About,
    ChannelList,
    UserInfo(String),
}

/// Global IRC state context
#[derive(Clone)]
pub struct IrcState {
    pub connections: Signal<HashMap<String, ConnectionInfo>>,
    pub current_server: Signal<Option<String>>,
    pub current_channel: Signal<Option<String>>,
    pub active_tab: Signal<String>,
}

impl Default for IrcState {
    fn default() -> Self {
        Self {
            connections: Signal::new(HashMap::new()),
            current_server: Signal::new(None),
            current_channel: Signal::new(None),
            active_tab: Signal::new("welcome".to_string()),
        }
    }
}

impl IrcState {
    /// Connect to a new IRC server
    pub fn connect_server(&self, server: String, port: u16, nickname: String) {
        let connection_id = Uuid::new_v4().to_string();
        let connection = ConnectionInfo {
            id: connection_id.clone(),
            server: server.clone(),
            port,
            nickname: nickname.clone(),
            real_name: nickname.clone(),
            state: ConnectionState::Connecting,
            channels: HashMap::new(),
            client: None,
        };

        self.connections
            .write()
            .insert(connection_id.clone(), connection);
        self.current_server.set(Some(connection_id));
    }

    /// Join a channel on the current server
    pub fn join_channel(&self, channel: String) {
        if let Some(server_id) = self.current_server.read().as_ref() {
            if let Some(connection) = self.connections.write().get_mut(server_id) {
                let channel_info = ChannelInfo {
                    name: channel.clone(),
                    topic: None,
                    users: HashMap::new(),
                    messages: Vec::new(),
                    unread_count: 0,
                    joined: false,
                };
                connection.channels.insert(channel.clone(), channel_info);
                self.current_channel.set(Some(channel.clone()));
                self.active_tab.set(format!("{}:{}", server_id, channel));
            }
        }
    }

    /// Add a message to a channel
    pub fn add_message(
        &self,
        server_id: String,
        target: String,
        sender: Option<String>,
        content: String,
        msg_type: MessageType,
    ) {
        if let Some(connection) = self.connections.write().get_mut(&server_id) {
            if let Some(channel) = connection.channels.get_mut(&target) {
                let message = ChatMessage {
                    id: Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    sender,
                    content,
                    message_type: msg_type,
                    formatted: false,
                };
                channel.messages.push(message);

                // Update unread count if not current channel
                let current_tab = self.active_tab.read();
                let expected_tab = format!("{}:{}", server_id, target);
                if *current_tab != expected_tab {
                    channel.unread_count += 1;
                }
            }
        }
    }
}

/// Theme state context
#[derive(Clone)]
pub struct ThemeState {
    pub current_theme: Signal<ThemeType>,
    pub custom_css: Signal<String>,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            current_theme: Signal::new(ThemeType::Dark),
            custom_css: Signal::new(String::new()),
        }
    }
}

impl ThemeState {
    pub fn set_theme(&self, theme: ThemeType) {
        self.current_theme.set(theme);
        // Update CSS custom properties based on theme
        self.update_css_variables(theme);
    }

    fn update_css_variables(&self, theme: ThemeType) {
        let css = match theme {
            ThemeType::Dark => include_str!("../assets/themes/dark.css"),
            ThemeType::Light => include_str!("../assets/themes/light.css"),
            ThemeType::Discord => include_str!("../assets/themes/discord.css"),
            _ => include_str!("../assets/themes/dark.css"), // Fallback
        };
        self.custom_css.set(css.to_string());
    }
}

/// UI state context for layout and interface preferences
#[derive(Clone)]
pub struct UiState {
    pub sidebar_width: Signal<f32>,
    pub user_list_visible: Signal<bool>,
    pub system_messages_visible: Signal<bool>,
    pub joins_parts_visible: Signal<bool>,
    pub active_dialogs: Signal<HashSet<DialogType>>,
    pub context_menu_position: Signal<Option<(f32, f32)>>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            sidebar_width: Signal::new(250.0),
            user_list_visible: Signal::new(true),
            system_messages_visible: Signal::new(true),
            joins_parts_visible: Signal::new(false),
            active_dialogs: Signal::new(HashSet::new()),
            context_menu_position: Signal::new(None),
        }
    }
}

impl UiState {
    pub fn show_dialog(&self, dialog: DialogType) {
        self.active_dialogs.write().insert(dialog);
    }

    pub fn hide_dialog(&self, dialog: DialogType) {
        self.active_dialogs.write().remove(&dialog);
    }

    pub fn is_dialog_open(&self, dialog: &DialogType) -> bool {
        self.active_dialogs.read().contains(dialog)
    }

    pub fn show_context_menu(&self, x: f32, y: f32) {
        self.context_menu_position.set(Some((x, y)));
    }

    pub fn hide_context_menu(&self) {
        self.context_menu_position.set(None);
    }
}

/// Context provider component that wraps the entire app
#[component]
pub fn ContextProvider(children: Element) -> Element {
    use_context_provider(|| IrcState::default());
    use_context_provider(|| ThemeState::default());
    use_context_provider(|| UiState::default());

    rsx! { {children} }
}
