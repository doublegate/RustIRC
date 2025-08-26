use iced::{
    alignment::{Horizontal, Vertical},
    event::{self, Event},
    mouse::{self, Cursor},
    theme::palette::Extended,
    time::{Duration, Instant},
    widget::{column, container, mouse_area, row, text, Container},
    Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
};
use std::collections::HashMap;

use crate::components::atoms::button::{ButtonVariant, MaterialButton};
use crate::components::atoms::typography::{MaterialText, TypographyVariant};
use crate::themes::material_design_3::{ElevationLevel, MaterialTheme};

// Server and channel data structures
#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub status: ConnectionStatus,
    pub channels: Vec<ChannelInfo>,
    pub unread_count: u32,
    pub is_expanded: bool,
}

#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    pub channel_type: ChannelType,
    pub unread_count: u32,
    pub mention_count: u32,
    pub activity_level: ActivityLevel,
    pub user_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelType {
    Public,  // #channel
    Private, // &channel
    DirectMessage,
    Server, // Server tab
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityLevel {
    None,
    Low,    // New messages
    Medium, // Multiple messages
    High,   // Mentions or important activity
}

// Sidebar component
#[derive(Debug)]
pub struct ModernSidebar {
    servers: Vec<ServerInfo>,
    selected_server: Option<String>,
    selected_channel: Option<String>,
    // animation_cache: Cache, // TODO: Implement when canvas support is added
    animations: HashMap<String, CollapseAnimation>,
    width: f32,
    is_compact: bool,
    theme: MaterialTheme,
}

#[derive(Debug, Clone)]
struct CollapseAnimation {
    target_height: f32,
    current_height: f32,
    start_time: Instant,
    duration: Duration,
    is_expanding: bool,
}

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    ServerToggled(String),
    ServerSelected(String),
    ChannelSelected(String, String), // server_id, channel_id
    ToggleCompact,
    AnimationTick,
}

impl ModernSidebar {
    pub fn new(theme: MaterialTheme) -> Self {
        Self {
            servers: Vec::new(),
            selected_server: None,
            selected_channel: None,
            // animation_cache: Cache::default(), // TODO: Implement when canvas support is added
            animations: HashMap::new(),
            width: 280.0,
            is_compact: false,
            theme,
        }
    }

    pub fn with_servers(mut self, servers: Vec<ServerInfo>) -> Self {
        self.servers = servers;
        self
    }

    pub fn update(&mut self, message: SidebarMessage) {
        match message {
            SidebarMessage::ServerToggled(server_id) => {
                if let Some(server) = self.servers.iter_mut().find(|s| s.id == server_id) {
                    server.is_expanded = !server.is_expanded;

                    // Calculate target height based on channel count
                    let target_height = if server.is_expanded {
                        (server.channels.len() as f32) * 40.0 // 40px per channel
                    } else {
                        0.0
                    };

                    // Start collapse/expand animation
                    let current_height = self
                        .animations
                        .get(&server_id)
                        .map(|anim| anim.current_height)
                        .unwrap_or(if server.is_expanded {
                            0.0
                        } else {
                            target_height
                        });

                    self.animations.insert(
                        server_id,
                        CollapseAnimation {
                            target_height,
                            current_height,
                            start_time: Instant::now(),
                            duration: Duration::from_millis(300), // Material Design duration
                            is_expanding: server.is_expanded,
                        },
                    );

                    // self.animation_cache.clear(); // TODO: Implement when canvas support is added
                }
            }
            SidebarMessage::ServerSelected(server_id) => {
                self.selected_server = Some(server_id);
                self.selected_channel = None;
            }
            SidebarMessage::ChannelSelected(server_id, channel_id) => {
                self.selected_server = Some(server_id);
                self.selected_channel = Some(channel_id);
            }
            SidebarMessage::ToggleCompact => {
                self.is_compact = !self.is_compact;
                self.width = if self.is_compact { 72.0 } else { 280.0 };
                // self.animation_cache.clear(); // TODO: Implement when canvas support is added
            }
            SidebarMessage::AnimationTick => {
                let now = Instant::now();
                let mut completed_animations = Vec::new();

                for (server_id, animation) in &mut self.animations {
                    let progress = (now - animation.start_time).as_secs_f32()
                        / animation.duration.as_secs_f32();

                    if progress >= 1.0 {
                        animation.current_height = animation.target_height;
                        completed_animations.push(server_id.clone());
                    } else {
                        // Material Design standard easing curve
                        let eased_progress = ease_out_cubic(progress);
                        animation.current_height = animation.current_height
                            + (animation.target_height - animation.current_height) * eased_progress;
                    }
                }

                // Remove completed animations
                for server_id in completed_animations {
                    if let Some(animation) = self.animations.get(&server_id) {
                        if (animation.current_height - animation.target_height).abs() < 0.1 {
                            self.animations.remove(&server_id);
                        }
                    }
                }

                if !self.animations.is_empty() {
                    // self.animation_cache.clear(); // TODO: Implement when canvas support is added
                }
            }
        }
    }

