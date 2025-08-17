//! Shared UI abstractions for RustIRC
//!
//! Provides common interfaces that both GUI and TUI implementations
//! can use for consistent behavior and state management.

use crate::events::Event as CoreEvent;
use rustirc_protocol::Message;
use std::collections::HashMap;
use anyhow::Result;

/// Extract the target (channel or user) from an IRC message
fn extract_message_target(message: &Message) -> String {
    match message.command.as_str() {
        "PRIVMSG" | "NOTICE" => {
            message.params.first().unwrap_or(&"unknown".to_string()).clone()
        }
        "JOIN" => {
            message.params.first().unwrap_or(&"unknown".to_string()).clone()
        }
        "PART" => {
            message.params.first().unwrap_or(&"unknown".to_string()).clone()
        }
        _ => "server".to_string(),
    }
}

/// Common interface for all user interface implementations
pub trait UserInterface: Send + Sync {
    type Message;
    
    /// Update the UI with an event
    fn update(&mut self, event: UiEvent) -> Option<Self::Message>;
    
    /// Handle state changes from the core IRC engine
    fn handle_state_change(&mut self, change: StateChange);
    
    /// Render the interface (implementation specific)
    fn render(&mut self) -> Result<()>;
    
    /// Get current UI state for serialization
    fn get_state(&self) -> UiState;
    
    /// Set UI state from deserialization
    fn set_state(&mut self, state: UiState);
}

/// Convert core IRC events to UI events
pub fn core_event_to_ui_event(core_event: &CoreEvent) -> Option<UiEvent> {
    match core_event {
        CoreEvent::Connected { connection_id } => {
            Some(UiEvent::StateChange(StateChange::ConnectionStateChanged {
                server_id: connection_id.clone(),
                state: crate::connection::ConnectionState::Connected,
            }))
        }
        CoreEvent::Disconnected { connection_id, reason } => {
            Some(UiEvent::StateChange(StateChange::ConnectionStateChanged {
                server_id: connection_id.clone(),
                state: crate::connection::ConnectionState::Failed(reason.clone()),
            }))
        }
        CoreEvent::MessageReceived { connection_id, message } => {
            // Extract message details for UI display
            let message_target = extract_message_target(message);
            Some(UiEvent::StateChange(StateChange::MessageReceived {
                server_id: connection_id.clone(),
                target: message_target,
                message: core_event.clone(),
            }))
        }
        CoreEvent::ChannelJoined { connection_id, channel } => {
            Some(UiEvent::StateChange(StateChange::ChannelJoined {
                server_id: connection_id.clone(),
                channel: channel.clone(),
            }))
        }
        CoreEvent::ChannelLeft { connection_id, channel } => {
            Some(UiEvent::StateChange(StateChange::ChannelLeft {
                server_id: connection_id.clone(),
                channel: channel.clone(),
            }))
        }
        _ => None, // Some core events may not have UI equivalents
    }
}

/// Events that can be sent to any UI implementation
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// User input (text or command)
    Input(String),
    
    /// Tab switching
    TabSwitch(ViewId),
    
    /// Scrolling in message area
    Scroll(ScrollDirection),
    
    /// Window/terminal resize
    Resize(u16, u16),
    
    /// Focus change between UI elements
    FocusChange(FocusTarget),
    
    /// Menu action
    MenuAction(MenuAction),
    
    /// Theme change
    ThemeChange(String),
    
    /// Settings update
    SettingsUpdate(SettingsUpdate),
    
    /// State change from IRC core
    StateChange(StateChange),
}

/// State changes from the IRC core
#[derive(Debug, Clone)]
pub enum StateChange {
    /// Server connection state changed
    ConnectionStateChanged {
        server_id: String,
        state: crate::connection::ConnectionState,
    },
    
    /// Message received
    MessageReceived {
        server_id: String,
        target: String,
        message: crate::events::Event,
    },
    
    /// Channel joined
    ChannelJoined {
        server_id: String,
        channel: String,
    },
    
    /// Channel left
    ChannelLeft {
        server_id: String,
        channel: String,
    },
    
    /// User list updated
    UserListUpdated {
        server_id: String,
        channel: String,
        users: Vec<ChannelUser>,
    },
    
    /// Nick changed
    NickChanged {
        server_id: String,
        old_nick: String,
        new_nick: String,
    },
}

