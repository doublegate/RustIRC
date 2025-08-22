//! Server tree widget for RustIRC GUI
//!
//! Displays a hierarchical tree of servers and their channels.
//! Features server status indicators, channel activity badges, and context menus.

use crate::state::{AppState, TabType};
use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Color, Element, Length, Task,
};
use rustirc_core::connection::ConnectionState;
use tracing::{info, warn};

/// Messages for server tree interactions
#[derive(Debug, Clone)]
pub enum ServerTreeMessage {
    ServerClicked(String),
    ChannelClicked(String),
    ServerContextMenu(String),
    ChannelContextMenu(String),
    ExpandServer(String),
    CollapseServer(String),
}

/// Server tree widget state
#[derive(Debug, Clone)]
pub struct ServerTree {
    expanded_servers: std::collections::HashSet<String>,
}

impl ServerTree {
    pub fn new() -> Self {
        Self {
            expanded_servers: std::collections::HashSet::new(),
        }
    }

    /// Update the server tree state
    pub fn update(
        &mut self,
        message: ServerTreeMessage,
        app_state: &mut AppState,
    ) -> Task<ServerTreeMessage> {
        match message {
            ServerTreeMessage::ServerClicked(server_id) => {
                let tab_id = format!("server:{server_id}");
                app_state.switch_to_tab(&tab_id);
                Task::none()
            }
            ServerTreeMessage::ChannelClicked(tab_id) => {
                app_state.switch_to_tab(&tab_id);
                Task::none()
            }
            ServerTreeMessage::ExpandServer(server_id) => {
                self.expanded_servers.insert(server_id);
                Task::none()
            }
            ServerTreeMessage::CollapseServer(server_id) => {
                self.expanded_servers.remove(&server_id);
                Task::none()
            }
            ServerTreeMessage::ServerContextMenu(server_id) => {
                // Show server context menu
                info!("Showing server context menu for: {}", server_id);

                // Validate server exists before showing menu
                if !app_state.servers.contains_key(&server_id) {
                    warn!(
                        "Attempted to show context menu for non-existent server: {}",
                        server_id
                    );
                    return Task::none();
                }

                // Server context menu actions: connect, disconnect, edit, remove
                // For now, just log available actions
                info!(
                    "Server context menu actions: connect, disconnect, edit, remove, add channel"
                );

                Task::none()
            }
            ServerTreeMessage::ChannelContextMenu(channel_id) => {
                // Show channel context menu
                info!("Showing channel context menu for: {}", channel_id);

                // Channel context menu actions: part, rejoin, clear messages, channel info
                info!("Channel context menu actions: part, rejoin, clear messages, channel info, user list");

                Task::none()
            }
        }
    }

    /// Render the server tree
    pub fn view(&self, app_state: &AppState) -> Element<'_, ServerTreeMessage> {
        let mut content = column![];

