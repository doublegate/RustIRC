# Phase 3: User Interface Implementation

**Duration**: 4-10 weeks  
**Goal**: Create intuitive graphical and terminal user interfaces

## Overview

Phase 3 transforms the CLI prototype into a full-featured client with both GUI and TUI options. The GUI will use Iced for a modern, cross-platform experience, while the TUI will use ratatui for terminal users. Both interfaces will share the same core logic through a common abstraction layer.

## Objectives

1. Design and implement GUI with Iced
2. Create TUI alternative with ratatui
3. Implement shared UI abstractions
4. Add theming and customization
5. Integrate platform-specific features
6. Ensure responsive performance

## GUI Implementation (Iced)

### Application Structure
```rust
// rustirc-gui/src/app.rs
#[derive(Debug)]
pub struct RustIrcApp {
    state: Arc<RwLock<IrcState>>,
    theme: Theme,
    layout: Layout,
    active_view: ViewId,
    command_tx: mpsc::Sender<Command>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Connection events
    ServerConnected(ServerId),
    ServerDisconnected(ServerId),
    
    // UI events
    TabSelected(ViewId),
    InputSubmitted(String),
    UserListClicked(String),
    
    // State updates
    StateUpdated(StateUpdate),
}
```

### Main Window Layout
```
┌─────────────────────────────────────────────────────┐
│  File  Edit  View  Server  Channel  Tools  Help    │ Menu Bar
├─────────────────────────────────────────────────────┤
│ ┌─────────┐ ┌───────────────────────────────────┐  │
│ │ Servers │ │  #channel-1 | #channel-2 | query  │  │ Tab Bar
│ │ └─libera │ └───────────────────────────────────┘  │
│ │   └─#rust│ ┌─────────────────────┬─────────────┐ │
│ │   └─#irc │ │                     │ @op1        │ │
│ │ └─oftc   │ │   Message Area      │ +voice1     │ │
│ │   └─#dev │ │                     │  user1      │ │
│ └─────────┘ │                     │  user2      │ │ Channel View
│             │                     │  ...        │ │
│             └─────────────────────┴─────────────┘ │
│ ┌─────────────────────────────────────────────────┐│
│ │ Input: /join #newchannel                        ││ Input Area
│ └─────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────┤
│ Connected to irc.libera.chat | 12:34 PM           │ Status Bar
└─────────────────────────────────────────────────────┘
```

### Core Widgets

#### Server/Channel Tree
```rust
// rustirc-gui/src/widgets/server_tree.rs
pub struct ServerTree {
    servers: Vec<ServerNode>,
    selected: Option<NodeId>,
    expanded: HashSet<NodeId>,
}

pub struct ServerNode {
    id: ServerId,
    name: String,
    state: ConnectionState,
    channels: Vec<ChannelNode>,
}
```

#### Message Display
```rust
// rustirc-gui/src/widgets/message_view.rs
pub struct MessageView {
    messages: VecDeque<DisplayMessage>,
    scroll_state: ScrollState,
    selection: Option<MessageId>,
}

pub struct DisplayMessage {
    id: MessageId,
    timestamp: DateTime<Local>,
    sender: String,
    content: FormattedText,
    message_type: MessageType,
}
```

#### User List
```rust
// rustirc-gui/src/widgets/user_list.rs
pub struct UserList {
    users: Vec<ChannelUser>,
    sort_mode: SortMode,
    filter: Option<String>,
}

pub struct ChannelUser {
    nick: String,
    modes: UserModes,
    away: bool,
    account: Option<String>,
}
```

### IRC-Specific Features

#### Color Code Rendering
```rust
// rustirc-gui/src/rendering/colors.rs
pub fn parse_irc_colors(text: &str) -> Vec<TextSpan> {
    // Parse mIRC color codes (^C)
    // Parse ANSI color codes
    // Handle bold, italic, underline
    // Return styled text spans
}
```

#### Context Menus
```rust
// rustirc-gui/src/menus/context.rs
pub fn user_context_menu(nick: &str) -> Menu {
    Menu::new()
        .add_item("Query", Message::OpenQuery(nick))
        .add_item("Whois", Message::SendWhois(nick))
        .add_separator()
        .add_item("Op", Message::ChangeMode("+o", nick))
        .add_item("Voice", Message::ChangeMode("+v", nick))
        .add_separator()
        .add_item("Kick", Message::KickUser(nick))
        .add_item("Ban", Message::BanUser(nick))
}
```

### Platform Integration

#### Notifications
```rust
// rustirc-gui/src/platform/notifications.rs
#[cfg(target_os = "windows")]
pub fn show_notification(title: &str, body: &str) {
    // Windows Toast Notifications
}

#[cfg(target_os = "macos")]
pub fn show_notification(title: &str, body: &str) {
    // macOS Notification Center
}

#[cfg(target_os = "linux")]
pub fn show_notification(title: &str, body: &str) {
    // D-Bus notifications
}
```

## TUI Implementation (ratatui)

### Terminal Layout
```
┌─────────────────────────────────────────────────────┐
│[Libera] #rust     [OFTC] #dev     [*] Query: user1 │ Tab Bar
├───────────┬─────────────────────────────────────────┤
│ Servers:  │ 12:01 <alice> Hello everyone!          │
│ ├─ Libera │ 12:02 <bob> Hey alice, how's it going? │
│ │  ├─ #rust│ 12:03 * charlie waves                  │
│ │  └─ #irc│ 12:04 <alice> Pretty good, working on  │
│ └─ OFTC   │       a new Rust project                │ Message Area
│    └─ #dev│ 12:05 <bob> Nice! What kind of project?│
│           │ 12:06 --> dave joined #rust            │
│ Users:    │ 12:07 <alice> An IRC client actually!  │
│ @operator │                                         │
│ +voice    ├─────────────────────────────────────────┤
│  alice    │ dave: Welcome to #rust!                 │ Input Area
│  bob      └─────────────────────────────────────────┤
│  charlie  │ F1:Help F2:Servers Alt+[1-9]:Switch    │ Status Bar
└───────────┴─────────────────────────────────────────┘
```

