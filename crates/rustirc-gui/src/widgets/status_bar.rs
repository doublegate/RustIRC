//! Status bar widget for RustIRC GUI
//!
//! Displays connection status, channel information, user count, lag, and other status indicators.
//! Features mode display, topic bar, and network information.

use crate::state::{AppState, ConnectionState, TabType};
use crate::theme::Theme;
use iced::{
    widget::{container, text, row, Space},
    Element, Length, Task, Color, Alignment,
};
use std::time::{SystemTime, Duration};

/// Messages for status bar interactions
#[derive(Debug, Clone)]
pub enum StatusBarMessage {
    UpdateStatus,
    ToggleTopicBar,
    ClearStatus,
}

/// Status bar widget state
#[derive(Debug, Clone)]
pub struct StatusBar {
    show_topic: bool,
    show_lag: bool,
    show_user_count: bool,
    show_modes: bool,
    last_update: SystemTime,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            show_topic: true,
            show_lag: true,
            show_user_count: true,
            show_modes: true,
            last_update: SystemTime::now(),
        }
    }

    /// Update the status bar state
    pub fn update(&mut self, message: StatusBarMessage, _app_state: &mut AppState) -> Task<StatusBarMessage> {
        match message {
            StatusBarMessage::UpdateStatus => {
                self.last_update = SystemTime::now();
                Task::none()
            }
            StatusBarMessage::ToggleTopicBar => {
                self.show_topic = !self.show_topic;
                Task::none()
            }
            StatusBarMessage::ClearStatus => {
                Task::none()
            }
        }
    }

    /// Render the status bar
    pub fn view(&self, app_state: &AppState) -> Element<StatusBarMessage> {
        let current_tab = app_state.current_tab();
        
        // Main status row
        let mut status_content = row![];

        // Connection status
        let connection_status = self.get_connection_status(app_state);
        status_content = status_content.push(
            text(connection_status)
                .size(11.0)
                .color(Color::from_rgb(0.7, 0.7, 0.7))
        );

        status_content = status_content.push(Space::with_width(Length::Fixed(8.0)));
        status_content = status_content.push(
            text("|")
                .size(11.0)
                .color(Color::from_rgb(0.5, 0.5, 0.5))
        );
        status_content = status_content.push(Space::with_width(Length::Fixed(8.0)));

        // Tab-specific information
        if let Some(tab) = current_tab {
            let tab_info = self.get_tab_info(tab, app_state);
            status_content = status_content.push(
                text(tab_info)
                    .size(11.0)
                    .color(Color::from_rgb(0.8, 0.8, 0.8))
            );

            // User count for channels
            if let TabType::Channel { .. } = tab.tab_type {
                if self.show_user_count {
                    let user_count = tab.users.len();
                    status_content = status_content.push(Space::with_width(Length::Fixed(8.0)));
                    status_content = status_content.push(
                        text(format!("({} users)", user_count))
                            .size(11.0)
                            .color(Color::from_rgb(0.6, 0.6, 0.6))
                    );
                }
            }

            // User modes
            if self.show_modes {
                if let Some(server_id) = &tab.server_id {
                    if let Some(server_state) = app_state.servers().get(server_id) {
                        if !server_state.modes.is_empty() {
                            status_content = status_content.push(Space::with_width(Length::Fixed(8.0)));
                            status_content = status_content.push(
                                text(format!("+{}", server_state.modes.join("")))
                                    .size(11.0)
                                    .color(Color::from_rgb(0.0, 0.8, 0.0))
                            );
                        }
                    }
                }
            }
        }

        // Spacer
        status_content = status_content.push(Space::with_width(Length::Fill));

        // Lag information
        if self.show_lag {
            let lag_info = self.get_lag_info(app_state);
            if !lag_info.is_empty() {
                status_content = status_content.push(
                    text(lag_info)
                        .size(11.0)
                        .color(Color::from_rgb(0.6, 0.6, 0.6))
                );
                status_content = status_content.push(Space::with_width(Length::Fixed(8.0)));
            }
        }

        // Time
        let time_str = self.get_current_time();
        status_content = status_content.push(
            text(time_str)
                .size(11.0)
                .color(Color::from_rgb(0.7, 0.7, 0.7))
        );

        let status_row = container(status_content)
            .padding([4, 8])
            .width(Length::Fill);

        // Topic bar (if enabled and applicable)
        if self.show_topic {
            if let Some(topic_bar) = self.render_topic_bar(app_state) {
                return container(
                    iced::widget::column![
                        topic_bar,
                        status_row
                    ]
                )
                .width(Length::Fill)
                .into();
            }
        }

        status_row.into()
    }

    /// Get connection status text
    fn get_connection_status(&self, app_state: &AppState) -> String {
        let current_tab = app_state.current_tab();
        
        if let Some(tab) = current_tab {
            if let Some(server_id) = &tab.server_id {
                if let Some(server_state) = app_state.servers().get(server_id) {
                    match server_state.connection_state {
                        ConnectionState::Disconnected => "Disconnected".to_string(),
                        ConnectionState::Connecting => "Connecting...".to_string(),
                        ConnectionState::Connected => "Connected".to_string(),
                        ConnectionState::Authenticating => "Authenticating...".to_string(),
                        ConnectionState::Registered => {
                            if server_state.nickname.is_empty() {
                                "Registered".to_string()
                            } else {
                                format!("Registered as {}", server_state.nickname)
                            }
                        }
                        ConnectionState::Reconnecting => "Reconnecting...".to_string(),
                        ConnectionState::Failed(ref error) => format!("Failed: {}", error),
                    }
                } else {
                    "Unknown server".to_string()
                }
            } else {
                "No server".to_string()
            }
        } else {
            "No connection".to_string()
        }
    }

    /// Get tab-specific information
    fn get_tab_info(&self, tab: &crate::state::Tab, _app_state: &AppState) -> String {
        match &tab.tab_type {
            TabType::Server => {
                if let Some(server_id) = &tab.server_id {
                    format!("Server: {}", server_id)
                } else {
                    "Server".to_string()
                }
            }
            TabType::Channel { channel } => {
                format!("Channel: {}", channel)
            }
            TabType::PrivateMessage { nick } => {
                format!("Private: {}", nick)
            }
        }
    }

    /// Get lag information
    fn get_lag_info(&self, _app_state: &AppState) -> String {
        // TODO: Implement actual lag measurement
        // For now, return empty string
        String::new()
    }

    /// Get current time
    fn get_current_time(&self) -> String {
        let now = SystemTime::now();
        let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
        let secs = duration.as_secs();
        
        let hours = (secs / 3600) % 24;
        let minutes = (secs / 60) % 60;
        
        format!("{:02}:{:02}", hours, minutes)
    }

    /// Render topic bar if applicable
    fn render_topic_bar(&self, app_state: &AppState) -> Option<Element<StatusBarMessage>> {
        let current_tab = app_state.current_tab()?;
        
        if let TabType::Channel { channel } = &current_tab.tab_type {
            if let Some(server_id) = &current_tab.server_id {
                if let Some(server_state) = app_state.servers().get(server_id) {
                    if let Some(channel_state) = server_state.channels.get(channel) {
                        if let Some(ref topic) = channel_state.topic {
                            let topic_text = if topic.len() > 100 {
                                format!("{}...", &topic[..97])
                            } else {
                                topic.clone()
                            };

                            return Some(
                                container(
                                    row![
                                        text("Topic:")
                                            .size(11.0)
                                            .color(Color::from_rgb(0.6, 0.6, 0.6)),
                                        Space::with_width(Length::Fixed(8.0)),
                                        text(topic_text)
                                            .size(11.0)
                                            .color(Color::from_rgb(0.8, 0.8, 0.8))
                                    ]
                                    .align_y(Alignment::Center)
                                )
                                .padding([4, 8])
                                .width(Length::Fill)
                                .into()
                            );
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Toggle topic display
    pub fn toggle_topic(&mut self) {
        self.show_topic = !self.show_topic;
    }

    /// Toggle lag display
    pub fn toggle_lag(&mut self) {
        self.show_lag = !self.show_lag;
    }

    /// Toggle user count display
    pub fn toggle_user_count(&mut self) {
        self.show_user_count = !self.show_user_count;
    }

    /// Toggle modes display
    pub fn toggle_modes(&mut self) {
        self.show_modes = !self.show_modes;
    }

    /// Get status summary
    pub fn get_status_summary(&self, app_state: &AppState) -> String {
        let connection_status = self.get_connection_status(app_state);
        let tab_count = app_state.tabs().len();
        
        format!("{} | {} tabs", connection_status, tab_count)
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}