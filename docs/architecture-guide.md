# RustIRC Architecture Guide - Dioxus v0.6 Branch

## Overview

RustIRC employs a modular, event-driven, message-passing architecture designed for maintainability, testability, and extensibility. This branch explores migrating from Iced's Elm-style architecture to Dioxus's React-like component patterns while preserving the robust IRC engine and providing superior concurrency through Rust's async ecosystem.

## High-Level Architecture - Dioxus Implementation

```
┌─────────────────────────────────────────────────────────────────┐
│                         Presentation Layer                       │
│  ┌─────────────────────┐         ┌───────────────────────────┐ │
│  │  GUI (Dioxus v0.6)  │ ← → │       TUI (ratatui)       │ │
│  │  React Components   │         │       (unchanged)        │ │
│  │  Virtual DOM        │         │                           │ │
│  │  WebView/Native     │         │                           │ │
│  └─────────────────────┘         └───────────────────────────┘ │
└─────────────┬─────────────────────────────────┬─────────────────┘
              │ Hook Events (useState/Effect)   │ Signal Updates
              ↓                                 ↑
┌─────────────┴─────────────────────────────────┴─────────────────┐
│                  Dioxus State Management Layer                   │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              Hooks & Signals (use_context/signal)          ││
│  └─────────────────────────────────────────────────────────────┘│
│      ↑          ↓           ↑          ↓           ↑          ↓  │
│  ┌───────────────┐  ┌────────────────┐  ┌────────────────────┐ │
│  │Signal Manager │  │useCoroutine    │  │  Custom Hooks      │ │
│  │(Global State) │  │(Async Tasks)   │  │  (IRC Operations)  │ │
│  └───────────────┘  └────────────────┘  └────────────────────┘ │
└─────────────┬─────────────────────────────────┬─────────────────┘
              │ IRC Engine Commands             │ State Updates
              ↓                                 ↑
┌─────────────┴─────────────────────────────────┴─────────────────┐
│                      Core IRC Engine (Unchanged)                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              Event Bus / Message Dispatcher                 ││
│  └─────────────────────────────────────────────────────────────┘│
│      ↑          ↓           ↑          ↓           ↑          ↓  │
│  ┌───────────────┐  ┌────────────────┐  ┌────────────────────┐ │
│  │State Manager  │  │Command Handler │  │  Plugin/Script     │ │
│  │(Servers,Chans)│  │  (/commands)   │  │      Host          │ │
│  └───────────────┘  └────────────────┘  └────────────────────┘ │
└─────────────┬─────────────────────────────────┬─────────────────┘
              │ Network Commands                │ Plugin Actions
              ↓                                 ↑
┌─────────────┴─────────────────────────────────┴─────────────────┐
│                         Network Layer                            │
│         (Async I/O via Tokio, TLS via rustls, DCC Manager)      │
└─────────────┬─────────────────────────────────┬─────────────────┘
              │                                 ↑
              ↓                                 │
┌─────────────┴─────────────────────────────────┴─────────────────┐
│                          IRC Networks                            │
└──────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Network Layer

The Network Layer handles all external communication with IRC servers and direct client connections.

#### Key Responsibilities:
- Managing TCP connections to IRC servers
- TLS encryption/decryption
- IRC message parsing and serialization
- DCC connection management
- Connection state tracking

#### Implementation Details:
```rust
// Conceptual structure
pub struct NetworkLayer {
    connections: HashMap<ServerId, ServerConnection>,
    dcc_manager: DccManager,
    runtime: tokio::runtime::Runtime,
}

pub struct ServerConnection {
    stream: TlsStream<TcpStream>,
    parser: IrcParser,
    write_queue: mpsc::Sender<IrcMessage>,
    read_task: JoinHandle<()>,
}
```

#### Design Decisions:
- Each server connection runs in its own Tokio task
- Zero-copy parsing where possible for performance
- Backpressure handling for write queues
- Automatic reconnection with exponential backoff

### 2. Core Logic Layer

The Core Logic Layer is the brain of the application, coordinating between network events and user interface updates.

#### Event Bus
Central message dispatcher using Tokio's broadcast channels:
```rust
pub enum Event {
    // Network events
    MessageReceived { server: ServerId, message: IrcMessage },
    ConnectionStateChanged { server: ServerId, state: ConnectionState },
    
    // UI events
    UserCommand { command: Command },
    WindowFocused { window: WindowId },
    