/// View management system
pub struct ViewManager {
    views: HashMap<ViewId, Box<dyn View>>,
    active: ViewId,
    history: Vec<ViewId>,
    view_registry: ViewRegistry,
}

impl ViewManager {
    pub fn new() -> Self {
        Self {
            views: HashMap::new(),
            active: ViewId::default(),
            history: Vec::new(),
            view_registry: ViewRegistry::new(),
        }
    }
    
    /// Register a new view type
    pub fn register_view_type<V: View + 'static>(&mut self, view_type: ViewType, factory: ViewFactory<V>) {
        self.view_registry.register(view_type, factory);
    }
    
    /// Create and add a view
    pub fn create_view(&mut self, view_type: ViewType, id: ViewId) -> Result<()> {
        if let Some(view) = self.view_registry.create_view(view_type, id.clone())? {
            self.views.insert(id, view);
        }
        Ok(())
    }
    
    /// Switch to a view
    pub fn switch_to_view(&mut self, id: ViewId) {
        if self.views.contains_key(&id) {
            self.history.push(self.active.clone());
            self.active = id;
        }
    }
    
    /// Get current view
    pub fn current_view(&self) -> Option<&dyn View> {
        self.views.get(&self.active).map(|v| v.as_ref())
    }
    
    /// Get mutable current view  
    pub fn current_view_mut(&mut self) -> Option<&mut dyn View> {
        if let Some(view) = self.views.get_mut(&self.active) {
            Some(view.as_mut())
        } else {
            None
        }
    }
    
    /// Go back to previous view
    pub fn go_back(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.active = previous;
        }
    }
    
    /// Close a view
    pub fn close_view(&mut self, id: &ViewId) {
        self.views.remove(id);
        if &self.active == id {
            self.go_back();
        }
    }
}

/// View trait for individual UI views (tabs, windows, etc.)
pub trait View: Send + Sync {
    /// Get unique view identifier
    fn id(&self) -> ViewId;
    
    /// Get display title
    fn title(&self) -> String;
    
    /// Update view with a message
    fn update(&mut self, message: ViewMessage);
    
    /// Check if view needs redraw
    fn needs_redraw(&self) -> bool;
    
    /// Mark as redrawn
    fn mark_redrawn(&mut self);
    
    /// Get view metadata
    fn metadata(&self) -> ViewMetadata;
}

/// View registry for managing view types and creation
pub struct ViewRegistry {
    factories: HashMap<ViewType, Box<dyn ViewFactoryTrait>>,
}

impl ViewRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }
    
    pub fn register<V: View + 'static>(&mut self, view_type: ViewType, factory: ViewFactory<V>) {
        self.factories.insert(view_type, Box::new(factory));
    }
    
    pub fn create_view(&self, view_type: ViewType, id: ViewId) -> Result<Option<Box<dyn View>>> {
        if let Some(factory) = self.factories.get(&view_type) {
            Ok(Some(factory.create(id)?))
        } else {
            Ok(None)
        }
    }
}

/// Factory trait for creating views
pub trait ViewFactoryTrait: Send + Sync {
    fn create(&self, id: ViewId) -> Result<Box<dyn View>>;
}

/// Concrete factory for a specific view type
pub struct ViewFactory<V: View + 'static> {
    create_fn: Box<dyn Fn(ViewId) -> Result<V> + Send + Sync>,
}

impl<V: View + 'static> ViewFactory<V> {
    pub fn new<F>(create_fn: F) -> Self 
    where 
        F: Fn(ViewId) -> Result<V> + Send + Sync + 'static 
    {
        Self {
            create_fn: Box::new(create_fn),
        }
    }
}

impl<V: View + 'static> ViewFactoryTrait for ViewFactory<V> {
    fn create(&self, id: ViewId) -> Result<Box<dyn View>> {
        Ok(Box::new((self.create_fn)(id)?))
    }
}

/// UI state for serialization
#[derive(Debug, Clone)]
pub struct UiState {
    pub window_geometry: WindowGeometry,
    pub layout_state: LayoutState,
    pub view_states: HashMap<ViewId, ViewState>,
    pub theme: String,
    pub settings: UiSettings,
}

/// Common types and enums
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewId(pub String);

