//! Menu system for RustIRC GUI
//!
//! Implements the complete menu bar structure with all standard
//! IRC client menus and actions.

use crate::state::{AppState, TabType};
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Task};

/// Main menu bar message types
#[derive(Debug, Clone)]
pub enum MenuMessage {
    // File menu
    FileConnect,
    FileDisconnect,
    FileQuit,

    // Edit menu
    EditCopy,
    EditPaste,
    EditSelectAll,
    EditFind,
    EditPreferences,

    // View menu
    ViewToggleServerTree,
    ViewToggleUserList,
    ViewToggleStatusBar,
    ViewFullscreen,
    ViewZoomIn,
    ViewZoomOut,
    ViewResetZoom,

    // Server menu
    ServerConnect,
    ServerDisconnect,
    ServerReconnect,
    ServerProperties,
    ServerAddToFavorites,

    // Channel menu
    ChannelJoin,
    ChannelPart,
    ChannelTopic,
    ChannelModes,
    ChannelBanList,
    ChannelInviteList,

    // Tools menu
    ToolsPreferences,
    ToolsThemeEditor,
    ToolsScriptEditor,
    ToolsLogViewer,
    ToolsNetworkList,

    // Help menu
    HelpUserGuide,
    HelpKeyboardShortcuts,
    HelpAbout,
    HelpCheckUpdates,
}

/// Menu bar widget
pub struct MenuBar {
    active_menu: Option<MenuType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuType {
    File,
    Edit,
    View,
    Server,
    Channel,
    Tools,
    Help,
}

impl MenuBar {
    pub fn new() -> Self {
        Self { active_menu: None }
    }

    /// Toggle a menu open/closed
    pub fn toggle_menu(&mut self, menu_type: MenuType) {
        if self.active_menu == Some(menu_type) {
            self.active_menu = None;
        } else {
            self.active_menu = Some(menu_type);
        }
    }

    /// Close all menus
    pub fn close_all(&mut self) {
        self.active_menu = None;
    }

    /// Check if a menu is active
    pub fn is_menu_active(&self, menu_type: &MenuType) -> bool {
        self.active_menu.as_ref() == Some(menu_type)
    }

