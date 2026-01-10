//! Dialog system for RustIRC GUI
//!
//! Implements modal dialogs for server connections, preferences,
//! channel joining, and other user interactions.

use crate::state::AppState;
use crate::theme::Theme;
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, slider, text, text_input,
    Space,
};
use iced::{Element, Length, Size, Task};
use rustirc_core::connection::ConnectionConfig;

/// Dialog message types
#[derive(Debug, Clone)]
pub enum DialogMessage {
    // Connection dialog
    ConnectionServerChanged(String),
    ConnectionPortChanged(String),
    ConnectionNickChanged(String),
    ConnectionUsernameChanged(String),
    ConnectionRealnameChanged(String),
    ConnectionPasswordChanged(String),
    ConnectionUseTlsToggled(bool),
    ConnectionAutoConnectToggled(bool),
    ConnectionConnect,
    ConnectionCancel,

    // Join channel dialog
    JoinChannelChanged(String),
    JoinChannelKeyChanged(String),
    JoinChannel,
    JoinCancel,

    // Preferences dialog
    PreferencesThemeChanged(String),
    PreferencesFontSizeChanged(String),
    PreferencesNotificationSoundToggled(bool),
    PreferencesNotificationPopupToggled(bool),
    PreferencesShowTimestampsToggled(bool),
    PreferencesNickColorsToggled(bool),
    PreferencesCompactModeToggled(bool),
    PreferencesApply,
    PreferencesCancel,

    // About dialog
    AboutOk,

    // Find dialog
    FindQueryChanged(String),
    FindNext,
    FindPrevious,
    FindCaseSensitiveToggled(bool),
    FindRegexToggled(bool),
    FindClose,

    // Network list dialog
    NetworkListAdd,
    NetworkListEdit(usize),
    NetworkListDelete(usize),
    NetworkListConnect(usize),
    NetworkListClose,
}

/// Available dialog types
#[derive(Debug, Clone, PartialEq)]
pub enum DialogType {
    None,
    Connection,
    JoinChannel,
    Preferences,
    About,
    Find,
    NetworkList,
}

