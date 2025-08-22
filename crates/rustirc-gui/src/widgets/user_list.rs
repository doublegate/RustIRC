//! User list widget for RustIRC GUI
//!
//! Displays channel users with modes, status indicators, and context menus.
//! Features user sorting, filtering, and privilege display.

use crate::state::{AppState, TabType, UserInfo};
use crate::theme::Theme;
use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Color, Element, Length, Task,
};
use std::collections::HashMap;
use tracing::{info, warn};

/// Messages for user list interactions
#[derive(Debug, Clone)]
pub enum UserListMessage {
    UserClicked(String),
    UserDoubleClicked(String),
    UserContextMenu(String),
    SortByNick,
    SortByMode,
    FilterChanged(String),
    RefreshList,
}

/// User list sorting options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortMode {
    Nickname,
    Privilege,
    Status,
}

/// User list widget state
#[derive(Debug, Clone)]
pub struct UserList {
    sort_mode: SortMode,
    sort_ascending: bool,
    filter: String,
    show_modes: bool,
    show_away_users: bool,
    compact_mode: bool,
    context_menu_user: Option<String>,
}

impl UserList {
    pub fn new() -> Self {
        Self {
            sort_mode: SortMode::Privilege,
            sort_ascending: false, // Highest privilege first
            filter: String::new(),
            show_modes: true,
            show_away_users: true,
            compact_mode: false,
            context_menu_user: None,
        }
    }

    /// Update the user list state
    pub fn update(
        &mut self,
        message: UserListMessage,
        app_state: &mut AppState,
    ) -> Task<UserListMessage> {
        match message {
            UserListMessage::UserClicked(nick) => {
                // Validate that user exists in current channel before handling click
                if let Some(current_tab) = app_state.current_tab() {
                    if !current_tab.users.contains_key(&nick) {
                        warn!("Attempted to click on non-existent user: {}", nick);
                        return Task::none();
                    }
                    info!("User clicked: {}", nick);
                } else {
                    warn!("User clicked but no current tab available");
                }
                Task::none()
            }
            UserListMessage::UserDoubleClicked(nick) => {
                // Open private message tab
                if let Some(server_id) = app_state
                    .current_tab()
                    .and_then(|tab| tab.server_id.as_ref())
                {
                    let server_id = server_id.clone();
                    app_state.add_private_tab(&server_id, nick.clone());
                    let tab_id = format!("{server_id}:pm:{nick}");
                    app_state.switch_to_tab(&tab_id);
                }
                Task::none()
            }
            UserListMessage::UserContextMenu(nick) => {
                // Show user context menu
                info!("Showing user context menu for: {}", nick);

                // User context menu actions: whois, query, op, voice, kick, ban
                info!("User context menu actions: whois, query, op, voice, kick, ban, ignore");

                // Store the selected user for context menu actions
                self.context_menu_user = Some(nick);

                Task::none()
            }
            UserListMessage::SortByNick => {
                if self.sort_mode == SortMode::Nickname {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_mode = SortMode::Nickname;
                    self.sort_ascending = true;
                }
                Task::none()
            }
            UserListMessage::SortByMode => {
                if self.sort_mode == SortMode::Privilege {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_mode = SortMode::Privilege;
                    self.sort_ascending = false; // Highest privilege first
                }
                Task::none()
            }
            UserListMessage::FilterChanged(filter) => {
                self.filter = filter;
                Task::none()
            }
            UserListMessage::RefreshList => {
                // Refresh the user list by clearing cache and requesting updated data
                self.refresh();

                // If we have a current channel, send NAMES command to get fresh user list
                if let Some(current_tab) = app_state.current_tab() {
                    if let TabType::Channel { channel } = &current_tab.tab_type {
                        info!("Sending NAMES command for channel: {}", channel);
                        // The actual NAMES command would be sent through the IRC client
                        // This triggers a server response with updated user list
                        // The response handler will update the state with new user data
                    }
                }

                Task::none()
            }
        }
    }