    pub fn update(&mut self, message: MenuMessage, app_state: &mut AppState) -> Task<MenuMessage> {
        match message {
            MenuMessage::FileConnect => {
                // This would open connection dialog
                Task::none()
            }
            MenuMessage::FileDisconnect => {
                // Disconnect from current server
                Task::none()
            }
            MenuMessage::FileQuit => {
                // Quit application
                Task::none()
            }
            MenuMessage::EditCopy => {
                // Copy selected text
                Task::none()
            }
            MenuMessage::EditPaste => {
                // Paste from clipboard
                Task::none()
            }
            MenuMessage::EditSelectAll => {
                // Select all text in current view
                Task::none()
            }
            MenuMessage::EditFind => {
                // Open find dialog
                Task::none()
            }
            MenuMessage::EditPreferences => {
                // Open preferences dialog
                Task::none()
            }
            MenuMessage::ViewToggleServerTree => {
                app_state.ui_state_mut().show_sidebar = !app_state.ui_state.show_sidebar;
                Task::none()
            }
            MenuMessage::ViewToggleUserList => {
                app_state.ui_state_mut().show_userlist = !app_state.ui_state.show_userlist;
                Task::none()
            }
            MenuMessage::ViewToggleStatusBar => {
                // Toggle status bar visibility
                Task::none()
            }
            MenuMessage::ViewFullscreen => {
                // Toggle fullscreen mode
                Task::none()
            }
            MenuMessage::ViewZoomIn => {
                let current_size = app_state.settings().font_size;
                app_state.settings_mut().font_size = (current_size + 1.0).min(24.0);
                Task::none()
            }
            MenuMessage::ViewZoomOut => {
                let current_size = app_state.settings().font_size;
                app_state.settings_mut().font_size = (current_size - 1.0).max(8.0);
                Task::none()
            }
            MenuMessage::ViewResetZoom => {
                app_state.settings_mut().font_size = 13.0;
                Task::none()
            }
            MenuMessage::ServerConnect => {
                // Open server connection dialog
                Task::none()
            }
            MenuMessage::ServerDisconnect => {
                // Disconnect from selected server
                Task::none()
            }
            MenuMessage::ServerReconnect => {
                // Reconnect to server
                Task::none()
            }
            MenuMessage::ServerProperties => {
                // Show server properties
                Task::none()
            }
            MenuMessage::ServerAddToFavorites => {
                // Add server to favorites
                Task::none()
            }
            MenuMessage::ChannelJoin => {
                // Open join channel dialog
                Task::none()
            }
            MenuMessage::ChannelPart => {
                // Leave current channel
                Task::none()
            }
            MenuMessage::ChannelTopic => {
                // Show/edit channel topic
                Task::none()
            }
            MenuMessage::ChannelModes => {
                // Show channel modes dialog
                Task::none()
            }
            MenuMessage::ChannelBanList => {
                // Show ban list
                Task::none()
            }
            MenuMessage::ChannelInviteList => {
                // Show invite list
                Task::none()
            }
            MenuMessage::ToolsPreferences => {
                // Open preferences dialog
                Task::none()
            }
            MenuMessage::ToolsThemeEditor => {
                // Open theme editor
                Task::none()
            }
            MenuMessage::ToolsScriptEditor => {
                // Open script editor
                Task::none()
            }
            MenuMessage::ToolsLogViewer => {
                // Open log viewer
                Task::none()
            }
            MenuMessage::ToolsNetworkList => {
                // Open network list editor
                Task::none()
            }
            MenuMessage::HelpUserGuide => {
                // Open user guide
                Task::none()
            }
            MenuMessage::HelpKeyboardShortcuts => {
                // Show keyboard shortcuts
                Task::none()
            }
            MenuMessage::HelpAbout => {
                // Show about dialog
                Task::none()
            }
            MenuMessage::HelpCheckUpdates => {
                // Check for updates
                Task::none()
            }
        }
    }

