# Phase 2: Core IRC Engine Development

**Duration**: 3-6 weeks  
**Goal**: Implement the foundational IRC protocol engine with basic connectivity

## Overview

Phase 2 focuses on building the core IRC engine that will power all client functionality. This includes the network layer, protocol parser, state management, and basic command handling. By the end of this phase, we'll have a working IRC client capable of basic operations via CLI.

## Objectives

1. Implement async network layer with Tokio
2. Create robust IRC message parser
3. Build state management system
4. Implement core IRC commands
5. Add basic SASL authentication
6. Create CLI prototype for testing

## Network Layer Implementation

### Connection Manager
```rust
// rustirc-core/src/network/manager.rs
pub struct ConnectionManager {
    connections: HashMap<ServerId, ServerConnection>,
    runtime: Arc<Runtime>,
}

impl ConnectionManager {
    pub async fn connect(&mut self, config: ServerConfig) -> Result<ServerId> {
        // Implementation
    }
    
    pub async fn disconnect(&mut self, server_id: ServerId) -> Result<()> {
        // Implementation
    }
}
```

### Server Connection
```rust
// rustirc-core/src/network/connection.rs
pub struct ServerConnection {
    id: ServerId,
    stream: TlsStream<TcpStream>,
    config: ServerConfig,
    state: ConnectionState,
    write_tx: mpsc::Sender<IrcMessage>,
    read_handle: JoinHandle<()>,
    write_handle: JoinHandle<()>,
}

impl ServerConnection {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        // Connect to server
        // Set up TLS if required
        // Spawn read/write tasks
    }
}
```

### Tasks
- [ ] Implement TCP connection handling
- [ ] Add TLS support with rustls
- [ ] Create connection pooling
- [ ] Implement automatic reconnection
- [ ] Add connection rate limiting
- [ ] Handle multiple server connections

## Protocol Implementation

### Message Parser
```rust
// rustirc-protocol/src/parser.rs
#[derive(Debug, Clone)]
pub struct IrcMessage {
    pub tags: Option<HashMap<String, String>>,
    pub prefix: Option<Prefix>,
    pub command: Command,
    pub params: Vec<String>,
}

pub fn parse_message(input: &str) -> Result<IrcMessage> {
    // Parse IRCv3 tags if present
    // Parse prefix if present
    // Parse command
    // Parse parameters
}
```

### Core Commands
Implement handlers for essential IRC commands:

#### Connection Registration
- [ ] NICK - Set nickname
- [ ] USER - Set user information
- [ ] PASS - Server password
- [ ] CAP - Capability negotiation
- [ ] AUTHENTICATE - SASL authentication

#### Channel Operations
- [ ] JOIN - Join channels
- [ ] PART - Leave channels
- [ ] MODE - Set channel/user modes
- [ ] TOPIC - Get/set channel topic
- [ ] NAMES - List channel users

#### Messaging
- [ ] PRIVMSG - Send messages
- [ ] NOTICE - Send notices
- [ ] CTCP handling (ACTION, VERSION, etc.)

#### User Queries
- [ ] WHOIS - User information
- [ ] WHO - Channel user list
- [ ] USERHOST - User host information

### Message Serializer
```rust
// rustirc-protocol/src/serializer.rs
impl IrcMessage {
    pub fn to_string(&self) -> String {
        let mut output = String::new();
        
        // Add tags if present
        if let Some(tags) = &self.tags {
            // Format: @key=value;key2=value2
        }
        
        // Add prefix if present
        if let Some(prefix) = &self.prefix {
            // Format: :prefix
        }
        
        // Add command and params
        // Handle trailing parameter
        
        output
    }
}
```

## State Management

### Core State Structure
```rust
// rustirc-core/src/state/mod.rs
pub struct IrcState {
    servers: HashMap<ServerId, ServerState>,
    channels: HashMap<(ServerId, String), ChannelState>,
    users: HashMap<(ServerId, String), UserState>,
}

pub struct ServerState {
    pub id: ServerId,
    pub config: ServerConfig,
    pub connection_state: ConnectionState,
    pub capabilities: HashSet<String>,
    pub isupport: HashMap<String, String>,
    pub current_nick: String,
}

pub struct ChannelState {
    pub name: String,
    pub topic: Option<String>,
    pub users: HashSet<String>,
    pub modes: ChannelModes,
    pub creation_time: Option<DateTime<Utc>>,
}
```

### State Updates
```rust
// rustirc-core/src/state/updates.rs
impl IrcState {
    pub fn handle_message(&mut self, server: ServerId, msg: IrcMessage) {
        match msg.command {
            Command::JOIN(channel) => self.handle_join(server, channel, msg),
            Command::PRIVMSG(target, text) => self.handle_privmsg(server, target, text, msg),
            // ... other commands
        }
    }
}
```

