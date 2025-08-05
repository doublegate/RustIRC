# RustIRC API Reference

## Overview

This document provides a comprehensive reference for the RustIRC public API. It covers the client library interface, event system, state management, and extension points for developers building on top of RustIRC.

## Core Client API

### Client Initialization

```rust
use rustirc::{Client, ClientConfig};

/// Create a new IRC client instance
pub struct Client {
    // Internal state
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Self {
        Client::with_config(ClientConfig::default())
    }
    
    /// Create a new client with custom configuration
    pub fn with_config(config: ClientConfig) -> Self {
        // Implementation
    }
}

/// Client configuration options
pub struct ClientConfig {
    /// Directory for storing client data
    pub data_dir: PathBuf,
    
    /// Maximum number of messages to keep in memory per buffer
    pub buffer_size: usize,
    
    /// Enable automatic reconnection
    pub auto_reconnect: bool,
    
    /// Reconnection delay in seconds
    pub reconnect_delay: u64,
    
    /// Default character encoding
    pub encoding: String,
    
    /// Enable debug logging
    pub debug: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            data_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("rustirc"),
            buffer_size: 5000,
            auto_reconnect: true,
            reconnect_delay: 10,
            encoding: "UTF-8".to_string(),
            debug: false,
        }
    }
}
```

### Server Connection Management

```rust
/// Server connection configuration
pub struct ServerConfig {
    /// Server address (hostname:port)
    pub address: SocketAddr,
    
    /// Use TLS for connection
    pub use_tls: bool,
    
    /// Verify TLS certificates
    pub verify_cert: bool,
    
    /// Primary nickname
    pub nick: String,
    
    /// Alternative nicknames
    pub alt_nicks: Vec<String>,
    
    /// Username (ident)
    pub username: String,
    
    /// Real name
    pub realname: String,
    
    /// Server password (optional)
    pub password: Option<String>,
    
    /// SASL authentication (optional)
    pub sasl: Option<SaslConfig>,
    
    /// Channels to join on connect
    pub autojoin: Vec<String>,
    
    /// Commands to execute on connect
    pub connect_commands: Vec<String>,
}

/// Unique server identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServerId(pub u64);

impl Client {
    /// Connect to an IRC server
    pub async fn connect(&mut self, config: ServerConfig) -> Result<ServerId> {
        // Returns unique server ID for this connection
    }
    
    /// Disconnect from a server
    pub async fn disconnect(&mut self, server_id: ServerId) -> Result<()> {
        // Graceful disconnect with QUIT message
    }
    
    /// Check if connected to a server
    pub fn is_connected(&self, server_id: ServerId) -> bool {
        // Check connection status
    }
    
    /// Get list of connected servers
    pub fn servers(&self) -> Vec<ServerId> {
        // Return all active server IDs
    }
    
    /// Get server information
    pub fn server_info(&self, server_id: ServerId) -> Option<&ServerInfo> {
        // Return server details
    }
}

/// Server information
pub struct ServerInfo {
    pub address: SocketAddr,
    pub network_name: Option<String>,
    pub server_name: String,
    pub motd: Vec<String>,
    pub supported_features: HashMap<String, Option<String>>,
    pub connected_at: DateTime<Utc>,
}
```

### Channel Operations