        for (server_id, server_state) in &app_state.servers {
            let is_expanded = self.expanded_servers.contains(server_id);

            // Server header
            let server_indicator = self.get_connection_indicator(&server_state.connection_state);
            let server_name = text(server_id.clone()).size(14);
            let expand_button: Element<ServerTreeMessage> = if server_state.channels.is_empty() {
                Space::new(Length::Fixed(16.0), Length::Fixed(16.0)).into()
            } else if is_expanded {
                button(text("▼").size(12))
                    .on_press(ServerTreeMessage::CollapseServer(server_id.clone()))
                    .padding(2)
                    .into()
            } else {
                button(text("▶").size(12))
                    .on_press(ServerTreeMessage::ExpandServer(server_id.clone()))
                    .padding(2)
                    .into()
            };

            let server_row = button(
                row![expand_button, server_indicator, server_name]
                    .spacing(8)
                    .align_y(Alignment::Center),
            )
            .on_press(ServerTreeMessage::ServerClicked(server_id.clone()))
            .width(Length::Fill)
            .padding(4);

            content = content.push(server_row);

            // Channel list (if expanded)
            if is_expanded {
                for channel_name in server_state.channels.keys() {
                    let tab_id = format!("{server_id}:{channel_name}");
                    let is_active = app_state
                        .current_tab()
                        .map(|tab| match &tab.tab_type {
                            TabType::Channel { channel } => channel == channel_name,
                            _ => false,
                        })
                        .unwrap_or(false);

                    let activity_indicator = if let Some(tab) = app_state.tabs.get(&tab_id) {
                        if tab.has_highlight {
                            text("●").color(Color::from_rgb(1.0, 0.0, 0.0)) // Red for highlights
                        } else if tab.has_activity {
                            text("●").color(Color::from_rgb(0.0, 0.6, 1.0)) // Blue for activity
                        } else {
                            text(" ")
                        }
                    } else {
                        text(" ")
                    };

                    let channel_name_style = if is_active {
                        text(channel_name.clone())
                            .size(13)
                            .color(Color::from_rgb(0.4, 0.6, 1.0))
                    } else {
                        text(channel_name.clone()).size(13)
                    };

                    let channel_row = button(
                        row![
                            Space::new(Length::Fixed(24.0), Length::Fixed(16.0)),
                            text("#").size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
                            channel_name_style,
                            Space::with_width(Length::Fill),
                            activity_indicator
                        ]
                        .spacing(4)
                        .align_y(Alignment::Center),
                    )
                    .on_press(ServerTreeMessage::ChannelClicked(tab_id))
                    .width(Length::Fill)
                    .padding(2);

                    content = content.push(channel_row);
                }

                // Private message tabs
                for (tab_id, tab) in &app_state.tabs {
                    if let TabType::PrivateMessage { nick } = &tab.tab_type {
                        if tab.server_id.as_ref() == Some(server_id) {
                            let is_active = app_state
                                .current_tab()
                                .map(|current_tab| current_tab.tab_type == tab.tab_type)
                                .unwrap_or(false);

                            let activity_indicator = if tab.has_highlight {
                                text("●").color(Color::from_rgb(1.0, 0.0, 0.0))
                            } else if tab.has_activity {
                                text("●").color(Color::from_rgb(0.0, 0.6, 1.0))
                            } else {
                                text(" ")
                            };

                            let nick_style = if is_active {
                                text(nick.clone())
                                    .size(13)
                                    .color(Color::from_rgb(0.4, 0.6, 1.0))
                            } else {
                                text(nick.clone()).size(13)
                            };

                            let pm_row = button(
                                row![
                                    Space::new(Length::Fixed(24.0), Length::Fixed(16.0)),
                                    text("@").size(12).color(Color::from_rgb(0.6, 0.6, 0.6)),
                                    nick_style,
                                    Space::with_width(Length::Fill),
                                    activity_indicator
                                ]
                                .spacing(4)
                                .align_y(Alignment::Center),
                            )
                            .on_press(ServerTreeMessage::ChannelClicked(tab_id.clone()))
                            .width(Length::Fill)
                            .padding(2);

                            content = content.push(pm_row);
                        }
                    }
                }
            }
        }

        scrollable(container(content).padding(8).width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Get connection status indicator
    fn get_connection_indicator(
        &self,
        state: &ConnectionState,
    ) -> Element<'static, ServerTreeMessage> {
        let (symbol, color) = match state {
            ConnectionState::Disconnected => ("●", Color::from_rgb(0.6, 0.6, 0.6)),
            ConnectionState::Connecting => ("●", Color::from_rgb(1.0, 0.8, 0.0)),
            ConnectionState::Connected => ("●", Color::from_rgb(0.0, 0.8, 0.0)),
            ConnectionState::Authenticating => ("●", Color::from_rgb(0.0, 0.6, 1.0)),
            ConnectionState::Registered => ("●", Color::from_rgb(0.0, 0.8, 0.0)),
            ConnectionState::Reconnecting => ("●", Color::from_rgb(1.0, 0.6, 0.0)),
            ConnectionState::Failed(_) => ("●", Color::from_rgb(1.0, 0.0, 0.0)),
        };

        text(symbol).size(12).color(color).into()
    }

    /// Expand a server in the tree view
    pub fn expand_server(&mut self, server_id: String) {
        self.expanded_servers.insert(server_id);
    }

    /// Collapse a server in the tree view
    pub fn collapse_server(&mut self, server_id: String) {
        self.expanded_servers.remove(&server_id);
    }

    /// Check if a server is expanded
    pub fn is_server_expanded(&self, server_id: &str) -> bool {
        self.expanded_servers.contains(server_id)
    }
}

impl Default for ServerTree {
    fn default() -> Self {
        Self::new()
    }
}