### Tasks
- [ ] Design state data structures
- [ ] Implement thread-safe state access
- [ ] Create state update handlers
- [ ] Add state persistence
- [ ] Implement state queries
- [ ] Add state change notifications

## SASL Authentication

### SASL Handler
```rust
// rustirc-core/src/auth/sasl.rs
pub enum SaslMechanism {
    Plain,
    External,
    ScramSha256,
}

pub struct SaslHandler {
    mechanism: SaslMechanism,
    state: SaslState,
}

impl SaslHandler {
    pub fn start(&mut self) -> String {
        match self.mechanism {
            SaslMechanism::Plain => "AUTHENTICATE PLAIN",
            // ... other mechanisms
        }
    }
    
    pub fn respond(&mut self, challenge: &str) -> Result<String> {
        // Handle SASL challenge/response
    }
}
```

### Tasks
- [ ] Implement PLAIN mechanism
- [ ] Add capability negotiation
- [ ] Handle authentication flow
- [ ] Store credentials securely
- [ ] Add authentication retry logic
- [ ] Implement timeout handling

## CLI Prototype

### Basic CLI Application
```rust
// rustirc-cli/src/main.rs
#[derive(Parser)]
struct Cli {
    #[clap(short, long)]
    server: String,
    
    #[clap(short, long, default_value = "6667")]
    port: u16,
    
    #[clap(short, long)]
    nick: String,
    
    #[clap(long)]
    tls: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    
    // Initialize logging
    // Create IRC client
    // Connect to server
    // Run interactive loop
}
```

### Interactive Commands
- [ ] /connect - Connect to server
- [ ] /join - Join channel
- [ ] /msg - Send private message
- [ ] /quit - Disconnect and exit
- [ ] /raw - Send raw IRC command
- [ ] /debug - Toggle debug output

## Testing Infrastructure

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_message() {
        let msg = parse_message("PRIVMSG #channel :Hello, world!").unwrap();
        assert_eq!(msg.command, Command::PRIVMSG);
        assert_eq!(msg.params[0], "#channel");
        assert_eq!(msg.params[1], "Hello, world!");
    }
    
    #[test]
    fn test_parse_message_with_tags() {
        let msg = parse_message("@time=2023-01-01T00:00:00Z PRIVMSG #channel :Test").unwrap();
        assert!(msg.tags.is_some());
        assert_eq!(msg.tags.unwrap().get("time"), Some(&"2023-01-01T00:00:00Z".to_string()));
    }
}
```

### Integration Tests
```rust
// tests/connection_test.rs
#[tokio::test]
async fn test_server_connection() {
    let mock_server = MockIrcServer::new();
    let addr = mock_server.start().await;
    
    let config = ServerConfig {
        address: addr,
        nick: "testbot".to_string(),
        // ...
    };
    
    let mut client = IrcClient::new();
    client.connect(config).await.unwrap();
    
    // Test connection flow
}
```

### Mock IRC Server
```rust
// tests/mock_server.rs
pub struct MockIrcServer {
    responses: HashMap<String, Vec<String>>,
}

impl MockIrcServer {
    pub async fn start(&self) -> SocketAddr {
        // Start TCP listener
        // Handle connections
        // Respond to commands based on responses map
    }
}
```

## Performance Benchmarks

### Parser Benchmarks
```rust
#[bench]
fn bench_parse_simple_message(b: &mut Bencher) {
    b.iter(|| {
        parse_message("PRIVMSG #channel :Hello, world!")
    });
}

#[bench]
fn bench_parse_message_with_tags(b: &mut Bencher) {
    b.iter(|| {
        parse_message("@tag1=value1;tag2=value2 :nick!user@host PRIVMSG #channel :Message")
    });
}
```

## Deliverables

By the end of Phase 2:

1. **Working Network Layer**
   - Multi-server connection support
   - TLS encryption
   - Automatic reconnection

2. **Complete Protocol Parser**
   - Full IRC message parsing
   - IRCv3 tag support
   - CTCP handling

3. **State Management System**
   - Thread-safe state storage
   - State update mechanisms
   - Query interfaces

4. **Basic SASL Support**
   - PLAIN mechanism
   - Capability negotiation

5. **CLI Prototype**
   - Connect to servers
   - Join channels
   - Send/receive messages
   - Basic commands

## Success Criteria

Phase 2 is complete when:
- [ ] Can connect to major IRC networks (Libera.Chat, OFTC)
- [ ] Can join channels and send messages
- [ ] Handles disconnections gracefully
- [ ] Passes all protocol compliance tests
- [ ] CLI client is usable for basic IRC tasks
- [ ] Performance benchmarks meet targets

## Next Phase

With the core engine complete, Phase 3 will focus on building the graphical user interface, making the client accessible to a broader audience beyond the command line.