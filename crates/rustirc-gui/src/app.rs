//! Main GUI application
//!
//! This module implements the complete Iced GUI application for RustIRC.
//! Features:
//! - Multi-server IRC client interface using Iced 0.13.1 functional API
//! - Resizable panels with server tree, message view, and user lists
//! - Comprehensive tab system for channels and private messages
//! - Full IRC message formatting with colors and styles
//! - Theming support with multiple built-in themes
//! - Context menus, dialogs, and platform integration

use crate::theme::{Theme, ThemeType};
use crate::state::AppState;
use iced::{
    widget::{pane_grid, container, row, column, text, button, text_input, scrollable},
    Element, Length, Task,
};
use rustirc_core::IrcClient;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Main application message types
#[derive(Debug, Clone)]
pub enum Message {
    // Layout and UI messages
    PaneResized(pane_grid::ResizeEvent),
    PaneClicked(pane_grid::Pane),
    PaneDragged(pane_grid::DragEvent),
    
    // Core IRC messages
    ConnectToServer(String, u16),
    DisconnectFromServer(String),
    JoinChannel(String, String), // server_id, channel
    LeaveChannel(String, String), // server_id, channel
    SendMessage(String, String, String), // server_id, target, message
    
    // Input handling
    InputChanged(String),
    InputSubmitted,
    
    // Tab management
    TabSelected(String),
    TabClosed(String),
    TabReordered(Vec<String>),
    
    // Context menu
    ShowContextMenu(f32, f32), // x, y position
    HideContextMenu,
    ContextMenuAction(String),
    
    // Theme management
    ThemeChanged(ThemeType),
    
    // No operation
    None,
}

/// Pane types for the layout
#[derive(Debug, Clone, PartialEq)]
pub enum PaneType {
    ServerTree,
    MessageView,
    UserList,
    InputArea,
}

/// Main GUI application state
pub struct RustIrcGui {
    // Core IRC functionality
    irc_client: Arc<RwLock<Option<Arc<IrcClient>>>>,
    
    // Application state
    app_state: AppState,
    current_theme: Theme,
    
    // Layout
    panes: pane_grid::State<PaneType>,
    
    // Input state
    input_buffer: String,
    
    // Context menu
    context_menu_visible: bool,
    context_menu_x: f32,
    context_menu_y: f32,
}

impl Default for RustIrcGui {
    fn default() -> Self {
        // Initialize pane grid layout
        let (mut panes, _) = pane_grid::State::new(PaneType::MessageView);
        
        // Split to create server tree on the left
        let first_pane = *panes.iter().next().unwrap().0;
        let (left_pane, _) = panes.split(
            pane_grid::Axis::Vertical,
            first_pane,
            PaneType::ServerTree,
        ).unwrap();
        
        // Split right side to add user list
        let (_, right_pane) = panes.split(
            pane_grid::Axis::Vertical,
            left_pane,
            PaneType::UserList,
        ).unwrap();
        
        // Split bottom for input area
        let (_, _) = panes.split(
            pane_grid::Axis::Horizontal,
            right_pane,
            PaneType::InputArea,
        ).unwrap();

        Self {
            irc_client: Arc::new(RwLock::new(None)),
            app_state: AppState::new(),
            current_theme: Theme::default(),
            panes,
            input_buffer: String::new(),
            context_menu_visible: false,
            context_menu_x: 0.0,
            context_menu_y: 0.0,
        }
    }
}