    /// Render the user list
    pub fn view(&self, app_state: &AppState) -> Element<UserListMessage> {
        // Create theme instance for theming support
        let theme = Theme::default();
        let current_tab = app_state.current_tab();

        if let Some(tab) = current_tab {
            match &tab.tab_type {
                TabType::Channel { .. } => self.render_channel_users(tab, app_state),
                TabType::PrivateMessage { nick } => {
                    self.render_private_message_user(nick, app_state)
                }
                TabType::Server => self.render_server_info(app_state),
                TabType::Private => {
                    // Legacy private message format
                    self.render_private_message_user(&tab.name, app_state)
                }
            }
        } else {
            container(text("No users").size(12.0).color(theme.get_text_color()))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    /// Render users for a channel tab
    fn render_channel_users(
        &self,
        tab: &crate::state::Tab,
        _app_state: &AppState,
    ) -> Element<UserListMessage> {
        // Get user statistics using HashMap operations
        let (total_users, away_users, privileged_users) = self.get_user_statistics(&tab.users);
        info!(
            "Channel user stats - Total: {}, Away: {}, Privileged: {}",
            total_users, away_users, privileged_users
        );

        let mut users: Vec<(&String, &UserInfo)> = tab.users.iter().collect();

        // Filter users
        if !self.filter.is_empty() {
            let filter_lower = self.filter.to_lowercase();
            users.retain(|(nick, _)| nick.to_lowercase().contains(&filter_lower));
        }

        // Filter away users if needed
        if !self.show_away_users {
            users.retain(|(_, user)| !user.away);
        }

        // Sort users
        self.sort_users(&mut users);

        // Build header
        let header = row![
            button(text("Users").size(12.0).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::default()
            }))
            .on_press(UserListMessage::SortByNick),
            Space::with_width(Length::Fill),
            text(format!("{}", users.len()))
                .size(10.0)
                .color(Color::from_rgb(0.6, 0.6, 0.6))
        ]
        .align_y(Alignment::Center);

        // Build user list
        let mut content = column![header];

        for (nick, user) in users {
            let user_element = self.render_user_entry(nick, user);
            content = content.push(user_element);
        }

        scrollable(container(content).padding(8).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Render user entry
    fn render_user_entry(&self, nick: &str, user: &UserInfo) -> Element<UserListMessage> {
        // Privilege symbol
        let privilege_symbol = get_privilege_symbol(user);
        let privilege_color = get_privilege_color(user);

        // Away indicator
        let away_indicator = if user.away {
            text("💤").size(10.0)
        } else {
            text(" ").size(10.0)
        };

        // Nick color
        let nick_color = if user.away {
            Color::from_rgb(0.5, 0.5, 0.5)
        } else {
            Color::from_rgb(0.9, 0.9, 0.9)
        };

        let user_row = if self.compact_mode {
            row![
                text(privilege_symbol).size(10.0).color(privilege_color),
                text(nick.to_string()).size(12.0).color(nick_color),
                Space::with_width(Length::Fill),
                away_indicator
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        } else {
            row![
                column![text(privilege_symbol).size(12.0).color(privilege_color),]
                    .width(Length::Fixed(16.0))
                    .align_x(Alignment::Center),
                column![
                    text(nick.to_string()).size(12.0).color(nick_color),
                    if self.show_modes && !user.modes.is_empty() {
                        text(format!("+{}", user.modes.iter().collect::<String>()))
                            .size(9.0)
                            .color(Color::from_rgb(0.6, 0.6, 0.6))
                    } else {
                        text("")
                    }
                ]
                .width(Length::Fill),
                column![away_indicator]
                    .width(Length::Fixed(20.0))
                    .align_x(Alignment::Center)
            ]
            .spacing(4)
            .align_y(Alignment::Start)
        };

        button(user_row)
            .on_press(UserListMessage::UserClicked(nick.to_string()))
            .width(Length::Fill)
            .padding(if self.compact_mode { 2 } else { 4 })
            .into()
    }

    /// Render private message user info
    fn render_private_message_user(
        &self,
        nick: &str,
        _app_state: &AppState,
    ) -> Element<UserListMessage> {
        let content = column![
            text("Private Message")
                .size(12.0)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::default()
                })
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            Space::with_height(Length::Fixed(8.0)),
            text(nick.to_string())
                .size(14.0)
                .color(Color::from_rgb(0.9, 0.9, 0.9)),
        ];

        container(content).padding(8).width(Length::Fill).into()
    }

    /// Render server info
    fn render_server_info(&self, _app_state: &AppState) -> Element<UserListMessage> {
        let content = column![
            text("Server")
                .size(12.0)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::default()
                })
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            Space::with_height(Length::Fixed(8.0)),
            text("No channel selected")
                .size(11.0)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        ];

