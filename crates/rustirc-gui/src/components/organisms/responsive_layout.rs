use iced::{
    widget::{column, container, row, pane_grid, scrollable, horizontal_space},
    Element, Length, Size, Background, Border, Color, Theme, Renderer,
    alignment::{Horizontal, Vertical},
};
use pane_grid::{Axis, Configuration, Pane, PaneGrid, Split, State};
use std::collections::HashMap;

use crate::themes::material_design_3::{MaterialTheme, ElevationLevel};
use crate::components::organisms::{
    sidebar::{ModernSidebar, SidebarMessage},
    message_bubble::{MessageBubble, ChatMessage},
    rich_text_editor::{RichTextEditor, RichTextMessage},
};

// Responsive breakpoints based on Material Design
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    Compact,    // 0-599dp (mobile portrait)
    Medium,     // 600-839dp (tablet portrait, mobile landscape)
    Expanded,   // 840-1199dp (tablet landscape, small desktop)
    Large,      // 1200-1599dp (desktop)
    ExtraLarge, // 1600dp+ (large desktop)
}

// Layout configurations for different screen sizes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    SinglePane,      // Mobile: only content
    SidebarOverlay,  // Mobile landscape: sidebar as overlay
    TwoPane,         // Tablet: sidebar + content
    ThreePane,       // Desktop: sidebar + content + details
    FourPane,        // Large desktop: sidebar + channels + content + details
}

// Golden ratio proportions for layout
const GOLDEN_RATIO: f32 = 1.618;
const SIDEBAR_MIN_WIDTH: f32 = 280.0;
const SIDEBAR_COMPACT_WIDTH: f32 = 72.0;
const CONTENT_MIN_WIDTH: f32 = 400.0;
const DETAILS_MIN_WIDTH: f32 = 320.0;

#[derive(Debug)]
pub struct ResponsiveLayout {
    pane_state: State<PaneContent>,
    current_breakpoint: Breakpoint,
    layout_mode: LayoutMode,
    window_size: Size,
    sidebar: ModernSidebar,
    theme: MaterialTheme,
    sidebar_collapsed: bool,
    details_collapsed: bool,
    show_overlay: bool,
    adaptive_spacing: bool,
    golden_ratio_layout: bool,
}

#[derive(Debug, Clone)]
pub enum PaneContent {
    Sidebar,
    ChannelList,
    ChatArea,
    UserDetails,
    Settings,
}

#[derive(Debug, Clone)]
pub enum LayoutMessage {
    Sidebar(SidebarMessage),
    RichTextEditor(RichTextMessage),
    WindowResized(Size),
    PaneResized(pane_grid::ResizeEvent),
    ToggleSidebar,
    ToggleDetails,
    ToggleOverlay,
    SwitchLayout(LayoutMode),
    AdaptToBreakpoint(Breakpoint),
}

impl ResponsiveLayout {
    pub fn new(theme: MaterialTheme) -> Self {
        let sidebar = ModernSidebar::new(theme.clone());
        
        // Initialize with default configuration
        let (pane_state, _) = State::new(PaneContent::ChatArea);
        
        Self {
            pane_state,
            current_breakpoint: Breakpoint::Large,
            layout_mode: LayoutMode::ThreePane,
            window_size: Size::new(1200.0, 800.0),
            sidebar,
            theme,
            sidebar_collapsed: false,
            details_collapsed: false,
            show_overlay: false,
            adaptive_spacing: true,
            golden_ratio_layout: true,
        }
    }