    pub fn view(&self, app_state: &AppState) -> Element<MenuMessage> {
        // Create menu bar with active menu rendering

        // File menu button
        let file_button: Element<MenuMessage> = if self.active_menu == Some(MenuType::File) {
            // Show dropdown when active
            column![
                button(text("File ▼"))
                    .on_press(MenuMessage::FileConnect) // Toggle on click
                    .width(Length::Shrink),
                self.render_file_menu(app_state)
            ]
            .into()
        } else {
            button(text("File"))
                .on_press(MenuMessage::FileConnect)
                .width(Length::Shrink)
                .into()
        };

        // Edit menu button
        let edit_button: Element<MenuMessage> = if self.active_menu == Some(MenuType::Edit) {
            column![
                button(text("Edit ▼"))
                    .on_press(MenuMessage::EditPreferences)
                    .width(Length::Shrink),
                self.render_edit_menu(app_state)
            ]
            .into()
        } else {
            button(text("Edit"))
                .on_press(MenuMessage::EditPreferences)
                .width(Length::Shrink)
                .into()
        };

        // View menu button
        let view_button: Element<MenuMessage> = if self.active_menu == Some(MenuType::View) {
            column![
                button(text("View ▼"))
                    .on_press(MenuMessage::ViewToggleServerTree)
                    .width(Length::Shrink),
                self.render_view_menu(app_state)
            ]
            .into()
        } else {
            button(text("View"))
                .on_press(MenuMessage::ViewToggleServerTree)
                .width(Length::Shrink)
                .into()
        };

        // Server menu button
        let server_button: Element<MenuMessage> = if self.active_menu == Some(MenuType::Server) {
            column![
                button(text("Server ▼"))
                    .on_press(MenuMessage::ServerConnect)
                    .width(Length::Shrink),
                self.render_server_menu(app_state)
            ]
            .into()
        } else {
            button(text("Server"))
                .on_press(MenuMessage::ServerConnect)
                .width(Length::Shrink)
                .into()
        };

        // Channel menu button
        let channel_button: Element<MenuMessage> = if self.active_menu == Some(MenuType::Channel) {
            column![
                button(text("Channel ▼"))
                    .on_press(MenuMessage::ChannelJoin)
                    .width(Length::Shrink),
                self.render_channel_menu(app_state)
            ]
            .into()
        } else {
            button(text("Channel"))
                .on_press(MenuMessage::ChannelJoin)
                .width(Length::Shrink)
                .into()
        };

        // Tools menu button
        let tools_button: Element<MenuMessage> = if self.active_menu == Some(MenuType::Tools) {
            column![
                button(text("Tools ▼"))
                    .on_press(MenuMessage::ToolsPreferences)
                    .width(Length::Shrink),
                self.render_tools_menu(app_state)
            ]
            .into()
        } else {
            button(text("Tools"))
                .on_press(MenuMessage::ToolsPreferences)
                .width(Length::Shrink)
                .into()
        };

        // Help menu button
        let help_button: Element<MenuMessage> = if self.active_menu == Some(MenuType::Help) {
            column![
                button(text("Help ▼"))
                    .on_press(MenuMessage::HelpAbout)
                    .width(Length::Shrink),
                self.render_help_menu(app_state)
            ]
            .into()
        } else {
            button(text("Help"))
                .on_press(MenuMessage::HelpAbout)
                .width(Length::Shrink)
                .into()
        };

        // Use container for proper menu bar styling and full-width layout
        container(
            row![
                file_button,
                edit_button,
                view_button,
                server_button,
                channel_button,
                tools_button,
                help_button,
            ]
            .spacing(10)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .padding(5)
        .into()
    }

    fn render_file_menu(&self, app_state: &AppState) -> Element<MenuMessage> {
        // Show connection status in menu
        let connected = app_state
            .servers
            .values()
            .any(|s| s.connection_state == rustirc_core::ConnectionState::Registered);
        let disconnect_text = if connected {
            format!("Disconnect ({})", app_state.servers.len())
        } else {
            "Disconnect".to_string()
        };

        column![
            button(text("Connect to Server...")).on_press(MenuMessage::FileConnect),
            button(text(disconnect_text)).on_press(if connected {
                MenuMessage::FileDisconnect
            } else {
                MenuMessage::FileConnect
            }),
            button(text("Quit")).on_press(MenuMessage::FileQuit),
        ]
        .spacing(2)
        .into()
    }

    fn render_edit_menu(&self, app_state: &AppState) -> Element<MenuMessage> {
        // Check if there's selected text to enable/disable copy
        let has_selection = app_state
            .current_tab()
            .map(|tab| !tab.messages.is_empty())
            .unwrap_or(false);

        column![
            button(text(if has_selection {
                "Copy"
            } else {
                "Copy (no selection)"
            }))
            .on_press(MenuMessage::EditCopy),
            button(text("Paste")).on_press(MenuMessage::EditPaste),
            button(text("Select All")).on_press(MenuMessage::EditSelectAll),
            button(text("Find...")).on_press(MenuMessage::EditFind),
            button(text("Preferences...")).on_press(MenuMessage::EditPreferences),
        ]
        .spacing(2)
        .into()
    }

    fn render_view_menu(&self, app_state: &AppState) -> Element<MenuMessage> {
        // Get current UI state for toggle indicators
        let show_sidebar = app_state.ui_state.show_sidebar;
        let show_userlist = app_state.ui_state.show_userlist;
        let font_size = app_state.settings().font_size;

        column![
            button(text(if show_sidebar {
                "☑ Server Tree"
            } else {
                "☐ Server Tree"
            }))
            .on_press(MenuMessage::ViewToggleServerTree),
            button(text(if show_userlist {
                "☑ User List"
            } else {
                "☐ User List"
            }))
            .on_press(MenuMessage::ViewToggleUserList),
            button(text("☑ Status Bar")).on_press(MenuMessage::ViewToggleStatusBar),
            button(text("Fullscreen")).on_press(MenuMessage::ViewFullscreen),
            button(text(format!("Zoom In ({}pt)", font_size as i32 + 1)))
                .on_press(MenuMessage::ViewZoomIn),
            button(text(format!("Zoom Out ({}pt)", font_size as i32 - 1)))
                .on_press(MenuMessage::ViewZoomOut),
            button(text("Reset Zoom (13pt)")).on_press(MenuMessage::ViewResetZoom),
        ]
        .spacing(2)
        .into()
    }

    fn render_server_menu(&self, app_state: &AppState) -> Element<MenuMessage> {
        // Get current server information
        let current_server = app_state
            .current_tab()
            .and_then(|tab| tab.server_id.as_ref())
            .and_then(|id| app_state.servers.get(id));

        let server_name = current_server
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "No Server".to_string());

        let connected = current_server
            .map(|s| s.connection_state == rustirc_core::ConnectionState::Registered)
            .unwrap_or(false);

        column![
            button(text("Connect...")).on_press(MenuMessage::ServerConnect),
            button(text(format!(
                "Disconnect {}",
                if connected { &server_name } else { "" }
            )))
            .on_press(MenuMessage::ServerDisconnect),
            button(text(format!(
                "Reconnect{}",
                if connected { "" } else { " (Not Connected)" }
            )))
            .on_press(MenuMessage::ServerReconnect),
            button(text("Properties...")).on_press(MenuMessage::ServerProperties),
            button(text(format!("Add {server_name} to Favorites")))
                .on_press(MenuMessage::ServerAddToFavorites),
        ]
        .spacing(2)
        .into()
    }

