//! Tab bar widget for RustIRC GUI
//!
//! Manages open tabs (channels, private messages, server tabs) with activity indicators,
//! close buttons, and drag-and-drop reordering.

use crate::state::{AppState, Tab, TabType};
use crate::theme::Theme;
use iced::{
    widget::{button, container, row, scrollable, text, Space},
    Alignment, Background, Color, Element, Length, Task,
};
use tracing::{info, warn};

/// Messages for tab bar interactions
#[derive(Debug, Clone)]
pub enum TabBarMessage {
    SwitchTab(String),
    TabSelected(String),
    CloseTab(String),
    TabClosed(String),
    MoveTab(String, usize),
    NewTab,
    CloseAllTabs,
    CloseOtherTabs(String),
    TabContextMenu(String),
}

/// Tab bar widget state
#[derive(Debug, Clone)]
pub struct TabBar {
    max_tab_width: f32,
    show_close_buttons: bool,
    compact_mode: bool,
    scrollable: bool,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            max_tab_width: 150.0,
            show_close_buttons: true,
            compact_mode: false,
            scrollable: true,
        }
    }

    /// Update the tab bar state
    pub fn update(
        &mut self,
        message: TabBarMessage,
        app_state: &mut AppState,
    ) -> Task<TabBarMessage> {
        match message {
            TabBarMessage::SwitchTab(tab_id) => {
                app_state.switch_to_tab(&tab_id);

                // Mark tab as read when switched to
                if let Some(tab) = app_state.current_tab_mut() {
                    tab.mark_as_read();
                }

                Task::none()
            }
            TabBarMessage::TabSelected(tab_id) => {
                // Handle tab selection (same as switch for now)
                app_state.switch_to_tab(&tab_id);

                // Mark tab as read when selected
                if let Some(tab) = app_state.current_tab_mut() {
                    tab.mark_as_read();
                }

                Task::none()
            }
            TabBarMessage::CloseTab(tab_id) => {
                app_state.close_tab(&tab_id);
                Task::none()
            }
            TabBarMessage::TabClosed(tab_id) => {
                // Handle tab closed event (same as close for now)
                app_state.close_tab(&tab_id);
                Task::none()
            }
            TabBarMessage::MoveTab(tab_id, new_position) => {
                // Implement tab reordering with validation
                if let Some(current_pos) = app_state.tab_order.iter().position(|id| id == &tab_id) {
                    if new_position >= app_state.tab_order.len() {
                        warn!(
                            "Invalid tab move position {} for tab {}, clamping to valid range",
                            new_position, tab_id
                        );
                    }
                    let tab_id = app_state.tab_order.remove(current_pos);
                    let insert_pos = new_position.min(app_state.tab_order.len());
                    info!("Moved tab {} to position {}", tab_id, new_position);
                    app_state.tab_order.insert(insert_pos, tab_id);
                } else {
                    warn!("Attempted to move non-existent tab: {}", tab_id);
                }
                Task::none()
            }
            TabBarMessage::NewTab => {
                // Create new server tab (simplified implementation)
                let server_id = format!("new_server_{}", app_state.servers.len() + 1);
                app_state.add_server(server_id.clone(), "New Server".to_string());
                app_state.current_tab_id = Some(server_id.clone());
                info!("Created new server tab: {}", server_id);
                Task::none()
            }
            TabBarMessage::CloseAllTabs => {
                // Close all tabs except server tabs
                let tabs_to_close: Vec<String> = app_state
                    .tabs
                    .iter()
                    .filter(|(_, tab)| !matches!(tab.tab_type, TabType::Server))
                    .map(|(id, _)| id.clone())
                    .collect();

                for tab_id in tabs_to_close {
                    app_state.close_tab(&tab_id);
                }

                Task::none()
            }
            TabBarMessage::CloseOtherTabs(keep_tab_id) => {
                // Close all tabs except the specified one and server tabs
                let tabs_to_close: Vec<String> = app_state
                    .tabs
                    .iter()
                    .filter(|(id, tab)| {
                        *id != &keep_tab_id && !matches!(tab.tab_type, TabType::Server)
                    })
                    .map(|(id, _)| id.clone())
                    .collect();

                for tab_id in tabs_to_close {
                    app_state.close_tab(&tab_id);
                }

                Task::none()
            }
            TabBarMessage::TabContextMenu(tab_id) => {
                // Show tab context menu with tab-specific actions
                info!("Processing context menu for tab: {}", tab_id);

                // Get tab context for action validation
                if let Some(tab) = app_state.tabs.get(&tab_id) {
                    match &tab.tab_type {
                        TabType::Channel { channel } => {
                            info!(
                                "Context menu for channel {}: close, leave, notifications",
                                channel
                            );
                            // Channel tabs support: close tab, leave channel, toggle notifications
                        }
                        TabType::PrivateMessage { nick } => {
                            info!("Context menu for PM with {}: close, clear history", nick);
                            // PM tabs support: close tab, clear history
                        }
                        TabType::Server => {
                            info!("Context menu for server tab: disconnect, reconnect");
                            // Server tabs support: disconnect, reconnect (cannot close)
                        }
                        TabType::Private => {
                            info!("Context menu for private tab: close");
                            // Generic private tabs support: close only
                        }
                    }

                    // Tab-specific menu actions based on context
                    match tab.tab_type {
                        TabType::Server => {
                            // Server tabs have different actions - cannot be closed
                            info!("Server tab context menu - disconnect/reconnect options");
                        }
                        _ => {
                            // Regular tabs can be closed, moved, etc.
                            info!("Regular tab context menu - close/move options available");
                        }
                    }
                } else {
                    warn!("Context menu requested for non-existent tab: {}", tab_id);
                }

                Task::none()
            }
        }
    }

    /// Render the tab bar
    pub fn view(&self, app_state: &AppState) -> Element<'_, TabBarMessage> {
        // Create theme instance for theming support
        let theme = Theme::default();
        let tabs = &app_state.tabs;
        let tab_order = &app_state.tab_order;
        let current_tab_id = app_state.current_tab().and_then(|tab| {
            // Need to find the tab ID from the tab order
            tab_order
                .iter()
                .find(|id| {
                    app_state
                        .tabs
                        .get(*id)
                        .map(|t| std::ptr::eq(t, tab))
                        .unwrap_or(false)
                })
                .cloned()
        });

        if tabs.is_empty() {
            return container(
                text("No tabs open")
                    .size(12.0)
                    .color(Color::from_rgb(0.6, 0.6, 0.6)),
            )
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fixed(40.0))
            .into();
        }

        let mut tab_row = row![];

        for tab_id in tab_order {
            if let Some(tab) = tabs.get(tab_id) {
                let tab_element = self.render_tab(
                    tab_id,
                    tab,
                    current_tab_id.as_ref() == Some(tab_id),
                    app_state,
                );
                tab_row = tab_row.push(tab_element);
            }
        }

        // Add new tab button
        let new_tab_button = button(
            text("+")
                .size(16.0)
                .color(theme.get_text_color())
                .align_x(Alignment::Center),
        )
        .on_press(TabBarMessage::NewTab)
        .width(Length::Fixed(30.0))
        .height(Length::Fixed(30.0))
        .padding(0);

        tab_row = tab_row.push(Space::with_width(Length::Fixed(4.0)));
        tab_row = tab_row.push(new_tab_button);
        // Remove Length::Fill space to prevent horizontal scrollable issue
        tab_row = tab_row.push(Space::with_width(Length::Fixed(10.0)));

        let content: Element<TabBarMessage> = if self.scrollable {
            scrollable(tab_row)
                .direction(scrollable::Direction::Horizontal(Default::default()))
                .width(Length::Shrink) // Fix: Use Shrink instead of Fill for horizontal scrollable
                .height(Length::Fixed(40.0))
                .into()
        } else {
            container(tab_row).height(Length::Fixed(40.0)).into()
        };

        container(content).width(Length::Fill).into()
    }

    /// Render a single tab
    fn render_tab(
        &self,
        tab_id: &str,
        tab: &Tab,
        is_active: bool,
        app_state: &AppState,
    ) -> Element<'_, TabBarMessage> {
        // Get tab icon and title
        let (icon, title) = self.get_tab_display(tab);

        // Activity indicators
        let activity_indicator = if tab.has_highlight {
            Some(Color::from_rgb(1.0, 0.2, 0.2)) // Red for highlights
        } else if tab.has_activity {
            Some(Color::from_rgb(0.2, 0.6, 1.0)) // Blue for activity
        } else {
            None
        };

        // Get themed colors based on app settings
        let theme_name = &app_state.settings().theme;
        let (background_color, themed_text_color) =
            self.get_themed_colors(theme_name, is_active, activity_indicator.is_some());

        // Use themed text color
        let text_color = themed_text_color;

        // Build tab content
        let mut tab_content = row![];

        // Activity indicator with visual feedback
        if let Some(indicator_color) = activity_indicator {
            // Create activity indicator with proper color
            let indicator = container(Space::with_width(Length::Fixed(4.0)))
                .width(Length::Fixed(4.0))
                .height(Length::Fixed(20.0))
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(indicator_color)),
                    border: iced::Border {
                        radius: iced::border::Radius::from(2.0),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                });

            tab_content = tab_content.push(indicator);
            tab_content = tab_content.push(Space::with_width(Length::Fixed(4.0)));
        }

        // Tab icon (if not compact)
        if !self.compact_mode && !icon.is_empty() {
            tab_content = tab_content.push(text(icon).size(12.0).color(text_color));
            tab_content = tab_content.push(Space::with_width(Length::Fixed(4.0)));
        }

        // Tab title
        let title_text = if title.len() > 15 && self.compact_mode {
            format!("{}...", &title[..12])
        } else if title.len() > 20 {
            format!("{}...", &title[..17])
        } else {
            title
        };

        tab_content = tab_content.push(
            text(title_text)
                .size(if self.compact_mode { 11.0 } else { 12.0 })
                .color(text_color),
        );

        // Close button
        if self.show_close_buttons && !matches!(tab.tab_type, TabType::Server) {
            tab_content = tab_content.push(Space::with_width(Length::Fixed(4.0)));
            tab_content = tab_content.push(
                button(text("×").size(14.0).color(Color::from_rgb(0.8, 0.8, 0.8)))
                    .on_press(TabBarMessage::CloseTab(tab_id.to_string()))
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .padding(0),
            );
        }

        // Wrap in clickable button
        let tab_button = button(tab_content)
            .on_press(TabBarMessage::SwitchTab(tab_id.to_string()))
            .width(Length::Fixed(if self.compact_mode {
                self.max_tab_width * 0.7
            } else {
                self.max_tab_width
            }))
            .height(Length::Fixed(if self.compact_mode { 28.0 } else { 32.0 }))
            .padding([4, 8]);

        container(tab_button)
            .style(move |_| container::Style {
                background: Some(Background::Color(background_color)),
                ..container::Style::default()
            })
            .into()
    }

    /// Get themed colors based on app theme
    fn get_themed_colors(
        &self,
        theme_name: &str,
        is_active: bool,
        has_activity: bool,
    ) -> (Color, Color) {
        let (bg_color, text_color) = match theme_name {
            "Dark" => {
                if is_active {
                    (Color::from_rgb(0.3, 0.3, 0.4), Color::WHITE)
                } else if has_activity {
                    (
                        Color::from_rgb(0.2, 0.2, 0.3),
                        Color::from_rgb(0.9, 0.9, 0.9),
                    )
                } else {
                    (Color::TRANSPARENT, Color::from_rgb(0.7, 0.7, 0.7))
                }
            }
            "Light" => {
                if is_active {
                    (Color::from_rgb(0.7, 0.7, 0.8), Color::BLACK)
                } else if has_activity {
                    (
                        Color::from_rgb(0.8, 0.8, 0.9),
                        Color::from_rgb(0.1, 0.1, 0.1),
                    )
                } else {
                    (Color::TRANSPARENT, Color::from_rgb(0.3, 0.3, 0.3))
                }
            }
            _ => {
                // Default theme
                if is_active {
                    (Color::from_rgb(0.3, 0.3, 0.4), Color::WHITE)
                } else {
                    (Color::TRANSPARENT, Color::from_rgb(0.7, 0.7, 0.7))
                }
            }
        };
        (bg_color, text_color)
    }

    /// Get display icon and title for a tab
    fn get_tab_display(&self, tab: &Tab) -> (String, String) {
        match &tab.tab_type {
            TabType::Server => {
                let title = tab
                    .server_id
                    .as_ref()
                    .unwrap_or(&"Server".to_string())
                    .clone();
                ("🖥".to_string(), title)
            }
            TabType::Channel { channel } => ("#".to_string(), channel.clone()),
            TabType::PrivateMessage { nick } => ("@".to_string(), nick.clone()),
            TabType::Private => ("@".to_string(), tab.name.clone()),
        }
    }

    /// Set maximum tab width
    pub fn set_max_tab_width(&mut self, width: f32) {
        self.max_tab_width = width;
    }

    /// Toggle close buttons
    pub fn toggle_close_buttons(&mut self) {
        self.show_close_buttons = !self.show_close_buttons;
    }

    /// Toggle compact mode
    pub fn toggle_compact_mode(&mut self) {
        self.compact_mode = !self.compact_mode;
    }

    /// Set scrollable behavior
    pub fn set_scrollable(&mut self, scrollable: bool) {
        self.scrollable = scrollable;
    }

    /// Get active tab count
    pub fn tab_count(&self, app_state: &AppState) -> usize {
        app_state.tabs.len()
    }

    /// Get activity count (tabs with activity or highlights)
    pub fn activity_count(&self, app_state: &AppState) -> usize {
        app_state
            .tabs
            .values()
            .filter(|tab| tab.has_activity || tab.has_highlight)
            .count()
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}
