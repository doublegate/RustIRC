//! User list widget for RustIRC GUI
//!
//! Displays channel users with modes, status indicators, and context menus.
//! Features user sorting, filtering, and privilege display.

use crate::state::{AppState, UserState, TabType};
use crate::theme::Theme;
use iced::{
    widget::{container, scrollable, text, button, column, row, Space},
    Element, Length, Task, Color, Alignment,
};
use std::collections::HashMap;

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
        }
    }

    /// Update the user list state
    pub fn update(&mut self, message: UserListMessage, app_state: &mut AppState) -> Task<UserListMessage> {
        match message {
            UserListMessage::UserClicked(nick) => {
                // Handle user selection
                Task::none()
            }
            UserListMessage::UserDoubleClicked(nick) => {
                // Open private message tab
                if let Some(server_id) = app_state.current_tab()
                    .and_then(|tab| tab.server_id.as_ref()) {
                    let server_id = server_id.clone();
                    app_state.add_private_tab(&server_id, &nick);
                    let tab_id = format!("{}:query:{}", server_id, nick);
                    app_state.switch_to_tab(&tab_id);
                }
                Task::none()
            }
            UserListMessage::UserContextMenu(nick) => {
                // TODO: Show user context menu
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
                Task::none()
            }
        }
    }

    /// Render the user list
    pub fn view<'a>(&'a self, app_state: &'a AppState) -> Element<'a, UserListMessage> {
        let current_tab = app_state.current_tab();
        
        if let Some(tab) = current_tab {
            match &tab.tab_type {
                TabType::Channel { .. } => {
                    self.render_channel_users(tab, app_state)
                }
                TabType::PrivateMessage { nick } => {
                    self.render_private_message_user(nick, app_state)
                }
                TabType::Server => {
                    self.render_server_info(app_state)
                }
            }
        } else {
            container(
                text("No users")
                    .size(12.0)
                    .color(Color::from_rgb(0.6, 0.6, 0.6))
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }

    /// Render users for a channel tab
    fn render_channel_users<'a>(&self, tab: &'a crate::state::Tab, _app_state: &AppState) -> Element<'a, UserListMessage> {
        let mut users: Vec<(&String, &UserState)> = tab.users.iter().collect();
        
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
            button(text("Users").size(12.0).font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::default() }))
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

        scrollable(
            container(content)
                .padding(8)
                .width(Length::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Render user entry
    fn render_user_entry<'a>(&self, nick: &'a str, user: &UserState) -> Element<'a, UserListMessage> {
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
                text(privilege_symbol)
                    .size(10.0)
                    .color(privilege_color),
                text(nick)
                    .size(12.0)
                    .color(nick_color),
                Space::with_width(Length::Fill),
                away_indicator
            ]
            .spacing(4)
            .align_y(Alignment::Center)
        } else {
            row![
                column![
                    text(privilege_symbol)
                        .size(12.0)
                        .color(privilege_color),
                ]
                .width(Length::Fixed(16.0))
                .align_x(Alignment::Center),
                column![
                    text(nick)
                        .size(12.0)
                        .color(nick_color),
                    if self.show_modes && !user.modes.is_empty() {
                        text(format!("+{}", user.modes.join("")))
                            .size(9.0)
                            .color(Color::from_rgb(0.6, 0.6, 0.6))
                    } else {
                        text("")
                    }
                ]
                .width(Length::Fill),
                column![
                    away_indicator
                ]
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
    fn render_private_message_user<'a>(&self, nick: &'a str, _app_state: &AppState) -> Element<'a, UserListMessage> {
        let content = column![
            text("Private Message")
                .size(12.0)
                .font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::default() })
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            Space::with_height(Length::Fixed(8.0)),
            text(nick)
                .size(14.0)
                .color(Color::from_rgb(0.9, 0.9, 0.9)),
        ];

        container(content)
            .padding(8)
            .width(Length::Fill)
            .into()
    }

    /// Render server info
    fn render_server_info(&self, _app_state: &AppState) -> Element<UserListMessage> {
        let content = column![
            text("Server")
                .size(12.0)
                .font(iced::Font { weight: iced::font::Weight::Bold, ..iced::Font::default() })
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            Space::with_height(Length::Fixed(8.0)),
            text("No channel selected")
                .size(11.0)
                .color(Color::from_rgb(0.6, 0.6, 0.6)),
        ];

        container(content)
            .padding(8)
            .width(Length::Fill)
            .into()
    }

    /// Sort users according to current sort mode
    fn sort_users(&self, users: &mut Vec<(&String, &UserState)>) {
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
}

impl Default for UserList {
    fn default() -> Self {
        Self::new()
    }
}

/// Get privilege symbol for user
fn get_privilege_symbol(user: &UserState) -> &'static str {
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
fn get_privilege_color(user: &UserState) -> Color {
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