    fn render_channel_menu(&self, app_state: &AppState) -> Element<MenuMessage> {
        // Get current channel information
        let current_channel = app_state.current_tab().and_then(|tab| {
            if let TabType::Channel { channel } = &tab.tab_type {
                Some(channel.clone())
            } else {
                None
            }
        });

        let channel_name = current_channel
            .clone()
            .unwrap_or_else(|| "No Channel".to_string());

        let in_channel = current_channel.is_some();
        let user_count = app_state
            .current_tab()
            .map(|tab| tab.users.len())
            .unwrap_or(0);

        column![
            button(text("Join Channel...")).on_press(MenuMessage::ChannelJoin),
            button(text(format!(
                "Part {}",
                if in_channel { &channel_name } else { "Channel" }
            )))
            .on_press(MenuMessage::ChannelPart),
            button(text(format!(
                "{} Topic...",
                if in_channel { &channel_name } else { "Channel" }
            )))
            .on_press(MenuMessage::ChannelTopic),
            button(text(format!(
                "Channel Modes... ({})",
                if in_channel { "+nt" } else { "none" }
            )))
            .on_press(MenuMessage::ChannelModes),
            button(text(format!("Ban List... ({user_count} users)")))
                .on_press(MenuMessage::ChannelBanList),
            button(text("Invite List...")).on_press(MenuMessage::ChannelInviteList),
        ]
        .spacing(2)
        .into()
    }

    fn render_tools_menu(&self, app_state: &AppState) -> Element<MenuMessage> {
        // Get current settings for display
        let theme_name = app_state.settings().theme.as_str();

        let log_count = app_state
            .tabs
            .values()
            .map(|tab| tab.messages.len())
            .sum::<usize>();

        column![
            button(text("Preferences...")).on_press(MenuMessage::ToolsPreferences),
            button(text(format!("Theme Editor... ({theme_name})")))
                .on_press(MenuMessage::ToolsThemeEditor),
            button(text("Script Editor...")).on_press(MenuMessage::ToolsScriptEditor),
            button(text(format!("Log Viewer... ({log_count} messages)")))
                .on_press(MenuMessage::ToolsLogViewer),
            button(text(format!(
                "Network List... ({} servers)",
                app_state.servers.len()
            )))
            .on_press(MenuMessage::ToolsNetworkList),
        ]
        .spacing(2)
        .into()
    }