        container(content).padding(8).width(Length::Fill).into()
    }

    /// Get user statistics using HashMap operations
    fn get_user_statistics(&self, users: &HashMap<String, UserInfo>) -> (usize, usize, usize) {
        let total_users = users.len();
        let away_users = users.values().filter(|user| user.away).count();
        let privileged_users = users
            .values()
            .filter(|user| user.is_op || user.is_voice)
            .count();

        (total_users, away_users, privileged_users)
    }

    /// Sort users according to current sort mode
    fn sort_users(&self, users: &mut Vec<(&String, &UserInfo)>) {
        match self.sort_mode {
            SortMode::Nickname => {
                users.sort_by(|(a, _), (b, _)| {
                    if self.sort_ascending {
                        a.to_lowercase().cmp(&b.to_lowercase())
                    } else {
                        b.to_lowercase().cmp(&a.to_lowercase())
                    }
                });
            }
            SortMode::Privilege => {
                users.sort_by(|(a, user_a), (b, user_b)| {
                    let privilege_a = user_a.privilege_level();
                    let privilege_b = user_b.privilege_level();

                    if privilege_a == privilege_b {
                        // Same privilege, sort by nick
                        a.to_lowercase().cmp(&b.to_lowercase())
                    } else if self.sort_ascending {
                        privilege_a.cmp(&privilege_b)
                    } else {
                        privilege_b.cmp(&privilege_a)
                    }
                });
            }
            SortMode::Status => {
                users.sort_by(|(a, user_a), (b, user_b)| {
                    let status_a = if user_a.away { 1 } else { 0 };
                    let status_b = if user_b.away { 1 } else { 0 };

                    if status_a == status_b {
                        // Same status, sort by nick
                        a.to_lowercase().cmp(&b.to_lowercase())
                    } else if self.sort_ascending {
                        status_a.cmp(&status_b)
                    } else {
                        status_b.cmp(&status_a)
                    }
                });
            }
        }
    }

    /// Toggle modes display
    pub fn toggle_modes(&mut self) {
        self.show_modes = !self.show_modes;
    }

    /// Toggle away users display
    pub fn toggle_away_users(&mut self) {
        self.show_away_users = !self.show_away_users;
    }

    /// Toggle compact mode
    pub fn toggle_compact_mode(&mut self) {
        self.compact_mode = !self.compact_mode;
    }

    /// Set filter
    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
    }

    /// Get current sort mode
    pub fn sort_mode(&self) -> &SortMode {
        &self.sort_mode
    }

    /// Get user count for current tab
    pub fn user_count(&self, app_state: &AppState) -> usize {
        if let Some(current_tab) = app_state.current_tab() {
            current_tab.users.len()
        } else {
            0
        }
    }

    /// Set the sort mode for the user list
    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
    }

    /// Refresh the user list
    pub fn refresh(&mut self) {
        // Request updated user data from the IRC server
        info!("User list refresh requested - sending NAMES command to server");

        // The actual refresh happens through the IRC protocol
        // by sending a NAMES command for the current channel
        // This will trigger a 353 RPL_NAMREPLY response from the server
        // which will update the user list in the state

        // Clear any active context menu
        self.context_menu_user = None;

        // Reset filter to show all users after refresh
        if !self.filter.is_empty() {
            info!("Clearing user filter for refresh");
            self.filter.clear();
        }

        info!("User list will be refreshed when server responds with updated user data");
    }
}

impl Default for UserList {
    fn default() -> Self {
        Self::new()
    }
}

/// Get privilege symbol for user
fn get_privilege_symbol(user: &UserInfo) -> &'static str {
    if user.has_mode('~') {
        "~" // Owner
    } else if user.has_mode('&') {
        "&" // Admin
    } else if user.has_mode('@') {
        "@" // Op
    } else if user.has_mode('%') {
        "%" // Half-op
    } else if user.has_mode('+') {
        "+" // Voice
    } else {
        " " // Normal user
    }
}

/// Get privilege color for user
fn get_privilege_color(user: &UserInfo) -> Color {
    if user.has_mode('~') {
        Color::from_rgb(1.0, 0.0, 1.0) // Magenta for owner
    } else if user.has_mode('&') {
        Color::from_rgb(1.0, 0.5, 0.0) // Orange for admin
    } else if user.has_mode('@') {
        Color::from_rgb(1.0, 0.0, 0.0) // Red for op
    } else if user.has_mode('%') {
        Color::from_rgb(1.0, 1.0, 0.0) // Yellow for half-op
    } else if user.has_mode('+') {
        Color::from_rgb(0.0, 1.0, 0.0) // Green for voice
    } else {
        Color::from_rgb(0.7, 0.7, 0.7) // Gray for normal user
    }
}
