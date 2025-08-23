# rustirc-core

Core functionality and shared components for the RustIRC client.

## Overview

The `rustirc-core` crate provides the foundational components that power the RustIRC client:

- **Connection Management**: Async IRC server connections with automatic reconnection
- **Event System**: Publish-subscribe event bus for client communication
- **State Management**: Application state and configuration handling
- **Authentication**: SASL authentication support (PLAIN, EXTERNAL, SCRAM-SHA-256)
- **Error Handling**: Comprehensive error types and result handling
- **CLI Interface**: Command-line interface components and utilities

## Features

- 🔌 **Multi-server connections** with connection pooling
- 🔐 **TLS/SSL support** with certificate validation
- 📡 **Event-driven architecture** for extensibility
- 🔄 **Automatic reconnection** with exponential backoff
- 🛡️ **SASL authentication** with multiple mechanisms
- ⚡ **Async/await** throughout for performance
- 🧪 **Mock server support** for testing

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustirc-core = "0.3.3"
```

### Basic Connection Example

```rust
use rustirc_core::connection::{IrcConnection, ConnectionConfig};
use rustirc_core::events::EventBus;
use rustirc_protocol::Command;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create event bus for receiving events
    let event_bus = Arc::new(EventBus::new());
    
    // Configure connection
    let config = ConnectionConfig {
        server: "irc.libera.chat".to_string(),
        port: 6697,
        use_tls: true,
        nickname: "mybotname".to_string(),
        username: "mybot".to_string(),
        realname: "My IRC Bot".to_string(),
        ..Default::default()
    };
    
    // Create connection
    let connection = IrcConnection::new(config, event_bus);
    
    // Connect to server
    connection.connect().await?;
    
    // Join a channel
    let join_cmd = Command::Join {
        channels: vec!["#rust".to_string()],
        keys: vec![],
    };
    connection.send_command(join_cmd).await?;
    
    // Send a message
    let msg_cmd = Command::PrivMsg {
        target: "#rust".to_string(),
        text: "Hello from RustIRC!".to_string(),
    };
    connection.send_command(msg_cmd).await?;
    
    Ok(())
}
```

### Event Handling

```rust
use rustirc_core::events::{Event, EventHandler, EventBus};
use async_trait::async_trait;

struct MessageLogger;

#[async_trait]
impl EventHandler for MessageLogger {
    async fn handle(&self, event: &Event) {
        match event {
            Event::MessageReceived { connection_id, message } => {
                println!("[{}] {}", connection_id, message);
            },
            Event::Connected { connection_id } => {
                println!("Connected to {}", connection_id);
            },
            _ => {}
        }
    }
    
    fn priority(&self) -> i32 {
        10 // Higher priority than default handlers
    }
}

#[tokio::main]
async fn main() {
    let event_bus = EventBus::new();
    event_bus.register(MessageLogger).await;
    
    // Events will now be logged as they occur
}
```

### Multi-Server Management

```rust
use rustirc_core::connection::{ConnectionManager, ConnectionConfig};
use rustirc_core::events::EventBus;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_bus = Arc::new(EventBus::new());
    let manager = ConnectionManager::new(event_bus);
    
    // Add multiple servers
    let libera_config = ConnectionConfig {
        server: "irc.libera.chat".to_string(),
        port: 6697,
        use_tls: true,
        nickname: "mybot".to_string(),
        ..Default::default()
    };
    
    let oftc_config = ConnectionConfig {
        server: "irc.oftc.net".to_string(),
        port: 6697,
        use_tls: true,
        nickname: "mybot".to_string(),
        ..Default::default()
    };
    
    manager.add_connection("libera".to_string(), libera_config).await?;
    manager.add_connection("oftc".to_string(), oftc_config).await?;
    
    // Connect to all servers
    manager.connect_all().await?;
    
    Ok(())
}
```

## Architecture

### Event System

The event system is built around a publish-subscribe pattern:

- **EventBus**: Central hub for event distribution
- **EventHandler**: Trait for handling specific events
- **Event**: Enumeration of all possible IRC client events

### Connection Management

Connections are managed through:

- **IrcConnection**: Individual server connection
- **ConnectionManager**: Multi-server connection pool
- **ConnectionConfig**: Configuration for connection parameters

### State Management

Application state includes:

- **Connection states**: Tracking server connection status
- **Channel information**: User lists, topics, modes
- **Configuration**: User preferences and settings
- **Authentication**: Stored credentials and session data

## API Documentation

For detailed API documentation, run:

```bash
cargo doc --open
```

Or visit the [online documentation](https://docs.rs/rustirc-core).

## Dependencies

- **tokio**: Async runtime for networking
- **rustls**: TLS implementation for secure connections
- **async-trait**: Async trait support
- **thiserror**: Error handling macros
- **tracing**: Structured logging

## Testing

Run the test suite:

```bash
cargo test
```

Run tests with mock server:

```bash
cargo test --features mock-server
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.