```rust
impl Client {
    /// Join a channel
    pub async fn join(&mut self, server_id: ServerId, channel: &str) -> Result<()> {
        self.join_with_key(server_id, channel, None).await
    }
    
    /// Join a channel with a key
    pub async fn join_with_key(
        &mut self, 
        server_id: ServerId, 
        channel: &str, 
        key: Option<&str>
    ) -> Result<()> {
        // Join channel implementation
    }
    
    /// Leave a channel
    pub async fn part(&mut self, server_id: ServerId, channel: &str) -> Result<()> {
        self.part_with_message(server_id, channel, None).await
    }
    
    /// Leave a channel with a part message
    pub async fn part_with_message(
        &mut self, 
        server_id: ServerId, 
        channel: &str, 
        message: Option<&str>
    ) -> Result<()> {
        // Part channel implementation
    }
    
    /// Get list of joined channels
    pub fn channels(&self, server_id: ServerId) -> Vec<String> {
        // Return channel list
    }
    
    /// Get channel information
    pub fn channel_info(&self, server_id: ServerId, channel: &str) -> Option<&ChannelInfo> {
        // Return channel details
    }
}

/// Channel information
pub struct ChannelInfo {
    pub name: String,
    pub topic: Option<String>,
    pub topic_set_by: Option<String>,
    pub topic_set_at: Option<DateTime<Utc>>,
    pub modes: HashSet<char>,
    pub key: Option<String>,
    pub limit: Option<u32>,
    pub users: HashMap<String, UserStatus>,
    pub creation_time: Option<DateTime<Utc>>,
}

/// User status in channel
#[derive(Debug, Clone, PartialEq)]
pub struct UserStatus {
    pub modes: HashSet<char>, // @, +, %, etc.
    pub away: bool,
    pub account: Option<String>,
}
```

### Messaging

```rust
impl Client {
    /// Send a message to a channel or user
    pub async fn send_message(
        &mut self, 
        server_id: ServerId, 
        target: &str, 
        message: &str
    ) -> Result<()> {
        // Send PRIVMSG
    }
    
    /// Send a notice
    pub async fn send_notice(
        &mut self, 
        server_id: ServerId, 
        target: &str, 
        message: &str
    ) -> Result<()> {
        // Send NOTICE
    }
    
    /// Send a CTCP request
    pub async fn send_ctcp(
        &mut self,
        server_id: ServerId,
        target: &str,
        command: &str,
        params: Option<&str>
    ) -> Result<()> {
        // Send CTCP message
    }
    
    /// Send an action (/me)
    pub async fn send_action(
        &mut self,
        server_id: ServerId,
        target: &str,
        action: &str
    ) -> Result<()> {
        self.send_ctcp(server_id, target, "ACTION", Some(action)).await
    }
}
```

### State Queries

```rust
impl Client {
    /// Get current nickname
    pub fn current_nick(&self, server_id: ServerId) -> Option<&str> {
        // Return current nick
    }
    
    /// Get user information
    pub fn user_info(&self, server_id: ServerId, nick: &str) -> Option<&UserInfo> {
        // Return user details
    }
    
    /// Check if user is online
    pub fn is_user_online(&self, server_id: ServerId, nick: &str) -> bool {
        // Check user presence
    }
    
    /// Get message buffer for a target
    pub fn messages(&self, server_id: ServerId, target: &str) -> &[Message] {
        // Return message history
    }
}

/// User information
pub struct UserInfo {
    pub nick: String,
    pub user: Option<String>,
    pub host: Option<String>,
    pub realname: Option<String>,
    pub account: Option<String>,
    pub away: bool,
    pub away_message: Option<String>,
    pub channels: HashSet<String>,
}
```

## Event System

### Event Types