    // Plugin events
    PluginMessage { plugin: PluginId, data: Value },
}
```

#### State Manager
Centralized, thread-safe state management:
```rust
pub struct StateManager {
    servers: Arc<RwLock<HashMap<ServerId, ServerState>>>,
    channels: Arc<RwLock<HashMap<ChannelId, ChannelState>>>,
    users: Arc<RwLock<HashMap<UserId, UserState>>>,
}
```

#### Command Handler
Processes user commands and translates them to network actions:
- Slash command parsing (/join, /msg, etc.)
- Command aliases and custom commands
- Input validation and error handling
- Command history management

### 3. Presentation Layer

The Presentation Layer provides user interfaces while maintaining minimal business logic.

#### GUI (Iced)
- Reactive, Elm-like architecture
- Custom widgets for IRC-specific needs
- Theme system with hot-reloading
- Hardware-accelerated rendering

#### TUI (ratatui)
- Full-featured terminal interface
- Keyboard-driven navigation
- Mouse support where available
- Screen reader compatibility

#### Shared Abstractions
```rust
pub trait UserInterface {
    fn update(&mut self, event: UiEvent) -> Command<Message>;
    fn view(&self) -> Element<Message>;
    fn subscription(&self) -> Subscription<Message>;
}
```

### 4. Plugin System

The plugin system provides extensibility at multiple levels.

#### Lua Scripting Engine
- Sandboxed execution environment
- Event hooks for all IRC events
- API for UI manipulation
- File system access controls

#### Binary Plugin Interface
```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> Version;
    fn on_load(&mut self, host: &mut PluginHost) -> Result<()>;
    fn on_event(&mut self, event: &Event) -> Result<()>;
}
```

## Data Flow

### Incoming Message Flow
1. Network layer receives bytes from socket
2. Parser converts bytes to IrcMessage
3. Message wrapped in Event::MessageReceived
4. Event broadcast to all subscribers
5. State manager updates internal state
6. UI receives state change notification
7. UI re-renders affected components

### Outgoing Command Flow
1. User enters command in UI
2. UI sends Event::UserCommand
3. Command handler validates and processes
4. Network command sent to network layer
5. Network layer serializes and sends
6. Confirmation event broadcast
7. UI updates to reflect sent message

## Concurrency Model

### Task Organization
```
Main Thread
├── GUI Event Loop (Iced)
└── Core Event Dispatcher

Tokio Runtime
├── Network Tasks (1 per server)
│   ├── Read Loop
│   └── Write Loop
├── DCC Tasks (1 per transfer)
├── Plugin Tasks (sandboxed)
└── Background Tasks
    ├── Auto-save
    ├── Log rotation
    └── Update checker
```

### Synchronization
- State access via RwLock for read-heavy workloads
- Message passing preferred over shared state
- Careful ordering to prevent deadlocks
- Async channels for cross-task communication

## Error Handling

### Error Categories
1. **Network Errors**: Connection failures, TLS issues
2. **Protocol Errors**: Malformed messages, invalid commands  
3. **Plugin Errors**: Script exceptions, API misuse
4. **User Errors**: Invalid input, permission denied

### Error Propagation
```rust
pub type Result<T> = std::result::Result<T, RustIrcError>;

pub enum RustIrcError {
    Network(NetworkError),
    Protocol(ProtocolError),
    Plugin(PluginError),
    User(UserError),
}
```

## Performance Considerations

### Optimization Strategies
- Lazy loading of channel backlogs
- Virtual scrolling for large buffers
- Message batching for UI updates
- Connection pooling for DCC
- Efficient string handling (Cow, Arc<str>)

### Memory Management
- Bounded channels to prevent unbounded growth
- Automatic log rotation and cleanup
- Configurable history limits
- Weak references for circular dependencies

## Testing Architecture

### Test Levels
1. **Unit Tests**: Individual component testing
2. **Integration Tests**: Component interaction testing
3. **End-to-End Tests**: Full client behavior testing
4. **Property Tests**: Protocol compliance verification

### Mock Infrastructure
```rust
pub struct MockIrcServer {
    responses: HashMap<String, Vec<String>>,
    connection_behavior: ConnectionBehavior,
}

pub struct MockPlugin {
    event_log: Vec<Event>,
    responses: HashMap<Event, PluginResponse>,
}
```

## Security Architecture

### Threat Model
- Malicious IRC servers
- Compromised scripts/plugins
- Network eavesdropping
- Injection attacks

### Mitigations
- Input sanitization at boundaries
- Capability-based plugin permissions
- TLS certificate validation
- Rate limiting and flood protection
- Sandboxed script execution

## Extensibility Points

### Plugin Hooks
- Pre/post message sending
- Message display filtering
- Custom command handlers
- UI element injection
- Network event interception

### Configuration System
- Layered configuration (defaults → system → user)
- Hot-reloading where possible
- Schema validation
- Migration support

## Future Considerations

### Scalability
- Sharding for very large deployments
- Remote UI possibilities
- Clustering for high availability

### Platform Extensions
- Mobile companion apps
- Web-based interface
- Voice/video via WebRTC

### Protocol Evolution
- Matrix bridge support
- XMPP gateway
- Custom protocol extensions