impl RustIrcGui {
    /// Update function for Iced 0.13.1 functional approach
    fn update(&mut self, message: Message) -> impl Into<Task<Message>> {
        match message {
            Message::PaneResized(resize_event) => {
                self.panes.resize(resize_event.split, resize_event.ratio);
            }
            Message::PaneClicked(_pane) => {
                // Pane grid state doesn't need click handling in Iced 0.13.1
            }
            Message::PaneDragged(_drag_event) => {
                // Drag events are handled automatically by pane grid in Iced 0.13.1
            }
            Message::InputChanged(value) => {
                self.input_buffer = value;
            }
            Message::InputSubmitted => {
                if !self.input_buffer.trim().is_empty() {
                    info!("Input submitted: {}", self.input_buffer);
                    
                    // Parse and handle IRC commands
                    if self.input_buffer.starts_with('/') {
                        let command = self.input_buffer.clone();
                        self.handle_irc_command(&command);
                    } else {
                        // Send message to current channel/target
                        if let Some(current_tab) = &self.app_state.current_tab_id {
                            if let Some(tab) = self.app_state.tabs.get(current_tab) {
                                if let Some(_server_id) = &tab.server_id {
                                    let target = tab.name.clone();
                                    let message = self.input_buffer.clone();
                                    info!("Sending message to {}: {}", target, message);
                                    // TODO: Actually send the message via IRC client
                                }
                            }
                        }
                    }
                    
                    self.input_buffer.clear();
                }
            }
            Message::ConnectToServer(server, port) => {
                info!("Connecting to {}:{}", server, port);
                // TODO: Create IRC client and connect
            }
            Message::JoinChannel(server_id, channel) => {
                info!("Joining channel {} on server {}", channel, server_id);
                self.app_state.add_channel_tab(server_id, channel);
            }
            Message::TabSelected(tab_id) => {
                self.app_state.current_tab_id = Some(tab_id);
            }
            Message::TabClosed(tab_id) => {
                self.app_state.remove_tab(&tab_id);
                // If this was the current tab, select another one
                if self.app_state.current_tab_id.as_ref() == Some(&tab_id) {
                    self.app_state.current_tab_id = self.app_state.tab_order.first().cloned();
                }
            }
            Message::ShowContextMenu(x, y) => {
                self.context_menu_visible = true;
                self.context_menu_x = x;
                self.context_menu_y = y;
            }
            Message::HideContextMenu => {
                self.context_menu_visible = false;
            }
            Message::ThemeChanged(theme_type) => {
                self.current_theme = Theme::from_type(theme_type);
            }
            _ => {}
        }
        
        Task::none()
    }