```rust
/// IRC events that can be handled
#[derive(Debug, Clone)]
pub enum IrcEvent {
    /// Connected to server
    Connected { server_id: ServerId },
    
    /// Disconnected from server
    Disconnected { server_id: ServerId, reason: String },
    
    /// Message received
    Message {
        server_id: ServerId,
        message: Message,
    },
    
    /// User joined channel
    Join {
        server_id: ServerId,
        channel: String,
        user: UserInfo,
    },
    
    /// User left channel
    Part {
        server_id: ServerId,
        channel: String,
        user: String,
        message: Option<String>,
    },
    
    /// User quit
    Quit {
        server_id: ServerId,
        user: String,
        message: String,
    },
    
    /// User was kicked
    Kick {
        server_id: ServerId,
        channel: String,
        user: String,
        kicker: String,
        reason: Option<String>,
    },
    
    /// Nick change
    NickChange {
        server_id: ServerId,
        old_nick: String,
        new_nick: String,
    },
    
    /// Topic changed
    TopicChange {
        server_id: ServerId,
        channel: String,
        topic: Option<String>,
        setter: String,
    },
    
    /// Mode changed
    ModeChange {
        server_id: ServerId,
        target: String,
        modes: String,
        params: Vec<String>,
        setter: String,
    },
    
    /// CTCP request received
    CtcpRequest {
        server_id: ServerId,
        from: String,
        to: String,
        command: String,
        params: Option<String>,
    },
    
    /// DCC request received
    DccRequest {
        server_id: ServerId,
        from: String,
        request_type: DccRequestType,
    },
    
    /// Raw IRC message (for debugging/extensions)
    Raw {
        server_id: ServerId,
        message: IrcMessage,
    },
}

/// DCC request types
#[derive(Debug, Clone)]
pub enum DccRequestType {
    Chat { address: SocketAddr },
    Send { filename: String, address: SocketAddr, size: u64 },
    Resume { filename: String, port: u16, position: u64 },
}
```

### Event Handling

```rust
/// Event handler trait
pub trait EventHandler: Send + Sync {
    /// Handle an IRC event
    fn handle_event(&mut self, event: &IrcEvent);
}

impl Client {
    /// Register an event handler
    pub fn add_handler<H: EventHandler + 'static>(&mut self, handler: H) {
        // Add handler to event system
    }
    
    /// Register a closure as event handler
    pub fn on<F>(&mut self, event_type: EventType, handler: F)
    where
        F: Fn(&IrcEvent) + Send + Sync + 'static
    {
        // Register typed handler
    }
}

/// Event types for filtering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    Connected,
    Disconnected,
    Message,
    Join,
    Part,
    Quit,
    Kick,
    NickChange,
    TopicChange,
    ModeChange,
    CtcpRequest,
    DccRequest,
    Raw,
}
```

## Message Types

### Message Structure

```rust
/// Parsed IRC message
#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub timestamp: DateTime<Utc>,
    pub server_id: ServerId,
    pub message_type: MessageType,
    pub from: String,
    pub to: String,
    pub content: String,
    pub tags: HashMap<String, String>,
}

/// Unique message identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(pub u64);

/// Message types
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Privmsg,
    Notice,
    Action,
    Join,
    Part,
    Quit,
    Kick,
    Nick,
    Topic,
    Mode,
    Server,
    Error,
}
```

### Raw IRC Message

```rust
/// Raw IRC message for protocol-level operations
#[derive(Debug, Clone)]
pub struct IrcMessage {
    pub tags: HashMap<String, String>,
    pub prefix: Option<Prefix>,
    pub command: String,
    pub params: Vec<String>,
}

/// Message prefix (source)
#[derive(Debug, Clone)]
pub enum Prefix {
    Server(String),
    User { nick: String, user: Option<String>, host: Option<String> },
}

impl IrcMessage {
    /// Parse a raw IRC message
    pub fn parse(line: &str) -> Result<Self> {
        // Parse implementation
    }
    
    /// Serialize to IRC wire format
    pub fn to_string(&self) -> String {
        // Serialization implementation
    }
}
```

## DCC API

### DCC File Transfers