/// Dialog manager
pub struct DialogManager {
    current_dialog: DialogType,
    connection_dialog: ConnectionDialog,
    join_channel_dialog: JoinChannelDialog,
    preferences_dialog: PreferencesDialog,
    about_dialog: AboutDialog,
    find_dialog: FindDialog,
    network_list_dialog: NetworkListDialog,
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            current_dialog: DialogType::None,
            connection_dialog: ConnectionDialog::new(),
            join_channel_dialog: JoinChannelDialog::new(),
            preferences_dialog: PreferencesDialog::new(),
            about_dialog: AboutDialog::new(),
            find_dialog: FindDialog::new(),
            network_list_dialog: NetworkListDialog::new(),
        }
    }

    pub fn show_dialog(&mut self, dialog_type: DialogType) {
        self.current_dialog = dialog_type;
    }

    pub fn hide_dialog(&mut self) {
        self.current_dialog = DialogType::None;
    }

    pub fn is_showing(&self) -> bool {
        self.current_dialog != DialogType::None
    }

    pub fn current_dialog(&self) -> &DialogType {
        &self.current_dialog
    }

    pub fn update(
        &mut self,
        message: DialogMessage,
        app_state: &mut AppState,
    ) -> Task<DialogMessage> {
        match message {
            // Connection dialog messages
            DialogMessage::ConnectionServerChanged(server) => {
                self.connection_dialog.server = server;
                Task::none()
            }
            DialogMessage::ConnectionPortChanged(port) => {
                if let Ok(port_num) = port.parse::<u16>() {
                    self.connection_dialog.port = port_num;
                }
                Task::none()
            }
            DialogMessage::ConnectionNickChanged(nick) => {
                self.connection_dialog.nickname = nick;
                Task::none()
            }
            DialogMessage::ConnectionUsernameChanged(username) => {
                self.connection_dialog.username = username;
                Task::none()
            }
            DialogMessage::ConnectionRealnameChanged(realname) => {
                self.connection_dialog.realname = realname;
                Task::none()
            }
            DialogMessage::ConnectionPasswordChanged(password) => {
                self.connection_dialog.password = password;
                Task::none()
            }
            DialogMessage::ConnectionUseTlsToggled(use_tls) => {
                self.connection_dialog.use_tls = use_tls;
                Task::none()
            }
            DialogMessage::ConnectionAutoConnectToggled(auto_connect) => {
                self.connection_dialog.auto_connect = auto_connect;
                Task::none()
            }
            DialogMessage::ConnectionConnect => {
                // Create connection config and trigger connection
                let config = self.connection_dialog.to_connection_config();
                self.hide_dialog();

                // Use the config to add server to app state
                let server_id = format!("{}:{}", config.server, config.port);
                app_state.add_server(server_id, config.server.clone());

                Task::none()
            }
            DialogMessage::ConnectionCancel => {
                self.hide_dialog();
                Task::none()
            }

            // Join channel dialog messages
            DialogMessage::JoinChannelChanged(channel) => {
                self.join_channel_dialog.channel = channel;
                Task::none()
            }
            DialogMessage::JoinChannelKeyChanged(key) => {
                self.join_channel_dialog.key = key;
                Task::none()
            }
            DialogMessage::JoinChannel => {
                let channel = self.join_channel_dialog.channel.clone();
                self.hide_dialog();

                // Use channel to add to app state
                if !channel.is_empty() {
                    // Get current server or use default
                    let server_id = "default".to_string(); // Would get from current connection
                    app_state.add_channel_tab(server_id, channel);
                }

                Task::none()
            }
            DialogMessage::JoinCancel => {
                self.hide_dialog();
                Task::none()
            }

            // Preferences dialog messages
            DialogMessage::PreferencesThemeChanged(theme) => {
                self.preferences_dialog.theme = theme;
                Task::none()
            }
            DialogMessage::PreferencesFontSizeChanged(size) => {
                if let Ok(font_size) = size.parse::<f32>() {
                    self.preferences_dialog.font_size = font_size;
                }
                Task::none()
            }
            DialogMessage::PreferencesNotificationSoundToggled(enabled) => {
                self.preferences_dialog.notification_sound = enabled;
                Task::none()
            }
            DialogMessage::PreferencesNotificationPopupToggled(enabled) => {
                self.preferences_dialog.notification_popup = enabled;
                Task::none()
            }
            DialogMessage::PreferencesShowTimestampsToggled(enabled) => {
                self.preferences_dialog.show_timestamps = enabled;
                Task::none()
            }
            DialogMessage::PreferencesNickColorsToggled(enabled) => {
                self.preferences_dialog.nick_colors = enabled;
                Task::none()
            }
            DialogMessage::PreferencesCompactModeToggled(enabled) => {
                self.preferences_dialog.compact_mode = enabled;
                Task::none()
            }
            DialogMessage::PreferencesApply => {
                self.preferences_dialog.apply_to_app_state(app_state);
                self.hide_dialog();
                Task::none()
            }
            DialogMessage::PreferencesCancel => {
                self.hide_dialog();
                Task::none()
            }

            // About dialog messages
            DialogMessage::AboutOk => {
                self.hide_dialog();
                Task::none()
            }

            // Find dialog messages
            DialogMessage::FindQueryChanged(query) => {
                self.find_dialog.query = query;
                Task::none()
            }
            DialogMessage::FindNext => {
                // Perform search
                Task::none()
            }
            DialogMessage::FindPrevious => {
                // Perform reverse search
                Task::none()
            }
            DialogMessage::FindCaseSensitiveToggled(case_sensitive) => {
                self.find_dialog.case_sensitive = case_sensitive;
                Task::none()
            }
            DialogMessage::FindRegexToggled(regex) => {
                self.find_dialog.regex = regex;
                Task::none()
            }
            DialogMessage::FindClose => {
                self.hide_dialog();
                Task::none()
            }

            // Network list dialog messages
            DialogMessage::NetworkListAdd => {
                // Add new network - open connection dialog for new network creation
                if let DialogType::NetworkList = self.current_dialog {
                    // Reset connection dialog for new network entry
                    self.connection_dialog = ConnectionDialog::new();
                    self.current_dialog = DialogType::Connection;
                }
                Task::none()
            }
            DialogMessage::NetworkListEdit(index) => {
                // Edit network at index - open edit form for selected network
                if let DialogType::NetworkList = self.current_dialog {
                    if index < self.network_list_dialog.networks.len() {
                        // Set the connection dialog with the network data for editing
                        let network = &self.network_list_dialog.networks[index];
                        if let Some(server) = network.servers.first() {
                            let parts: Vec<&str> = server.split(':').collect();
                            self.connection_dialog.server = parts[0].to_string();
                            self.connection_dialog.port =
                                parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(6667);
                        }
                        self.connection_dialog.auto_connect = network.auto_connect;
                        self.current_dialog = DialogType::Connection;
                    }
                }
                Task::none()
            }
            DialogMessage::NetworkListDelete(index) => {
                // Delete network at index - remove from network list
                if let DialogType::NetworkList = self.current_dialog {
                    if index < self.network_list_dialog.networks.len() {
                        self.network_list_dialog.networks.remove(index);
                    }
                }
                Task::none()
            }
            DialogMessage::NetworkListConnect(index) => {
                // Connect to network at index - initiate connection
                if let DialogType::NetworkList = self.current_dialog {
                    if index < self.network_list_dialog.networks.len() {
                        let network = &self.network_list_dialog.networks[index];
                        if let Some(server) = network.servers.first() {
                            // Create connection config from network entry
                            let parts: Vec<&str> = server.split(':').collect();
                            let server_addr = parts[0].to_string();
                            let port = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(6667);

                            // Return a task to connect to this network
                            self.hide_dialog();
                            return Task::perform(
                                async move {
                                    crate::app::Message::ConnectToServer(
                                        format!("{server_addr}:{port}"),
                                        port,
                                    )
                                },
                                |_| DialogMessage::NetworkListClose, // Convert to DialogMessage
                            );
                        }
                    }
                }
                self.hide_dialog();
                Task::none()
            }
            DialogMessage::NetworkListClose => {
                self.hide_dialog();
                Task::none()
            }
        }
    }

    pub fn build(&self, app_state: &AppState) -> Option<Element<'_, DialogMessage>> {
        match self.current_dialog {
            DialogType::None => None,
            DialogType::Connection => Some(self.connection_dialog.build()),
            DialogType::JoinChannel => Some(self.join_channel_dialog.build()),
            DialogType::Preferences => Some(self.preferences_dialog.view_with_state(app_state)),
            DialogType::About => Some(self.about_dialog.build()),
            DialogType::Find => Some(self.find_dialog.build()),
            DialogType::NetworkList => Some(self.network_list_dialog.build()),
        }
    }
}