impl Default for ViewId {
    fn default() -> Self {
        Self("main".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViewType {
    ServerList,
    Channel,
    PrivateMessage,
    ServerConsole,
    Settings,
    Help,
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    StateUpdate(StateChange),
    UserAction(UserAction),
    SystemEvent(SystemEvent),
}

#[derive(Debug, Clone)]
pub struct ViewMetadata {
    pub view_type: ViewType,
    pub title: String,
    pub closable: bool,
    pub has_activity: bool,
    pub has_highlight: bool,
}

#[derive(Debug, Clone)]
pub enum ScrollDirection {
    Up(usize),
    Down(usize),
    PageUp,
    PageDown,
    Home,
    End,
}

#[derive(Debug, Clone)]
pub enum FocusTarget {
    ServerTree,
    MessageArea,
    InputArea,
    UserList,
    StatusBar,
    TabBar,
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    FileNew,
    FileOpen,
    FileQuit,
    EditCopy,
    EditPaste,
    ViewToggleServerTree,
    ViewToggleUserList,
    ServerConnect,
    ServerDisconnect,
    ChannelJoin,
    ChannelPart,
    ToolsPreferences,
    HelpAbout,
}

#[derive(Debug, Clone)]
pub enum SettingsUpdate {
    ThemeChanged(String),
    FontSizeChanged(f32),
    NotificationsChanged(bool),
    SoundChanged(bool),
}

#[derive(Debug, Clone)]
pub enum UserAction {
    SendMessage(String),
    JoinChannel(String),
    LeaveChannel,
    ChangeNick(String),
    SendCommand(String),
}

#[derive(Debug, Clone)]
pub enum SystemEvent {
    WindowFocused,
    WindowUnfocused,
    NetworkChanged,
    PowerStateChanged,
}

#[derive(Debug, Clone)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

#[derive(Debug, Clone)]
pub struct LayoutState {
    pub sidebar_width: f32,
    pub userlist_width: f32,
    pub show_sidebar: bool,
    pub show_userlist: bool,
    pub pane_layout: PaneLayout,
}

#[derive(Debug, Clone)]
pub enum PaneLayout {
    Standard,
    Compact,
    Wide,
    Custom(Vec<PaneConfig>),
}

#[derive(Debug, Clone)]
pub struct PaneConfig {
    pub pane_type: PaneType,
    pub size: PaneSize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum PaneType {
    ServerTree,
    MessageArea,
    UserList,
    StatusBar,
}

#[derive(Debug, Clone)]
pub enum PaneSize {
    Fixed(f32),
    Percentage(f32),
    Flex(f32),
}

#[derive(Debug, Clone)]
pub struct ViewState {
    pub scroll_position: usize,
    pub selection: Option<usize>,
    pub search_query: Option<String>,
    pub view_specific: ViewSpecificState,
}

#[derive(Debug, Clone)]
pub enum ViewSpecificState {
    Channel {
        user_list_sorted_by: UserSortMode,
        topic_expanded: bool,
    },
    PrivateMessage {
        user_info_visible: bool,
    },
    ServerConsole {
        filter_level: LogLevel,
    },
}

#[derive(Debug, Clone)]
pub enum UserSortMode {
    Alphabetical,
    ByMode,
    ByActivity,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct UiSettings {
    pub font_family: String,
    pub font_size: f32,
    pub line_height: f32,
    pub show_timestamps: bool,
    pub timestamp_format: String,
    pub nick_colors: bool,
    pub compact_mode: bool,
    pub animations_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ChannelUser {
    pub nick: String,
    pub modes: Vec<char>,
    pub away: bool,
    pub hostname: Option<String>,
    pub account: Option<String>,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1200,
            height: 800,
            maximized: false,
        }
    }
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_width: 200.0,
            userlist_width: 150.0,
            show_sidebar: true,
            show_userlist: true,
            pane_layout: PaneLayout::Standard,
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            font_family: "monospace".to_string(),
            font_size: 13.0,
            line_height: 1.2,
            show_timestamps: true,
            timestamp_format: "%H:%M:%S".to_string(),
            nick_colors: true,
            compact_mode: false,
            animations_enabled: true,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            window_geometry: WindowGeometry::default(),
            layout_state: LayoutState::default(),
            view_states: HashMap::new(),
            theme: "dark".to_string(),
            settings: UiSettings::default(),
        }
    }
}