    pub fn update(&mut self, message: LayoutMessage) {
        match message {
            LayoutMessage::Sidebar(sidebar_msg) => {
                self.sidebar.update(sidebar_msg);
            }
            LayoutMessage::WindowResized(size) => {
                self.window_size = size;
                let new_breakpoint = self.calculate_breakpoint(size);
                if new_breakpoint != self.current_breakpoint {
                    self.current_breakpoint = new_breakpoint;
                    self.adapt_layout_to_breakpoint();
                }
            }
            LayoutMessage::PaneResized(resize_event) => {
                self.pane_state.resize(&resize_event.split, resize_event.ratio);
            }
            LayoutMessage::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
                if self.sidebar_collapsed && self.current_breakpoint <= Breakpoint::Medium {
                    self.show_overlay = false;
                }
            }
            LayoutMessage::ToggleDetails => {
                self.details_collapsed = !self.details_collapsed;
            }
            LayoutMessage::ToggleOverlay => {
                self.show_overlay = !self.show_overlay;
            }
            LayoutMessage::SwitchLayout(mode) => {
                self.layout_mode = mode;
                self.rebuild_pane_layout();
            }
            LayoutMessage::AdaptToBreakpoint(breakpoint) => {
                self.current_breakpoint = breakpoint;
                self.adapt_layout_to_breakpoint();
            }
            _ => {}
        }
    }

    pub fn view(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        match self.current_breakpoint {
            Breakpoint::Compact => self.compact_view(),
            Breakpoint::Medium => self.medium_view(), 
            _ => self.desktop_view(),
        }
    }

    fn compact_view(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        // Mobile portrait: single pane with overlay sidebar
        let main_content = self.create_chat_content();
        
        if self.show_overlay {
            // Show sidebar as overlay
            let sidebar_overlay = container(
                self.sidebar.view().map(LayoutMessage::Sidebar)
            )
            .width(Length::Fixed(SIDEBAR_MIN_WIDTH))
            .height(Length::Fill)
            .style(move |_theme: &Theme| container::Style {
                background: Some(Background::Color(self.theme.color_scheme.surface_container)),
                border: Border {
                    color: self.theme.color_scheme.outline_variant,
                    width: 1.0,
                    radius: [0.0, 12.0, 12.0, 0.0].into(),
                },
                ..Default::default()
            });

            // Overlay layout
            container(
                row![
                    sidebar_overlay,
                    horizontal_space().width(Length::FillPortion(1))
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            main_content
        }
    }

    fn medium_view(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        // Tablet: two pane layout
        let sidebar_width = if self.sidebar_collapsed {
            SIDEBAR_COMPACT_WIDTH
        } else {
            SIDEBAR_MIN_WIDTH
        };

        row![
            self.sidebar.view().map(LayoutMessage::Sidebar),
            self.create_chat_content()
        ]
        .spacing(self.adaptive_spacing())
        .into()
    }

    fn desktop_view(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        // Desktop: use pane grid for flexible layout
        let pane_grid = PaneGrid::new(&self.pane_state, |_pane, content, _is_maximized| {
            self.create_pane_content(content)
        })
        .on_resize(10, LayoutMessage::PaneResized)
        .spacing(self.adaptive_spacing())
        .style(|theme: &Theme| pane_grid::Style {
            hovered_region: pane_grid::Line {
                color: self.theme.color_scheme.primary,
                width: 2.0,
            },
            picked_split: pane_grid::Line {
                color: self.theme.color_scheme.primary,
                width: 2.0,
            },
            hovered_split: pane_grid::Line {
                color: self.theme.color_scheme.primary.scale_alpha(0.5),
                width: 2.0,
            },
            ..Default::default()
        });

        container(pane_grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_pane_content(&self, content: &PaneContent) -> Element<'_, LayoutMessage, Theme, Renderer> {
        match content {
            PaneContent::Sidebar => {
                container(self.sidebar.view().map(LayoutMessage::Sidebar))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            PaneContent::ChannelList => {
                // Simplified channel list for four-pane layout
                self.create_channel_list()
            }
            PaneContent::ChatArea => {
                self.create_chat_content()
            }
            PaneContent::UserDetails => {
                self.create_user_details()
            }
            PaneContent::Settings => {
                self.create_settings_pane()
            }
        }
    }

    fn create_chat_content(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        // Main chat interface
        let chat_messages = self.create_message_list();
        let input_area = self.create_input_area();

        column![
            // Messages area
            container(scrollable(chat_messages))
                .height(Length::FillPortion(8))
                .padding(self.adaptive_padding()),
            
            // Input area
            container(input_area)
                .height(Length::Shrink)
                .padding(self.adaptive_padding())
        ]
        .spacing(self.adaptive_spacing())
        .into()
    }

    fn create_message_list(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        // Demo messages - in real implementation, this would come from state
        let demo_messages = vec![
            ChatMessage {
                id: "1".to_string(),
                user_id: "alice".to_string(),
                username: "Alice".to_string(),
                avatar_url: None,
                content: "Hello everyone! 👋".to_string(),
                timestamp: chrono::Utc::now(),
                message_type: crate::components::molecules::message_bubble::MessageType::Regular,
                reactions: vec![],
                thread_count: 0,
                is_edited: false,
                reply_to: None,
                attachments: vec![],
                user_badges: vec![],
                user_status: crate::components::organisms::sidebar::ConnectionStatus::Connected,
            },
            ChatMessage {
                id: "2".to_string(),
                user_id: "bob".to_string(),
                username: "Bob".to_string(),
                avatar_url: None,
                content: "Hey Alice! How's the new IRC client coming along?".to_string(),
                timestamp: chrono::Utc::now(),
                message_type: crate::components::molecules::message_bubble::MessageType::Regular,
                reactions: vec![],
                thread_count: 0,
                is_edited: false,
                reply_to: Some("1".to_string()),
                attachments: vec![],
                user_badges: vec![],
                user_status: crate::components::organisms::sidebar::ConnectionStatus::Connected,
            },
        ];

        let mut messages_column = column![];
        
        for message in demo_messages {
            let message_bubble = MessageBubble::new(message, self.theme.clone());
            messages_column = messages_column.push(
                container(message_bubble.view())
                    .padding([4, 0])
            );
        }

        messages_column.spacing(self.adaptive_spacing()).into()
    }

    fn create_input_area(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        let editor = RichTextEditor::new(self.theme.clone())
            .placeholder("Type a message...")
            .multiline(false)
            .max_length(Some(512));

        container(editor.view().map(LayoutMessage::RichTextEditor))
            .width(Length::Fill)
            .into()
    }

    fn create_channel_list(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        // Simplified channel list for desktop layouts
        container(
            column![
                container(
                    crate::components::atoms::typography::MaterialText::new("Channels")
                        .variant(crate::components::atoms::typography::TextVariant::HeadlineSmall)
                        .color(self.theme.color_scheme.on_surface)
                )
                .padding(16),
                
                // Demo channels
                container(
                    column![
                        crate::components::atoms::typography::MaterialText::new("# rust")
                            .variant(crate::components::atoms::typography::TextVariant::LabelLarge)
                            .color(self.theme.color_scheme.primary),
                        crate::components::atoms::typography::MaterialText::new("# programming")
                            .variant(crate::components::atoms::typography::TextVariant::LabelLarge)
                            .color(self.theme.color_scheme.on_surface),
                        crate::components::atoms::typography::MaterialText::new("# linux")
                            .variant(crate::components::atoms::typography::TextVariant::LabelLarge)
                            .color(self.theme.color_scheme.on_surface),
                    ]
                    .spacing(8)
                )
                .padding([0, 16])
            ]
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(self.theme.color_scheme.surface_container)),
            border: Border {
                color: self.theme.color_scheme.outline_variant,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn create_user_details(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        container(
            column![
                container(
                    crate::components::atoms::typography::MaterialText::new("User Details")
                        .variant(crate::components::atoms::typography::TextVariant::HeadlineSmall)
                        .color(self.theme.color_scheme.on_surface)
                )
                .padding(16),
                
                // Demo user info
                container(
                    column![
                        crate::components::atoms::typography::MaterialText::new("Alice")
                            .variant(crate::components::atoms::typography::TextVariant::TitleMedium)
                            .color(self.theme.color_scheme.on_surface),
                        crate::components::atoms::typography::MaterialText::new("Online")
                            .variant(crate::components::atoms::typography::TextVariant::LabelMedium)
                            .color(self.theme.color_scheme.primary),
                        crate::components::atoms::typography::MaterialText::new("Rust developer")
                            .variant(crate::components::atoms::typography::TextVariant::BodyMedium)
                            .color(self.theme.color_scheme.on_surface_variant),
                    ]
                    .spacing(4)
                )
                .padding([0, 16])
            ]
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(self.theme.color_scheme.surface_container)),
            border: Border {
                color: self.theme.color_scheme.outline_variant,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn create_settings_pane(&self) -> Element<'_, LayoutMessage, Theme, Renderer> {
        container(
            column![
                container(
                    crate::components::atoms::typography::MaterialText::new("Settings")
                        .variant(crate::components::atoms::typography::TextVariant::HeadlineSmall)
                        .color(self.theme.color_scheme.on_surface)
                )
                .padding(16),
                
                // Demo settings
                container(
                    column![
                        crate::components::atoms::typography::MaterialText::new("Theme")
                            .variant(crate::components::atoms::typography::TextVariant::LabelLarge)
                            .color(self.theme.color_scheme.on_surface),
                        crate::components::atoms::typography::MaterialText::new("Notifications")
                            .variant(crate::components::atoms::typography::TextVariant::LabelLarge)
                            .color(self.theme.color_scheme.on_surface),
                        crate::components::atoms::typography::MaterialText::new("Privacy")
                            .variant(crate::components::atoms::typography::TextVariant::LabelLarge)
                            .color(self.theme.color_scheme.on_surface),
                    ]
                    .spacing(8)
                )
                .padding([0, 16])
            ]
        )
        .style(move |_theme: &Theme| container::Style {
            background: Some(Background::Color(self.theme.color_scheme.surface_container)),
            border: Border {
                color: self.theme.color_scheme.outline_variant,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    // Helper methods
    fn calculate_breakpoint(&self, size: Size) -> Breakpoint {
        let width = size.width;
        match width {
            w if w < 600.0 => Breakpoint::Compact,
            w if w < 840.0 => Breakpoint::Medium,
            w if w < 1200.0 => Breakpoint::Expanded,
            w if w < 1600.0 => Breakpoint::Large,
            _ => Breakpoint::ExtraLarge,
        }
    }

    fn adapt_layout_to_breakpoint(&mut self) {
        self.layout_mode = match self.current_breakpoint {
            Breakpoint::Compact => LayoutMode::SinglePane,
            Breakpoint::Medium => LayoutMode::TwoPane,
            Breakpoint::Expanded => LayoutMode::ThreePane,
            Breakpoint::Large | Breakpoint::ExtraLarge => LayoutMode::ThreePane,
        };

        // Adjust collapsed states based on available space
        match self.current_breakpoint {
            Breakpoint::Compact => {
                self.sidebar_collapsed = true;
                self.details_collapsed = true;
            }
            Breakpoint::Medium => {
                self.details_collapsed = true;
            }
            _ => {
                // Desktop: expand everything if there's space
                if self.window_size.width > 1400.0 {
                    self.sidebar_collapsed = false;
                    self.details_collapsed = false;
                }
            }
        }

        self.rebuild_pane_layout();
    }

    fn rebuild_pane_layout(&mut self) {
        // Rebuild pane grid based on current layout mode
        let (new_state, _) = match self.layout_mode {
            LayoutMode::SinglePane => {
                State::new(PaneContent::ChatArea)
            }
            LayoutMode::TwoPane => {
                let (state, pane) = State::new(PaneContent::Sidebar);
                let (state, _) = state.split(Axis::Vertical, &pane, PaneContent::ChatArea);
                (state, pane)
            }
            LayoutMode::ThreePane => {
                let (state, pane) = State::new(PaneContent::Sidebar);
                let (state, content_pane) = state.split(Axis::Vertical, &pane, PaneContent::ChatArea);
                let (state, _) = state.split(Axis::Vertical, &content_pane, PaneContent::UserDetails);
                (state, pane)
            }
            LayoutMode::FourPane => {
                let (state, pane) = State::new(PaneContent::Sidebar);
                let (state, channel_pane) = state.split(Axis::Vertical, &pane, PaneContent::ChannelList);
                let (state, content_pane) = state.split(Axis::Vertical, &channel_pane, PaneContent::ChatArea);
                let (state, _) = state.split(Axis::Vertical, &content_pane, PaneContent::UserDetails);
                (state, pane)
            }
            _ => State::new(PaneContent::ChatArea),
        };

        self.pane_state = new_state;
    }

    fn adaptive_spacing(&self) -> u16 {
        if self.adaptive_spacing {
            match self.current_breakpoint {
                Breakpoint::Compact => 8,
                Breakpoint::Medium => 12,
                _ => 16,
            }
        } else {
            16
        }
    }

    fn adaptive_padding(&self) -> u16 {
        match self.current_breakpoint {
            Breakpoint::Compact => 8,
            Breakpoint::Medium => 12,
            _ => 16,
        }
    }

    // Calculate proportions using golden ratio
    fn calculate_golden_proportions(&self, total_width: f32) -> (f32, f32, f32) {
        if self.golden_ratio_layout {
            let content_width = total_width / (1.0 + GOLDEN_RATIO);
            let sidebar_width = content_width / GOLDEN_RATIO;
            let details_width = sidebar_width;
            (sidebar_width, content_width, details_width)
        } else {
            // Equal proportions
            let section_width = total_width / 3.0;
            (section_width, section_width, section_width)
        }
    }

    // Public API for configuration
    pub fn set_adaptive_spacing(&mut self, enabled: bool) {
        self.adaptive_spacing = enabled;
    }

    pub fn set_golden_ratio_layout(&mut self, enabled: bool) {
        self.golden_ratio_layout = enabled;
    }

    pub fn get_current_breakpoint(&self) -> Breakpoint {
        self.current_breakpoint
    }

    pub fn get_layout_mode(&self) -> LayoutMode {
        self.layout_mode
    }
}

impl Default for ResponsiveLayout {
    fn default() -> Self {
        Self::new(MaterialTheme::default())
    }
}