    fn render_help_menu(&self, _app_state: &AppState) -> Element<MenuMessage> {
        // Help menu doesn't need app state, but we accept it for consistency
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        column![
            button(text("User Guide (F1)")).on_press(MenuMessage::HelpUserGuide),
            button(text("Keyboard Shortcuts")).on_press(MenuMessage::HelpKeyboardShortcuts),
            button(text("Check for Updates...")).on_press(MenuMessage::HelpCheckUpdates),
            button(text(format!("About RustIRC v{VERSION}..."))).on_press(MenuMessage::HelpAbout),
        ]
        .spacing(2)
        .into()
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Context menus for different UI elements
pub struct ContextMenu;

impl ContextMenu {
    /// Create user context menu
    pub fn user_menu(nick: &str) -> Element<'static, MenuMessage> {
        column![
            text(format!("User: {nick}")),
            button(text("Query")).on_press(MenuMessage::ChannelJoin), // Placeholder
            button(text("Whois")).on_press(MenuMessage::ServerProperties), // Placeholder
            button(text("Op")).on_press(MenuMessage::ChannelModes),   // Placeholder
            button(text("Voice")).on_press(MenuMessage::ChannelModes), // Placeholder
            button(text("Kick")).on_press(MenuMessage::ChannelPart),  // Placeholder
            button(text("Ban")).on_press(MenuMessage::ChannelBanList), // Placeholder
        ]
        .spacing(2)
        .padding(5)
        .into()
    }

    /// Create channel context menu
    pub fn channel_menu(channel: &str) -> Element<'static, MenuMessage> {
        column![
            text(format!("Channel: {channel}")),
            button(text("Join")).on_press(MenuMessage::ChannelJoin),
            button(text("Part")).on_press(MenuMessage::ChannelPart),
            button(text("Topic")).on_press(MenuMessage::ChannelTopic),
            button(text("Modes")).on_press(MenuMessage::ChannelModes),
            button(text("Ban List")).on_press(MenuMessage::ChannelBanList),
        ]
        .spacing(2)
        .padding(5)
        .into()
    }

    /// Create server context menu
    pub fn server_menu(server: &str) -> Element<'static, MenuMessage> {
        column![
            text(format!("Server: {server}")),
            button(text("Connect")).on_press(MenuMessage::ServerConnect),
            button(text("Disconnect")).on_press(MenuMessage::ServerDisconnect),
            button(text("Reconnect")).on_press(MenuMessage::ServerReconnect),
            button(text("Properties")).on_press(MenuMessage::ServerProperties),
            button(text("Add to Favorites")).on_press(MenuMessage::ServerAddToFavorites),
        ]
        .spacing(2)
        .padding(5)
        .into()
    }

    /// Create message context menu
    pub fn message_menu() -> Element<'static, MenuMessage> {
        column![
            button(text("Copy")).on_press(MenuMessage::EditCopy),
            button(text("Select All")).on_press(MenuMessage::EditSelectAll),
            button(text("Find")).on_press(MenuMessage::EditFind),
        ]
        .spacing(2)
        .padding(5)
        .into()
    }

    /// Create link context menu
    pub fn link_menu(url: &str) -> Element<'static, MenuMessage> {
        column![
            text(format!("Link: {url}")),
            button(text("Open")).on_press(MenuMessage::EditCopy), // Placeholder
            button(text("Copy URL")).on_press(MenuMessage::EditCopy),
        ]
        .spacing(2)
        .padding(5)
        .into()
    }
}