/// Server connection dialog
#[derive(Debug, Clone)]
pub struct ConnectionDialog {
    pub server: String,
    pub port: u16,
    pub nickname: String,
    pub username: String,
    pub realname: String,
    pub password: String,
    pub use_tls: bool,
    pub auto_connect: bool,
}

impl Default for ConnectionDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionDialog {
    pub fn new() -> Self {
        Self {
            server: String::new(),
            port: 6667,
            nickname: "RustIRC".to_string(),
            username: "rustirc".to_string(),
            realname: "RustIRC User".to_string(),
            password: String::new(),
            use_tls: false,
            auto_connect: false,
        }
    }

    pub fn to_connection_config(&self) -> ConnectionConfig {
        ConnectionConfig {
            server: self.server.clone(),
            port: self.port,
            nickname: self.nickname.clone(),
            username: self.username.clone(),
            realname: self.realname.clone(),
            password: if self.password.is_empty() {
                None
            } else {
                Some(self.password.clone())
            },
            use_tls: self.use_tls,
            ..Default::default()
        }
    }

    pub fn build(&self) -> Element<'_, DialogMessage> {
        // Use Size for proper dialog dimensions
        let min_size = Size::new(400.0, 300.0);
        let max_size = Size::new(600.0, 500.0);

        let content = column![
            text("Connect to IRC Server").size(20),
            Space::new().height(10),
            row![
                text("Server:").width(80),
                text_input("irc.libera.chat", &self.server)
                    .on_input(DialogMessage::ConnectionServerChanged)
                    .width(200),
            ]
            .spacing(10),
            row![
                text("Port:").width(80),
                text_input("6667", &self.port.to_string())
                    .on_input(DialogMessage::ConnectionPortChanged)
                    .width(100),
            ]
            .spacing(10),
            row![
                text("Nickname:").width(80),
                text_input("nickname", &self.nickname)
                    .on_input(DialogMessage::ConnectionNickChanged)
                    .width(200),
            ]
            .spacing(10),
            row![
                text("Username:").width(80),
                text_input("username", &self.username)
                    .on_input(DialogMessage::ConnectionUsernameChanged)
                    .width(200),
            ]
            .spacing(10),
            row![
                text("Real name:").width(80),
                text_input("Real Name", &self.realname)
                    .on_input(DialogMessage::ConnectionRealnameChanged)
                    .width(200),
            ]
            .spacing(10),
            row![
                text("Password:").width(80),
                text_input("", &self.password)
                    .on_input(DialogMessage::ConnectionPasswordChanged)
                    .secure(true)
                    .width(200),
            ]
            .spacing(10),
            checkbox(self.use_tls)
                .label("Use TLS/SSL")
                .on_toggle(DialogMessage::ConnectionUseTlsToggled),
            checkbox(self.auto_connect)
                .label("Auto-connect on startup")
                .on_toggle(DialogMessage::ConnectionAutoConnectToggled),
            Space::new().height(20),
            row![
                button(text("Connect")).on_press(DialogMessage::ConnectionConnect),
                Space::new().width(10),
                button(text("Cancel")).on_press(DialogMessage::ConnectionCancel),
            ],
        ]
        .spacing(10)
        .padding(20)
        .max_width(max_size.width); // Use Size for maximum constraints

