# rustirc-protocol

IRC protocol implementation with comprehensive parsing and validation.

## Overview

The `rustirc-protocol` crate provides a robust implementation of the IRC protocol including:

- **Message Parsing**: Complete IRC message parsing with IRCv3 support
- **Protocol Validation**: Security-focused validation and sanitization
- **Command Building**: Type-safe IRC command construction
- **Format Compliance**: RFC 1459, RFC 2812, and IRCv3 standard compliance
- **CTCP Support**: Client-to-Client Protocol message handling

## Features

- 📝 **Complete IRC parsing** with tags, prefix, command, and parameters
- 🔒 **Security validation** to prevent injection attacks
- 🏷️ **IRCv3 message tags** support for modern IRC features
- 📋 **Command builders** for type-safe message construction
- 🛡️ **Input sanitization** with configurable validation rules
- ⚡ **High performance** parsing with minimal allocations
- 🧪 **Comprehensive tests** covering edge cases and malformed input

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustirc-protocol = "0.3.3"
```

### Basic Message Parsing

```rust
use rustirc_protocol::parser::Parser;

// Parse a simple IRC message
let message = Parser::parse_message("PING :irc.example.com")?;
assert_eq!(message.command, "PING");
assert_eq!(message.params, vec!["irc.example.com"]);

// Parse message with prefix
let message = Parser::parse_message(":nick!user@host PRIVMSG #channel :Hello, world!")?;
assert_eq!(message.command, "PRIVMSG");
assert_eq!(message.params, vec!["#channel", "Hello, world!"]);

// Parse IRCv3 message with tags
let message = Parser::parse_message(
    "@time=2021-01-01T00:00:00.000Z :server NOTICE #channel :Test"
)?;
assert!(message.tags.is_some());
assert_eq!(message.command, "NOTICE");
```

### Validation and Security

```rust
use rustirc_protocol::validation::IrcValidator;

let validator = IrcValidator::new();

// Validate nicknames
assert!(validator.validate_nickname("alice").is_ok());
assert!(validator.validate_nickname("user@host").is_err()); // Invalid

// Validate channel names
assert!(validator.validate_channel_name("#rust").is_ok());
assert!(validator.validate_channel_name("notachannel").is_err()); // No #

// Sanitize input
let clean_nick = validator.sanitize_nickname("bad@nick!");
assert_eq!(clean_nick, "badnick");

// Strict RFC compliance mode
let strict_validator = IrcValidator::strict();
assert!(strict_validator.validate_nickname("verylongname").is_err());
```

### Command Building

```rust
use rustirc_protocol::{Command, Message};

// Build IRC commands
let join_cmd = Command::Join {
    channels: vec!["#rust".to_string(), "#programming".to_string()],
    keys: vec!["secret123".to_string()],
};

let privmsg_cmd = Command::PrivMsg {
    target: "#rust".to_string(),
    text: "Hello, everyone!".to_string(),
};

// Convert to IRC message format
let message = join_cmd.to_message();
println!("{}", message); // "JOIN #rust,#programming secret123"

// Build raw messages
let raw_message = Message {
    tags: None,
    prefix: Some(rustirc_protocol::Prefix::Server("irc.example.com".to_string())),
    command: "001".to_string(),
    params: vec!["nick".to_string(), "Welcome to the network".to_string()],
};
```

### IRCv3 Tag Handling

```rust
use rustirc_protocol::{Tag, parser::Parser};

// Parse message with tags
let message = Parser::parse_message(
    "@time=2021-01-01T00:00:00.000Z;msgid=abc123 :nick!user@host PRIVMSG #channel :Hello"
)?;

if let Some(tags) = &message.tags {
    for tag in tags {
        match tag.key.as_str() {
            "time" => {
                if let Some(timestamp) = &tag.value {
                    println!("Message timestamp: {}", timestamp);
                }
            },
            "msgid" => {
                if let Some(id) = &tag.value {
                    println!("Message ID: {}", id);
                }
            },
            _ => {}
        }
    }
}
```

### CTCP Message Handling

```rust
use rustirc_protocol::ctcp::{CtcpMessage, parse_ctcp};