### Key Bindings
```rust
// rustirc-tui/src/input.rs
pub fn handle_key_event(key: KeyEvent, app: &mut App) -> Result<()> {
    match key.code {
        KeyCode::F(1) => app.show_help(),
        KeyCode::F(2) => app.toggle_server_list(),
        KeyCode::Tab => app.complete_input(),
        KeyCode::PageUp => app.scroll_up(),
        KeyCode::PageDown => app.scroll_down(),
        KeyCode::Alt('1'..='9') => app.switch_to_tab(n),
        // ... more bindings
    }
}
```

## Shared UI Abstractions

### UI Trait
```rust
// rustirc-core/src/ui/mod.rs
pub trait UserInterface: Send + Sync {
    type Message;
    
    fn update(&mut self, event: UiEvent) -> Option<Self::Message>;
    fn handle_state_change(&mut self, change: StateChange);
    fn render(&mut self) -> Result<()>;
}

pub enum UiEvent {
    Input(String),
    TabSwitch(ViewId),
    Scroll(ScrollDirection),
    Resize(u16, u16),
}
```

### View Management
```rust
// rustirc-core/src/ui/views.rs
pub struct ViewManager {
    views: HashMap<ViewId, Box<dyn View>>,
    active: ViewId,
    history: Vec<ViewId>,
}

pub trait View {
    fn id(&self) -> ViewId;
    fn title(&self) -> String;
    fn update(&mut self, message: ViewMessage);
    fn render(&self, area: Rect) -> Element;
}
```

## Theming System

### Theme Definition
```rust
// rustirc-core/src/theme/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub fonts: FontScheme,
    pub spacing: SpacingScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
    
    // IRC-specific colors
    pub nick_colors: Vec<Color>,
    pub channel_color: Color,
    pub private_color: Color,
    pub notice_color: Color,
    pub action_color: Color,
}
```

### Theme Loading
```toml
# themes/dark.toml
name = "Dark"

[colors]
background = "#1e1e1e"
foreground = "#d4d4d4"
primary = "#569cd6"
secondary = "#4ec9b0"

[colors.irc]
nick_colors = [
    "#d16969", "#ce9178", "#dcdcaa", "#4ec9b0",
    "#569cd6", "#c586c0", "#9cdcfe", "#d7ba7d"
]
```

## Performance Optimization

### Virtual Scrolling
```rust
// rustirc-gui/src/widgets/virtual_scroll.rs
pub struct VirtualScroll<T> {
    items: Vec<T>,
    viewport_height: usize,
    scroll_offset: usize,
    item_height: usize,
}

impl<T> VirtualScroll<T> {
    pub fn visible_items(&self) -> &[T] {
        let start = self.scroll_offset;
        let end = (start + self.viewport_height).min(self.items.len());
        &self.items[start..end]
    }
}
```

### Message Batching
```rust
// rustirc-gui/src/rendering/batch.rs
pub struct MessageBatcher {
    pending: Vec<DisplayMessage>,
    last_update: Instant,
    update_interval: Duration,
}

impl MessageBatcher {
    pub fn add_message(&mut self, msg: DisplayMessage) -> bool {
        self.pending.push(msg);
        
        if self.pending.len() > 100 || 
           self.last_update.elapsed() > self.update_interval {
            true // Trigger UI update
        } else {
            false // Batch more messages
        }
    }
}
```

## Accessibility

### Screen Reader Support
```rust
// rustirc-gui/src/accessibility/mod.rs
pub fn announce_message(msg: &DisplayMessage) {
    let announcement = format!(
        "{} says: {}",
        msg.sender,
        msg.content.plain_text()
    );
    
    #[cfg(target_os = "windows")]
    windows::announce(&announcement);
    
    #[cfg(target_os = "macos")]
    macos::announce(&announcement);
    
    #[cfg(target_os = "linux")]
    atspi::announce(&announcement);
}
```

## Testing

### GUI Testing
```rust
#[cfg(test)]
mod tests {
    use iced_test::*;
    
    #[test]
    fn test_message_input() {
        let mut app = TestApp::new(RustIrcApp::new());
        
        app.simulate_input("Hello, world!");
        app.simulate_key(Key::Enter);
        
        assert!(app.contains_message("Hello, world!"));
    }
}
```

## Deliverables

By the end of Phase 3:

1. **Fully functional GUI**
   - Server/channel management
   - Message display with formatting
   - User lists and interaction
   - Platform-native features

2. **Complete TUI**
   - Keyboard navigation
   - Efficient rendering
   - Feature parity with GUI

3. **Theming System**
   - Built-in themes
   - Custom theme support
   - Hot-reloading

4. **Performance**
   - Smooth scrolling
   - Responsive input
   - Efficient rendering

## Success Criteria

Phase 3 is complete when:
- [ ] GUI runs on Windows, macOS, and Linux
- [ ] TUI provides full IRC functionality
- [ ] Can handle 100+ channels smoothly
- [ ] Theming system is functional
- [ ] Accessibility features work
- [ ] Performance targets are met

## Next Phase

With the user interfaces complete, Phase 4 will add the powerful scripting and plugin system that will make RustIRC truly extensible.