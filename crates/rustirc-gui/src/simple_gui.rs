//! Simplified GUI implementation for Iced 0.13.1 compatibility
//!
//! This provides a basic working GUI while the complex widgets are updated.

use crate::theme::{Theme, ThemeType};
use crate::state::AppState;
use iced::{
    widget::{container, column, text, button, text_input, scrollable, row},
    Element, Length, Task,
};
use rustirc_core::IrcClient;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Simplified GUI application state
pub struct SimpleRustIrcGui {
    // Core IRC functionality
    irc_client: Arc<RwLock<Option<Arc<IrcClient>>>>,
    
    // Application state
    app_state: AppState,
    current_theme: Theme,
    
    // Input state
    input_buffer: String,
}

#[derive(Debug, Clone)]
pub enum SimpleMessage {
    // Input handling
    InputChanged(String),
    InputSubmitted,
    
    // IRC operations
    ConnectToServer,
    JoinChannel(String),
    
    // Theme management
    ThemeChanged(ThemeType),
    
    // No operation
    None,
}

impl Default for SimpleRustIrcGui {
    fn default() -> Self {
        Self {
            irc_client: Arc::new(RwLock::new(None)),
            app_state: AppState::new(),
            current_theme: Theme::default(),
            input_buffer: String::new(),
        }
    }
}

impl SimpleRustIrcGui {
    /// Update function for Iced 0.13.1 functional approach
    fn update(&mut self, message: SimpleMessage) -> impl Into<Task<SimpleMessage>> {
        match message {
            SimpleMessage::InputChanged(value) => {
                self.input_buffer = value;
            }
            SimpleMessage::InputSubmitted => {
                if !self.input_buffer.trim().is_empty() {
                    info!("Input submitted: {}", self.input_buffer);
                    self.input_buffer.clear();
                }
            }
            SimpleMessage::ConnectToServer => {
                info!("Connecting to server...");
                self.app_state.add_server("libera".to_string(), "Libera.Chat".to_string());
                
                // Actually connect using IRC client
                let irc_client_clone = self.irc_client.clone();
                tokio::spawn(async move {
                    let config = rustirc_core::Config::default();
                    let client = Arc::new(rustirc_core::IrcClient::new(config));
                    
                    if let Ok(()) = client.connect("irc.libera.chat", 6697).await {
                        let mut client_guard = irc_client_clone.write().await;
                        *client_guard = Some(client);
                        info!("Successfully connected to Libera.Chat");
                    } else {
                        info!("Failed to connect to Libera.Chat");
                    }
                });
            }
            SimpleMessage::JoinChannel(channel) => {
                info!("Joining channel: {}", channel);
                self.app_state.add_channel_tab("libera".to_string(), channel.clone());
                
                // Actually join the channel using IRC client
                let irc_client_clone = self.irc_client.clone();
                let channel_clone = channel.clone();
                tokio::spawn(async move {
                    let client_guard = irc_client_clone.read().await;
                    if let Some(client) = client_guard.as_ref() {
                        if let Ok(()) = client.join_channel(&channel_clone).await {
                            info!("Successfully joined channel: {}", channel_clone);
                        } else {
                            info!("Failed to join channel: {}", channel_clone);
                        }
                    } else {
                        info!("No IRC client available to join channel: {}", channel_clone);
                    }
                });
            }
            SimpleMessage::ThemeChanged(theme_type) => {
                self.current_theme = Theme::from_type(theme_type);
            }
            SimpleMessage::None => {}
        }
        
        Task::none()
    }

    /// View function for Iced 0.13.1 functional approach
    fn view(&self) -> Element<SimpleMessage> {
        let content = column![
            self.render_header(),
            self.render_main_area(),
            self.render_input_area(),
        ]
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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

    /// Run the simplified GUI application using Iced 0.13.1 functional API
    pub fn run() -> iced::Result {
        iced::application("RustIRC - Modern IRC Client", Self::update, Self::view)
            .theme(Self::theme)
            .run()
    }

    fn render_header(&self) -> Element<SimpleMessage> {
        let title = text("RustIRC - Modern IRC Client").size(24);
        
        let connect_button = button("Connect to Libera.Chat")
            .on_press(SimpleMessage::ConnectToServer);
            
        let join_button = button("Join #rust")
            .on_press(SimpleMessage::JoinChannel("#rust".to_string()));

        let controls = row![connect_button, join_button].spacing(10);

        column![title, controls]
            .spacing(10)
            .into()
    }

    fn render_main_area(&self) -> Element<SimpleMessage> {
        let content = if self.app_state.servers.is_empty() {
            column![
                text("Welcome to RustIRC!").size(18),
                text("Click 'Connect to Libera.Chat' to get started."),
            ]
            .spacing(10)
        } else {
            let mut messages = column![
                text("Connected to Libera.Chat").size(16),
                text("Messages will appear here..."),
            ]
            .spacing(5);

            // Show current tab if available
            if let Some(current_tab_id) = &self.app_state.current_tab_id {
                if let Some(tab) = self.app_state.tabs.get(current_tab_id) {
                    messages = messages.push(text(format!("Current tab: {}", tab.name)).size(14));
                }
            }

            // Show servers
            for (server_id, server) in &self.app_state.servers {
                messages = messages.push(text(format!("Server: {} ({})", server.name, server_id)));
                
                for (channel_name, _) in &server.channels {
                    messages = messages.push(text(format!("  Channel: {}", channel_name)));
                }
            }

            messages.spacing(5)
        };

        scrollable(content)
            .height(Length::Fill)
            .into()
    }

    fn render_input_area(&self) -> Element<SimpleMessage> {
        let input = text_input("Type a message or IRC command...", &self.input_buffer)
            .on_input(SimpleMessage::InputChanged)
            .on_submit(SimpleMessage::InputSubmitted)
            .padding(10);

        container(input)
            .width(Length::Fill)
            .padding(5)
            .into()
    }
}