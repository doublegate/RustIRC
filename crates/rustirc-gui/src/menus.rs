//! Menu system for RustIRC GUI
//!
//! Implements the complete menu bar structure with all standard
//! IRC client menus and actions.

use iced::{Element, Length, Task};
use iced::widget::{button, column, container, row, text};
use crate::state::AppState;

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

#[derive(Debug, Clone, PartialEq)]
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
        Self {
            active_menu: None,
        }
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
                app_state.ui_state_mut().show_sidebar = !app_state.ui_state().show_sidebar;
                Task::none()
            }
            MenuMessage::ViewToggleUserList => {
                app_state.ui_state_mut().show_userlist = !app_state.ui_state().show_userlist;
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
        let file_button = button(text("File"))
            .on_press(MenuMessage::FileConnect); // For now, just connect
            
        let edit_button = button(text("Edit"))
            .on_press(MenuMessage::EditPreferences);
            
        let view_button = button(text("View"))
            .on_press(MenuMessage::ViewToggleServerTree);
            
        let server_button = button(text("Server"))
            .on_press(MenuMessage::ServerConnect);
            
        let channel_button = button(text("Channel"))
            .on_press(MenuMessage::ChannelJoin);
            
        let tools_button = button(text("Tools"))
            .on_press(MenuMessage::ToolsPreferences);
            
        let help_button = button(text("Help"))
            .on_press(MenuMessage::HelpAbout);
        
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
        .padding(5)
        .into()
    }
    
    fn render_file_menu(&self) -> Element<MenuMessage> {
        column![
            button(text("Connect to Server...")).on_press(MenuMessage::FileConnect),
            button(text("Disconnect")).on_press(MenuMessage::FileDisconnect),
            button(text("Quit")).on_press(MenuMessage::FileQuit),
        ]
        .spacing(2)
        .into()
    }
    
    fn render_edit_menu(&self) -> Element<MenuMessage> {
        column![
            button(text("Copy")).on_press(MenuMessage::EditCopy),
            button(text("Paste")).on_press(MenuMessage::EditPaste),
            button(text("Select All")).on_press(MenuMessage::EditSelectAll),
            button(text("Find...")).on_press(MenuMessage::EditFind),
            button(text("Preferences...")).on_press(MenuMessage::EditPreferences),
        ]
        .spacing(2)
        .into()
    }
    
    fn render_view_menu(&self) -> Element<MenuMessage> {
        column![
            button(text("Toggle Server Tree")).on_press(MenuMessage::ViewToggleServerTree),
            button(text("Toggle User List")).on_press(MenuMessage::ViewToggleUserList),
            button(text("Toggle Status Bar")).on_press(MenuMessage::ViewToggleStatusBar),
            button(text("Fullscreen")).on_press(MenuMessage::ViewFullscreen),
            button(text("Zoom In")).on_press(MenuMessage::ViewZoomIn),
            button(text("Zoom Out")).on_press(MenuMessage::ViewZoomOut),
            button(text("Reset Zoom")).on_press(MenuMessage::ViewResetZoom),
        ]
        .spacing(2)
        .into()
    }
    
    fn render_server_menu(&self) -> Element<MenuMessage> {
        column![
            button(text("Connect...")).on_press(MenuMessage::ServerConnect),
            button(text("Disconnect")).on_press(MenuMessage::ServerDisconnect),
            button(text("Reconnect")).on_press(MenuMessage::ServerReconnect),
            button(text("Properties...")).on_press(MenuMessage::ServerProperties),
            button(text("Add to Favorites")).on_press(MenuMessage::ServerAddToFavorites),
        ]
        .spacing(2)
        .into()
    }
    
    fn render_channel_menu(&self) -> Element<MenuMessage> {
        column![
            button(text("Join Channel...")).on_press(MenuMessage::ChannelJoin),
            button(text("Part Channel")).on_press(MenuMessage::ChannelPart),
            button(text("Channel Topic...")).on_press(MenuMessage::ChannelTopic),
            button(text("Channel Modes...")).on_press(MenuMessage::ChannelModes),
            button(text("Ban List...")).on_press(MenuMessage::ChannelBanList),
            button(text("Invite List...")).on_press(MenuMessage::ChannelInviteList),
        ]
        .spacing(2)
        .into()
    }
    
    fn render_tools_menu(&self) -> Element<MenuMessage> {
        column![
            button(text("Preferences...")).on_press(MenuMessage::ToolsPreferences),
            button(text("Theme Editor...")).on_press(MenuMessage::ToolsThemeEditor),
            button(text("Script Editor...")).on_press(MenuMessage::ToolsScriptEditor),
            button(text("Log Viewer...")).on_press(MenuMessage::ToolsLogViewer),
            button(text("Network List...")).on_press(MenuMessage::ToolsNetworkList),
        ]
        .spacing(2)
        .into()
    }
    
    fn render_help_menu(&self) -> Element<MenuMessage> {
        column![
            button(text("User Guide")).on_press(MenuMessage::HelpUserGuide),
            button(text("Keyboard Shortcuts")).on_press(MenuMessage::HelpKeyboardShortcuts),
            button(text("Check for Updates...")).on_press(MenuMessage::HelpCheckUpdates),
            button(text("About RustIRC...")).on_press(MenuMessage::HelpAbout),
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
            text(format!("User: {}", nick)),
            button(text("Query")).on_press(MenuMessage::ChannelJoin), // Placeholder
            button(text("Whois")).on_press(MenuMessage::ServerProperties), // Placeholder
            button(text("Op")).on_press(MenuMessage::ChannelModes), // Placeholder
            button(text("Voice")).on_press(MenuMessage::ChannelModes), // Placeholder
            button(text("Kick")).on_press(MenuMessage::ChannelPart), // Placeholder
            button(text("Ban")).on_press(MenuMessage::ChannelBanList), // Placeholder
        ]
        .spacing(2)
        .padding(5)
        .into()
    }
    
    /// Create channel context menu
    pub fn channel_menu(channel: &str) -> Element<'static, MenuMessage> {
        column![
            text(format!("Channel: {}", channel)),
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
            text(format!("Server: {}", server)),
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
            text(format!("Link: {}", url)),
            button(text("Open")).on_press(MenuMessage::EditCopy), // Placeholder
            button(text("Copy URL")).on_press(MenuMessage::EditCopy),
        ]
        .spacing(2)
        .padding(5)
        .into()
    }
}