    pub fn view(&self) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let surface_color = self.theme.scheme.surface;
        let surface_variant = self.theme.scheme.surface_variant;

        let content = if self.is_compact {
            self.compact_view()
        } else {
            self.expanded_view()
        };

        container(content)
            .width(Length::Fixed(self.width))
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(surface_color)),
                border: Border {
                    color: surface_variant,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn expanded_view(&self) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let mut content = column![];

        // Header with toggle button
        let header = container(
            row![
                MaterialText::new("Servers")
                    .variant(TypographyVariant::HeadlineSmall)
                    .color(self.theme.scheme.on_surface),
                MaterialButton::new("⇔")
                    .variant(ButtonVariant::Text)
                    .on_press(SidebarMessage::ToggleCompact)
            ]
            .spacing(8),
        )
        .padding(16)
        .width(Length::Fill);

        content = content.push(header);

        // Server list
        for server in &self.servers {
            let server_item = self.create_server_item(server);
            content = content.push(server_item);

            // Channel list with animation
            if server.is_expanded || self.animations.contains_key(&server.id) {
                let channel_height = self
                    .animations
                    .get(&server.id)
                    .map(|anim| anim.current_height)
                    .unwrap_or_else(|| {
                        if server.is_expanded {
                            (server.channels.len() as f32) * 40.0
                        } else {
                            0.0
                        }
                    });

                if channel_height > 0.0 {
                    let channel_list = self.create_channel_list(server, channel_height);
                    content = content.push(channel_list);
                }
            }
        }

        content.spacing(4).into()
    }

    fn compact_view(&self) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let mut content = column![];

        // Compact toggle button
        let header = container(
            MaterialButton::new("☰")
                .variant(ButtonVariant::Text)
                .on_press(SidebarMessage::ToggleCompact),
        )
        .padding(8)
        .width(Length::Fill);

        content = content.push(header);

        // Compact server icons
        for server in &self.servers {
            let server_icon = self.create_compact_server_item(server);
            content = content.push(server_icon);
        }

        content.spacing(8).into()
    }

    fn create_server_item(
        &self,
        server: &ServerInfo,
    ) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let is_selected = self.selected_server.as_ref() == Some(&server.id);

        let status_color = match server.status {
            ConnectionStatus::Connected => self.theme.scheme.primary,
            ConnectionStatus::Connecting => Color::from_rgb(1.0, 0.6, 0.0), // Orange
            ConnectionStatus::Disconnected => self.theme.scheme.outline,
            ConnectionStatus::Error => self.theme.scheme.error,
        };

        let expand_icon = if server.is_expanded { "▼" } else { "▶" };

        let content = row![
            // Expand/collapse button
            MaterialButton::new(expand_icon)
                .variant(ButtonVariant::Text)
                .on_press(SidebarMessage::ServerToggled(server.id.clone())),
            // Status indicator
            container(text("●").size(12).color(status_color)).padding(4),
            // Server name
            MaterialText::new(&server.name)
                .variant(TypographyVariant::LabelLarge)
                .color(if is_selected {
                    self.theme.scheme.on_primary_container
                } else {
                    self.theme.scheme.on_surface
                }),
            // Unread badge
            if server.unread_count > 0 {
                Some(self.create_unread_badge(server.unread_count))
            } else {
                None
            }
        ]
        .spacing(8)
        .align_items(iced::Alignment::Center);

        let background_color = if is_selected {
            self.theme.scheme.primary_container
        } else {
            Color::TRANSPARENT
        };

        mouse_area(
            container(content)
                .padding([8, 12])
                .width(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(background_color)),
                    border: Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .on_press(SidebarMessage::ServerSelected(server.id.clone()))
        .into()
    }

    fn create_channel_list(
        &self,
        server: &ServerInfo,
        height: f32,
    ) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let mut channel_content = column![];

        for channel in &server.channels {
            let channel_item = self.create_channel_item(server, channel);
            channel_content = channel_content.push(channel_item);
        }

        container(channel_content)
            .height(Length::Fixed(height))
            .padding([0, 24]) // Indent channels
            .into()
    }

    fn create_channel_item(
        &self,
        server: &ServerInfo,
        channel: &ChannelInfo,
    ) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let is_selected = self.selected_server.as_ref() == Some(&server.id)
            && self.selected_channel.as_ref() == Some(&channel.id);

        let channel_icon = match channel.channel_type {
            ChannelType::Public => "#",
            ChannelType::Private => "&",
            ChannelType::DirectMessage => "@",
            ChannelType::Server => "🖥",
        };

        let activity_color = match channel.activity_level {
            ActivityLevel::None => self.theme.scheme.outline_variant,
            ActivityLevel::Low => self.theme.scheme.primary,
            ActivityLevel::Medium => Color::from_rgb(1.0, 0.6, 0.0), // Orange
            ActivityLevel::High => self.theme.scheme.error,
        };

        let content = row![
            // Channel type icon
            MaterialText::new(channel_icon)
                .variant(TypographyVariant::LabelMedium)
                .color(activity_color),
            // Channel name
            MaterialText::new(&channel.name)
                .variant(TypographyVariant::LabelMedium)
                .color(if is_selected {
                    self.theme.scheme.on_secondary_container
                } else {
                    self.theme.scheme.on_surface_variant
                }),
            // User count for channels
            if channel.channel_type != ChannelType::DirectMessage && channel.user_count > 0 {
                Some(
                    MaterialText::new(&format!("({})", channel.user_count))
                        .variant(TypographyVariant::LabelSmall)
                        .color(self.theme.scheme.outline),
                )
            } else {
                None
            },
            // Unread indicators
            if channel.mention_count > 0 {
                Some(self.create_mention_badge(channel.mention_count))
            } else if channel.unread_count > 0 {
                Some(self.create_unread_indicator())
            } else {
                None
            }
        ]
        .spacing(6)
        .align_items(iced::Alignment::Center);

        let background_color = if is_selected {
            self.theme.scheme.secondary_container
        } else {
            Color::TRANSPARENT
        };

        mouse_area(
            container(content)
                .padding([6, 8])
                .width(Length::Fill)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(background_color)),
                    border: Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .on_press(SidebarMessage::ChannelSelected(
            server.id.clone(),
            channel.id.clone(),
        ))
        .into()
    }

    fn create_compact_server_item(
        &self,
        server: &ServerInfo,
    ) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let is_selected = self.selected_server.as_ref() == Some(&server.id);

        let status_color = match server.status {
            ConnectionStatus::Connected => self.theme.scheme.primary,
            ConnectionStatus::Connecting => Color::from_rgb(1.0, 0.6, 0.0),
            ConnectionStatus::Disconnected => self.theme.scheme.outline,
            ConnectionStatus::Error => self.theme.scheme.error,
        };

        let server_initial = server
            .name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();

        let content = container(
            MaterialText::new(&server_initial)
                .variant(TypographyVariant::LabelLarge)
                .color(if is_selected {
                    self.theme.scheme.on_primary
                } else {
                    self.theme.scheme.on_surface
                }),
        )
        .padding(8)
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0))
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(if is_selected {
                self.theme.scheme.primary
            } else {
                self.theme.scheme.surface_variant
            })),
            border: Border {
                color: status_color,
                width: 2.0,
                radius: 24.0.into(),
            },
            ..Default::default()
        });

        mouse_area(content)
            .on_press(SidebarMessage::ServerSelected(server.id.clone()))
            .into()
    }

    fn create_unread_badge(&self, count: u32) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let count_text = if count > 99 {
            "99+".to_string()
        } else {
            count.to_string()
        };

        container(
            MaterialText::new(&count_text)
                .variant(TypographyVariant::LabelSmall)
                .color(self.theme.scheme.on_primary),
        )
        .padding([2, 6])
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(self.theme.scheme.primary)),
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn create_mention_badge(&self, count: u32) -> Element<'_, SidebarMessage, Theme, Renderer> {
        let count_text = if count > 9 {
            "9+".to_string()
        } else {
            count.to_string()
        };

        container(
            MaterialText::new(&count_text)
                .variant(TypographyVariant::LabelSmall)
                .color(self.theme.scheme.on_error),
        )
        .padding([2, 6])
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(self.theme.scheme.error)),
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    fn create_unread_indicator(&self) -> Element<'_, SidebarMessage, Theme, Renderer> {
        container(text(""))
            .width(Length::Fixed(8.0))
            .height(Length::Fixed(8.0))
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(self.theme.scheme.primary)),
                border: Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }
}