        // Apply theme styling to the container
        let theme_style = Theme::from_type(crate::theme::ThemeType::default());

        container(content)
            .width(Length::Fixed(min_size.width))  // Use min_size for container width
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(theme_style.palette.background.into()),
                border: iced::Border {
                    color: theme_style.palette.text_primary,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}

/// Join channel dialog
#[derive(Debug, Clone)]
pub struct JoinChannelDialog {
    pub channel: String,
    pub key: String,
}

impl Default for JoinChannelDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl JoinChannelDialog {
    pub fn new() -> Self {
        Self {
            channel: String::new(),
            key: String::new(),
        }
    }

    pub fn build(&self) -> Element<'_, DialogMessage> {
        let content = column![
            text("Join Channel").size(20),
            Space::new().height(10),
            row![
                text("Channel:").width(80),
                text_input("#channel", &self.channel)
                    .on_input(DialogMessage::JoinChannelChanged)
                    .width(200),
            ]
            .spacing(10),
            row![
                text("Key:").width(80),
                text_input("", &self.key)
                    .on_input(DialogMessage::JoinChannelKeyChanged)
                    .width(200),
            ]
            .spacing(10),
            Space::new().height(20),
            row![
                button(text("Join")).on_press(DialogMessage::JoinChannel),
                Space::new().width(10),
                button(text("Cancel")).on_press(DialogMessage::JoinCancel),
            ],
        ]
        .spacing(10)
        .padding(20)
        .max_width(350);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

/// Preferences dialog
#[derive(Debug, Clone)]
pub struct PreferencesDialog {
    pub theme: String,
    pub font_size: f32,
    pub notification_sound: bool,
    pub notification_popup: bool,
    pub show_timestamps: bool,
    pub nick_colors: bool,
    pub compact_mode: bool,
}

impl Default for PreferencesDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl PreferencesDialog {
    pub fn new() -> Self {
        Self {
            theme: "Dark".to_string(),
            font_size: 13.0,
            notification_sound: true,
            notification_popup: true,
            show_timestamps: true,
            nick_colors: true,
            compact_mode: false,
        }
    }

    pub fn from_app_state(app_state: &AppState) -> Self {
        let settings = app_state.settings();
        Self {
            theme: "Dark".to_string(), // Would get from theme system
            font_size: settings.font_size,
            notification_sound: settings.notification_sound,
            notification_popup: settings.notification_popup,
            show_timestamps: settings.show_timestamps,
            nick_colors: settings.nick_colors,
            compact_mode: settings.compact_mode,
        }
    }

    pub fn apply_to_app_state(&self, app_state: &mut AppState) {
        let settings = app_state.settings_mut();
        settings.font_size = self.font_size;
        settings.notification_sound = self.notification_sound;
        settings.notification_popup = self.notification_popup;
        settings.show_timestamps = self.show_timestamps;
        settings.nick_colors = self.nick_colors;
        settings.compact_mode = self.compact_mode;
        settings.theme = self.theme.clone();
    }

    pub fn build(&self) -> Element<'_, DialogMessage> {
        let theme_options = vec![
            "Dark".to_string(),
            "Light".to_string(),
            "Material Design 3".to_string(),
            "Dracula".to_string(),
            "Nord".to_string(),
            "Solarized Light".to_string(),
            "Solarized Dark".to_string(),
            "Gruvbox Light".to_string(),
            "Gruvbox Dark".to_string(),
            "Catppuccin Latte".to_string(),
            "Catppuccin Frappe".to_string(),
            "Catppuccin Macchiato".to_string(),
            "Catppuccin Mocha".to_string(),
            "Tokyo Night".to_string(),
            "Tokyo Night Storm".to_string(),
            "Tokyo Night Light".to_string(),
            "Kanagawa Wave".to_string(),
            "Kanagawa Dragon".to_string(),
            "Kanagawa Lotus".to_string(),
            "Moonfly".to_string(),
            "Nightfly".to_string(),
            "Oxocarbon".to_string(),
        ];