// Parse CTCP message
let text = "\x01VERSION RustIRC 0.3.3\x01";
if let Some(ctcp) = parse_ctcp(text) {
    match ctcp {
        CtcpMessage::Version(version) => {
            println!("Client version: {}", version);
        },
        CtcpMessage::Action(action) => {
            println!("* {}", action);
        },
        _ => {}
    }
}

// Create CTCP messages
let version_ctcp = CtcpMessage::Version("RustIRC 0.3.3".to_string());
let version_text = version_ctcp.to_string();
```

### Custom Validation Rules

```rust
use rustirc_protocol::validation::IrcValidator;

// Create validator with custom settings
let mut validator = IrcValidator::new();
validator.max_nickname_length = 16; // Custom nick length
validator.max_channel_length = 64;  // Custom channel length
validator.strict_mode = true;       // Enable strict validation

// Validate with custom rules
let result = validator.validate_nickname("verylongnicknamethatexceedslimit");
assert!(result.is_err());

// Sanitize with custom rules
let sanitized = validator.sanitize_parameter("text\x00with\r\nnull\x00bytes");
assert_eq!(sanitized, "textwithbytes"); // Null bytes removed
```

## Architecture

### Message Structure

IRC messages follow this format:
```
[@tags] [:prefix] command [params...] [:trailing]
```

The parser handles each component:

- **Tags**: IRCv3 key-value metadata (optional)
- **Prefix**: Server name or user info (optional)
- **Command**: Numeric (001-999) or alphabetic command
- **Parameters**: Space-separated parameters
- **Trailing**: Final parameter that may contain spaces

### Validation Layers

Multiple validation layers ensure security:

1. **Length validation**: Prevents oversized messages
2. **Character validation**: Blocks control characters and invalid sequences
3. **Format validation**: Ensures proper IRC message structure
4. **Protocol validation**: Enforces IRC specification compliance

### Command Types

Commands are categorized by type:

- **Connection**: NICK, USER, PASS, QUIT
- **Channel**: JOIN, PART, TOPIC, MODE
- **Messaging**: PRIVMSG, NOTICE
- **Information**: WHO, WHOIS, LIST
- **Server**: PING, PONG, ERROR

## Protocol Support

### RFC Compliance

- **RFC 1459**: Original IRC protocol specification
- **RFC 2812**: Updated IRC client protocol
- **RFC 2813**: IRC server protocol elements

### IRCv3 Extensions

- Message tags with escaping/unescaping
- Capability negotiation support
- Extended JOIN format
- Server-time extension
- Message ID support

### CTCP Support

- ACTION messages (/me commands)
- VERSION requests/responses
- TIME requests/responses
- Custom CTCP command support

## API Documentation

For detailed API documentation, run:

```bash
cargo doc --open
```

Or visit the [online documentation](https://docs.rs/rustirc-protocol).

## Testing

The crate includes comprehensive tests covering:

```bash
# Run all tests
cargo test

# Test with specific features
cargo test --features ircv3-extensions

# Run benchmark tests
cargo bench
```

## Performance

The parser is optimized for performance with:

- Zero-copy parsing where possible
- Minimal heap allocations
- Efficient string handling
- Comprehensive benchmarking

Typical performance on modern hardware:
- **Simple messages**: ~50ns per message
- **Complex messages with tags**: ~200ns per message
- **Validation**: ~10ns per field

## Error Handling

The crate provides detailed error information:

```rust
use rustirc_protocol::parser::{Parser, ParseError};

match Parser::parse_message("invalid message format") {
    Ok(message) => { /* Handle success */ },
    Err(ParseError::InvalidFormat(msg)) => {
        eprintln!("Parse error: {}", msg);
    },
    Err(ParseError::MessageTooLong(len)) => {
        eprintln!("Message too long: {} bytes", len);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## Dependencies

- **thiserror**: Error handling macros
- **serde** (optional): Serialization support

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../../LICENSE-MIT))

at your option.