// Material Design easing function
fn ease_out_cubic(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

// Example usage and demo data
impl Default for ModernSidebar {
    fn default() -> Self {
        let theme = MaterialTheme::default();

        let demo_servers = vec![
            ServerInfo {
                id: "libera".to_string(),
                name: "Libera.Chat".to_string(),
                status: ConnectionStatus::Connected,
                is_expanded: true,
                unread_count: 3,
                channels: vec![
                    ChannelInfo {
                        id: "rust".to_string(),
                        name: "rust".to_string(),
                        channel_type: ChannelType::Public,
                        unread_count: 5,
                        mention_count: 2,
                        activity_level: ActivityLevel::High,
                        user_count: 1247,
                    },
                    ChannelInfo {
                        id: "programming".to_string(),
                        name: "programming".to_string(),
                        channel_type: ChannelType::Public,
                        unread_count: 12,
                        mention_count: 0,
                        activity_level: ActivityLevel::Medium,
                        user_count: 892,
                    },
                    ChannelInfo {
                        id: "linux".to_string(),
                        name: "linux".to_string(),
                        channel_type: ChannelType::Public,
                        unread_count: 0,
                        mention_count: 0,
                        activity_level: ActivityLevel::None,
                        user_count: 543,
                    },
                ],
            },
            ServerInfo {
                id: "oftc".to_string(),
                name: "OFTC".to_string(),
                status: ConnectionStatus::Connecting,
                is_expanded: false,
                unread_count: 0,
                channels: vec![ChannelInfo {
                    id: "debian".to_string(),
                    name: "debian".to_string(),
                    channel_type: ChannelType::Public,
                    unread_count: 0,
                    mention_count: 0,
                    activity_level: ActivityLevel::None,
                    user_count: 324,
                }],
            },
        ];

        Self::new(theme).with_servers(demo_servers)
    }
}