        let content = column![
            text("Preferences").size(20),
            Space::new().height(10),
            text("Appearance").size(16),
            row![
                text("Theme:").width(120),
                pick_list(
                    theme_options,
                    Some(self.theme.clone()),
                    DialogMessage::PreferencesThemeChanged
                )
                .width(150),
            ]
            .spacing(10),
            row![
                text("Font size:").width(120),
                text_input("13", &self.font_size.to_string())
                    .on_input(DialogMessage::PreferencesFontSizeChanged)
                    .width(100),
            ]
            .spacing(10),
            Space::new().height(10),
            text("Notifications").size(16),
            checkbox(self.notification_sound)
                .label("Sound notifications")
                .on_toggle(DialogMessage::PreferencesNotificationSoundToggled),
            checkbox(self.notification_popup)
                .label("Popup notifications")
                .on_toggle(DialogMessage::PreferencesNotificationPopupToggled),
            Space::new().height(10),
            text("Display").size(16),
            checkbox(self.show_timestamps)
                .label("Show timestamps")
                .on_toggle(DialogMessage::PreferencesShowTimestampsToggled),
            checkbox(self.nick_colors)
                .label("Colored nicknames")
                .on_toggle(DialogMessage::PreferencesNickColorsToggled),
            checkbox(self.compact_mode)
                .label("Compact mode")
                .on_toggle(DialogMessage::PreferencesCompactModeToggled),
            Space::new().height(20),
            row![
                button(text("Apply")).on_press(DialogMessage::PreferencesApply),
                Space::new().width(10),
                button(text("Cancel")).on_press(DialogMessage::PreferencesCancel),
            ],
        ]
        .spacing(10)
        .padding(20)
        .max_width(400);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    pub fn view_with_state(&self, app_state: &AppState) -> Element<'_, DialogMessage> {
        // Create a preferences view that reflects current app state
        let settings = app_state.settings();

        // Build the view directly with current app settings instead of relying on dialog state
        let theme_options = vec![
            "Dark".to_string(),
            "Light".to_string(),
            "Material Design 3".to_string(),
            "Dracula".to_string(),
            "Nord".to_string(),
            "Solarized Light".to_string(),
            "Solarized Dark".to_string(),
            "Gruvbox Light".to_string(),
            "Gruvbox Dark".to_string(),
            "Catppuccin Latte".to_string(),
            "Catppuccin Frappe".to_string(),
            "Catppuccin Macchiato".to_string(),
            "Catppuccin Mocha".to_string(),
            "Tokyo Night".to_string(),
            "Tokyo Night Storm".to_string(),
            "Tokyo Night Light".to_string(),
            "Kanagawa Wave".to_string(),
            "Kanagawa Dragon".to_string(),
            "Kanagawa Lotus".to_string(),
            "Moonfly".to_string(),
            "Nightfly".to_string(),
            "Oxocarbon".to_string(),
        ];

        let theme_picker = pick_list(
            theme_options,
            Some(settings.theme.clone()),
            DialogMessage::PreferencesThemeChanged,
        );

        let font_size_slider = slider(8.0..=24.0, settings.font_size, |size| {
            DialogMessage::PreferencesFontSizeChanged(size.to_string())
        });

        let notification_checkbox = checkbox(settings.notification_sound)
            .label("Enable notifications")
            .on_toggle(DialogMessage::PreferencesNotificationSoundToggled);

        let compact_checkbox = checkbox(settings.compact_mode)
            .label("Compact mode")
            .on_toggle(DialogMessage::PreferencesCompactModeToggled);

        let timestamps_checkbox = checkbox(settings.show_timestamps)
            .label("Show timestamps")
            .on_toggle(DialogMessage::PreferencesShowTimestampsToggled);

        let nick_colors_checkbox = checkbox(settings.nick_colors)
            .label("Nick colors")
            .on_toggle(DialogMessage::PreferencesNickColorsToggled);