```rust
/// DCC transfer manager
impl Client {
    /// Send a file via DCC
    pub async fn dcc_send(
        &mut self,
        server_id: ServerId,
        nick: &str,
        file_path: &Path
    ) -> Result<TransferId> {
        // Initiate DCC SEND
    }
    
    /// Accept incoming DCC transfer
    pub async fn dcc_accept(&mut self, transfer_id: TransferId) -> Result<()> {
        // Accept transfer
    }
    
    /// Resume a partial transfer
    pub async fn dcc_resume(&mut self, transfer_id: TransferId) -> Result<()> {
        // Resume from last position
    }
    
    /// Cancel a transfer
    pub async fn dcc_cancel(&mut self, transfer_id: TransferId) -> Result<()> {
        // Cancel ongoing transfer
    }
    
    /// Get transfer status
    pub fn dcc_status(&self, transfer_id: TransferId) -> Option<&TransferStatus> {
        // Return transfer info
    }
}

/// Transfer identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferId(pub u64);

/// Transfer status
#[derive(Debug, Clone)]
pub struct TransferStatus {
    pub transfer_type: TransferType,
    pub peer: String,
    pub filename: String,
    pub size: u64,
    pub transferred: u64,
    pub speed: f64, // bytes per second
    pub state: TransferState,
    pub started_at: DateTime<Utc>,
    pub eta: Option<Duration>,
}

/// Transfer type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferType {
    Send,
    Receive,
}

/// Transfer state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransferState {
    Pending,
    Connecting,
    Transferring,
    Paused,
    Completed,
    Failed(TransferError),
    Cancelled,
}
```

### DCC Chat

```rust
impl Client {
    /// Initiate DCC chat
    pub async fn dcc_chat(&mut self, server_id: ServerId, nick: &str) -> Result<ChatId> {
        // Start DCC CHAT
    }
    
    /// Accept incoming DCC chat
    pub async fn dcc_chat_accept(&mut self, chat_id: ChatId) -> Result<()> {
        // Accept chat request
    }
    
    /// Send message in DCC chat
    pub async fn dcc_chat_send(&mut self, chat_id: ChatId, message: &str) -> Result<()> {
        // Send chat message
    }
    
    /// Close DCC chat
    pub async fn dcc_chat_close(&mut self, chat_id: ChatId) -> Result<()> {
        // Close chat connection
    }
}

/// DCC chat identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChatId(pub u64);
```

## Advanced Features

### Capability Negotiation

```rust
impl Client {
    /// Request IRCv3 capabilities
    pub async fn request_capabilities(
        &mut self,
        server_id: ServerId,
        caps: &[&str]
    ) -> Result<Vec<String>> {
        // Request and return granted capabilities
    }
    
    /// Check if capability is enabled
    pub fn has_capability(&self, server_id: ServerId, cap: &str) -> bool {
        // Check capability status
    }
    
    /// Get all enabled capabilities
    pub fn capabilities(&self, server_id: ServerId) -> &HashSet<String> {
        // Return capability set
    }
}
```

### SASL Authentication

```rust
/// SASL configuration
pub struct SaslConfig {
    pub mechanism: SaslMechanism,
    pub username: String,
    pub password: String,
    pub authzid: Option<String>,
}

/// SASL mechanisms
#[derive(Debug, Clone)]
pub enum SaslMechanism {
    Plain,
    External,
    ScramSha256,
}
```

### Proxy Support

```rust
/// Proxy configuration
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub address: SocketAddr,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Proxy types
#[derive(Debug, Clone, Copy)]
pub enum ProxyType {
    Socks5,
    Http,
}

impl ServerConfig {
    /// Set proxy for connection
    pub fn with_proxy(mut self, proxy: ProxyConfig) -> Self {
        self.proxy = Some(proxy);
        self
    }
}
```

## Script Integration

### Script Engine Access

```rust
impl Client {
    /// Get reference to script engine
    pub fn script_engine(&self) -> &ScriptEngine {
        // Return script engine
    }
    
    /// Get mutable reference to script engine
    pub fn script_engine_mut(&mut self) -> &mut ScriptEngine {
        // Return mutable script engine
    }
}

/// Script engine interface
pub struct ScriptEngine {
    // Internal implementation
}

impl ScriptEngine {
    /// Load a script file
    pub fn load_script(&mut self, path: &Path) -> Result<ScriptId> {
        // Load and compile script
    }
    
    /// Execute a script string
    pub fn execute(&mut self, code: &str) -> Result<ScriptValue> {
        // Execute Lua code
    }
    
    /// Call a script function
    pub fn call_function(
        &mut self,
        script_id: ScriptId,
        function: &str,
        args: Vec<ScriptValue>
    ) -> Result<ScriptValue> {
        // Call script function
    }
    
    /// Register a Rust function for scripts
    pub fn register_function<F>(&mut self, name: &str, func: F) -> Result<()>
    where
        F: Fn(&[ScriptValue]) -> Result<ScriptValue> + Send + Sync + 'static
    {
        // Register native function
    }
}

/// Script identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScriptId(pub u64);

/// Script value types
#[derive(Debug, Clone)]
pub enum ScriptValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Table(HashMap<String, ScriptValue>),
    Function,
    UserData,
}
```