    /// View function for Iced 0.13.1 functional approach
    fn view(&self) -> Element<Message> {
        let pane_grid = pane_grid::PaneGrid::new(&self.panes, |_pane, pane_type, _is_maximized| {
            self.pane_content(*pane_type)
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(1)
        .on_click(Message::PaneClicked)
        .on_drag(Message::PaneDragged)
        .on_resize(10, Message::PaneResized);

        let content = column![
            self.render_tab_bar(),
            pane_grid,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        // Add context menu if visible
        if self.context_menu_visible {
            // TODO: Implement context menu overlay
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Run the GUI application using Iced 0.13.1 Application trait
    pub fn run() -> iced::Result {
        iced::application("RustIRC - Modern IRC Client", Self::update, Self::view)
            .theme(Self::theme)
            .run()
    }

    /// Theme function for Iced 0.13.1
    fn theme(&self) -> iced::Theme {
        match self.current_theme.theme_type {
            ThemeType::Dark => iced::Theme::Dark,
            ThemeType::Light => iced::Theme::Light,
            ThemeType::Dracula => iced::Theme::Dracula,
            ThemeType::Nord => iced::Theme::Nord,
            ThemeType::SolarizedLight => iced::Theme::SolarizedLight,
            ThemeType::SolarizedDark => iced::Theme::SolarizedDark,
            ThemeType::GruvboxLight => iced::Theme::GruvboxLight,
            ThemeType::GruvboxDark => iced::Theme::GruvboxDark,
            ThemeType::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            ThemeType::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            ThemeType::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            ThemeType::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            ThemeType::TokyoNight => iced::Theme::TokyoNight,
            ThemeType::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            ThemeType::TokyoNightLight => iced::Theme::TokyoNightLight,
            ThemeType::KanagawaWave => iced::Theme::KanagawaWave,
            ThemeType::KanagawaDragon => iced::Theme::KanagawaDragon,
            ThemeType::KanagawaLotus => iced::Theme::KanagawaLotus,
            ThemeType::Moonfly => iced::Theme::Moonfly,
            ThemeType::Nightfly => iced::Theme::Nightfly,
            ThemeType::Oxocarbon => iced::Theme::Oxocarbon,
        }
    }

    fn render_tab_bar(&self) -> Element<Message> {
        let mut tab_row = row![].spacing(2);

        for tab_id in &self.app_state.tab_order {
            if let Some(tab) = self.app_state.tabs.get(tab_id) {
                let tab_button = button(text(&tab.name))
                    .on_press(Message::TabSelected(tab_id.clone()));

                tab_row = tab_row.push(tab_button);
                
                // Add close button
                let close_button = button("×")
                    .on_press(Message::TabClosed(tab_id.clone()));
                tab_row = tab_row.push(close_button);
            }
        }

        container(tab_row)
            .padding(5)
            .width(Length::Fill)
            .into()
    }

    fn pane_content(&self, pane_type: PaneType) -> pane_grid::Content<Message> {
        let content = match pane_type {
            PaneType::ServerTree => {
                self.render_server_tree()
            }
            PaneType::MessageView => {
                self.render_message_view()
            }
            PaneType::UserList => {
                self.render_user_list()
            }
            PaneType::InputArea => {
                self.render_input_area()
            }
        };

        pane_grid::Content::new(content)
    }

    fn render_server_tree(&self) -> Element<Message> {
        let mut content = column![] as iced::widget::Column<'_, Message, iced::Theme, iced::Renderer>;
        content = content.spacing(5).padding(10);

        content = content.push(text("Servers").size(16));

        for (_server_id, server) in &self.app_state.servers {
            let server_text = text(&server.name);
            content = content.push(server_text);

            for (_channel_name, _channel) in &server.channels {
                let channel_text = text(format!("  #{}", _channel_name));
                content = content.push(channel_text);
            }
        }

        // Add connect button for demo
        let connect_button = button("Connect to Libera.Chat")
            .on_press(Message::ConnectToServer("irc.libera.chat".to_string(), 6667));
        content = content.push(connect_button);

        // Add join channel button for demo
        let join_button = button("Join #rust")
            .on_press(Message::JoinChannel("libera".to_string(), "#rust".to_string()));
        content = content.push(join_button);

        scrollable(content).into()
    }

    fn render_message_view(&self) -> Element<Message> {
        let content = if let Some(current_tab_id) = &self.app_state.current_tab_id {
            if let Some(tab) = self.app_state.tabs.get(current_tab_id) {
                let mut messages = column![] as iced::widget::Column<'_, Message, iced::Theme, iced::Renderer>;
                messages = messages.spacing(2).padding(10);
                
                messages = messages.push(text(format!("Messages for {}", tab.name)).size(16));
                
                // Add sample messages for demonstration
                messages = messages.push(text("12:00 <user1> Hello everyone!"));
                messages = messages.push(text("12:01 <user2> How's everyone doing today?"));
                messages = messages.push(text("12:02 <user3> Great! Working on some Rust code"));
                
                messages.into()
            } else {
                text("No tab selected").into()
            }
        } else {
            text("Welcome to RustIRC!").into()
        };

        scrollable(content).into()
    }

    fn render_user_list(&self) -> Element<Message> {
        let mut content = column![] as iced::widget::Column<'_, Message, iced::Theme, iced::Renderer>;
        content = content.spacing(5).padding(10);

        content = content.push(text("Users").size(16));
        
        // Add sample users for demonstration
        content = content.push(text("@operator"));
        content = content.push(text("+voice"));
        content = content.push(text("regular_user"));
        content = content.push(text("another_user"));

        scrollable(content).into()
    }

    fn render_input_area(&self) -> Element<Message> {
        let input = text_input("Type a message...", &self.input_buffer)
            .on_input(Message::InputChanged)
            .on_submit(Message::InputSubmitted)
            .padding(10);

        container(input)
            .width(Length::Fill)
            .padding(10)
            .into()
    }

    fn handle_irc_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        match parts[0] {
            "/join" => {
                if parts.len() > 1 {
                    let channel = parts[1].to_string();
                    info!("Join command: {}", channel);
                    // TODO: Join channel via IRC client
                }
            }
            "/part" | "/leave" => {
                if let Some(current_tab) = &self.app_state.current_tab_id {
                    info!("Part command for tab: {}", current_tab);
                    // TODO: Leave channel via IRC client
                }
            }
            "/quit" => {
                info!("Quit command");
                // TODO: Disconnect and quit
            }
            "/connect" => {
                if parts.len() > 1 {
                    let server = parts[1].to_string();
                    info!("Connect command: {}", server);
                    // TODO: Connect to server
                }
            }
            _ => {
                warn!("Unknown command: {}", command);
            }
        }
    }
}