        let content = column![
            text("Preferences").size(20),
            Space::new().height(10),
            text("Theme:"),
            theme_picker,
            Space::new().height(10),
            text(format!("Font Size: {:.0}", settings.font_size)),
            font_size_slider,
            Space::new().height(10),
            notification_checkbox,
            compact_checkbox,
            timestamps_checkbox,
            nick_colors_checkbox,
            Space::new().height(20),
            row![
                button("Apply").on_press(DialogMessage::PreferencesApply),
                Space::new().width(10),
                button("Cancel").on_press(DialogMessage::PreferencesCancel),
            ]
            .spacing(10)
        ]
        .spacing(10)
        .padding(20)
        .max_width(400);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

/// About dialog
#[derive(Debug, Clone)]
pub struct AboutDialog;

impl Default for AboutDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl AboutDialog {
    pub fn new() -> Self {
        Self
    }

    pub fn build(&self) -> Element<'_, DialogMessage> {
        let content = column![
            text("RustIRC").size(24),
            text("Modern IRC Client").size(16),
            Space::new().height(10),
            text("Version 1.0.0"),
            text("Built with Rust and Iced"),
            Space::new().height(10),
            text("A modern IRC client combining the best features"),
            text("of mIRC, HexChat, and WeeChat."),
            Space::new().height(20),
            text("© 2025 RustIRC Project"),
            Space::new().height(20),
            button(text("OK")).on_press(DialogMessage::AboutOk),
        ]
        .spacing(5)
        .padding(20)
        .max_width(350);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

/// Find dialog
#[derive(Debug, Clone)]
pub struct FindDialog {
    pub query: String,
    pub case_sensitive: bool,
    pub regex: bool,
}

impl Default for FindDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl FindDialog {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            case_sensitive: false,
            regex: false,
        }
    }

    pub fn build(&self) -> Element<'_, DialogMessage> {
        let content = column![
            text("Find").size(20),
            Space::new().height(10),
            row![
                text("Find:").width(60),
                text_input("Search text", &self.query)
                    .on_input(DialogMessage::FindQueryChanged)
                    .width(250),
            ]
            .spacing(10),
            checkbox(self.case_sensitive)
                .label("Case sensitive")
                .on_toggle(DialogMessage::FindCaseSensitiveToggled),
            checkbox(self.regex)
                .label("Regular expression")
                .on_toggle(DialogMessage::FindRegexToggled),
            Space::new().height(20),
            row![
                button(text("Find Next")).on_press(DialogMessage::FindNext),
                Space::new().width(10),
                button(text("Find Previous")).on_press(DialogMessage::FindPrevious),
                Space::new().width(10),
                button(text("Close")).on_press(DialogMessage::FindClose),
            ],
        ]
        .spacing(10)
        .padding(20)
        .max_width(400);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

/// Network list dialog
#[derive(Debug, Clone)]
pub struct NetworkListDialog {
    pub networks: Vec<NetworkEntry>,
}

#[derive(Debug, Clone)]
pub struct NetworkEntry {
    pub name: String,
    pub servers: Vec<String>,
    pub auto_connect: bool,
}

impl Default for NetworkListDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkListDialog {
    pub fn new() -> Self {
        Self {
            networks: vec![
                NetworkEntry {
                    name: "Libera.Chat".to_string(),
                    servers: vec!["irc.libera.chat:6667".to_string()],
                    auto_connect: false,
                },
                NetworkEntry {
                    name: "OFTC".to_string(),
                    servers: vec!["irc.oftc.net:6667".to_string()],
                    auto_connect: false,
                },
            ],
        }
    }

    pub fn build(&self) -> Element<'_, DialogMessage> {
        let network_list = scrollable(column(
            self.networks
                .iter()
                .enumerate()
                .map(|(i, network)| {
                    row![
                        text(&network.name).width(150),
                        text(network.servers.join(", ")).width(200),
                        button(text("Connect")).on_press(DialogMessage::NetworkListConnect(i)),
                        button(text("Edit")).on_press(DialogMessage::NetworkListEdit(i)),
                        button(text("Delete")).on_press(DialogMessage::NetworkListDelete(i)),
                    ]
                    .spacing(10)
                    .into()
                })
                .collect::<Vec<_>>(),
        ));

        let content = column![
            text("Network List").size(20),
            Space::new().height(10),
            row![
                text("Network").width(150),
                text("Servers").width(200),
                text("Actions"),
            ],
            network_list.height(300),
            Space::new().height(10),
            row![
                button(text("Add Network")).on_press(DialogMessage::NetworkListAdd),
                Space::new().width(Length::Fill),
                button(text("Close")).on_press(DialogMessage::NetworkListClose),
            ],
        ]
        .spacing(10)
        .padding(20)
        .max_width(600);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

impl Default for DialogManager {
    fn default() -> Self {
        Self::new()
    }
}