## Error Handling

### Error Types

```rust
/// Main error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Script error: {0}")]
    Script(#[from] ScriptError),
    
    #[error("DCC error: {0}")]
    Dcc(#[from] DccError),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Not connected to server")]
    NotConnected,
    
    #[error("Invalid target: {0}")]
    InvalidTarget(String),
    
    #[error("Operation timed out")]
    Timeout,
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
```

### Connection Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Failed to resolve host: {0}")]
    DnsError(String),
    
    #[error("TLS error: {0}")]
    TlsError(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Connection refused")]
    ConnectionRefused,
    
    #[error("Connection reset")]
    ConnectionReset,
    
    #[error("Connection timeout")]
    Timeout,
}
```

## Thread Safety

All public API methods are designed to be thread-safe:

```rust
// Client can be shared across threads
let client = Arc::new(Mutex::new(Client::new()));

// Clone for another thread
let client_clone = client.clone();
tokio::spawn(async move {
    let mut client = client_clone.lock().await;
    client.send_message(server_id, "#channel", "Hello from thread!").await?;
});
```

## Examples

### Basic Bot

```rust
use rustirc::{Client, ServerConfig, IrcEvent, EventType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new();
    
    // Register message handler
    client.on(EventType::Message, |event| {
        if let IrcEvent::Message { server_id, message } = event {
            if message.content.starts_with("!hello") {
                // Response handled through commands queue
                println!("Responding to hello command");
            }
        }
    });
    
    // Connect to server
    let server = client.connect(ServerConfig {
        address: "irc.libera.chat:6697".parse()?,
        use_tls: true,
        nick: "rustbot".to_string(),
        username: "rustbot".to_string(),
        realname: "RustIRC Bot".to_string(),
        autojoin: vec!["#rust-beginners".to_string()],
        ..Default::default()
    }).await?;
    
    // Keep running
    tokio::signal::ctrl_c().await?;
    client.disconnect(server).await?;
    
    Ok(())
}
```

### File Transfer

```rust
use rustirc::{Client, IrcEvent, DccRequestType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::new();
    
    // Handle incoming DCC requests
    client.add_handler(DccHandler);
    
    // ... connect to server ...
    
    // Send a file
    let transfer_id = client.dcc_send(
        server_id,
        "friend",
        Path::new("/path/to/file.zip")
    ).await?;
    
    Ok(())
}

struct DccHandler;

impl EventHandler for DccHandler {
    fn handle_event(&mut self, event: &IrcEvent) {
        if let IrcEvent::DccRequest { from, request_type, .. } = event {
            match request_type {
                DccRequestType::Send { filename, size, .. } => {
                    println!("{} wants to send {} ({} bytes)", from, filename, size);
                    // Auto-accept from trusted users
                }
                _ => {}
            }
        }
    }
}
```

## Best Practices

1. **Error Handling**: Always handle connection errors gracefully
2. **Resource Management**: Use timeouts for network operations
3. **Memory Usage**: Configure appropriate buffer sizes
4. **Security**: Validate all user input before sending
5. **Performance**: Use batch operations when possible
6. **Threading**: Minimize lock contention by batching updates

## Version Compatibility

This API is designed for RustIRC 1.0 and follows semantic versioning:
- Major version changes may break compatibility
- Minor version changes add functionality without breaking existing code
- Patch versions contain only bug fixes

For migration guides between versions, see the changelog and migration documentation.