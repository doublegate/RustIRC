# IRC Protocol Implementation

## Overview

RustIRC implements the modern IRC protocol as defined by the living specification at [modern.ircdocs.horse](https://modern.ircdocs.horse/), which consolidates and supersedes the historical RFCs 1459 and 2812. This document details our implementation approach and compliance strategy.

## Message Format

### Basic Structure
```
[@tags] [:prefix] <command> [params] [:trailing]
```

### Components

#### Tags (IRCv3)
Optional metadata in key=value format:
```
@time=2023-01-01T00:00:00.000Z;account=alice
```

#### Prefix
Source of the message:
```
:nick!user@host         # User prefix
:server.example.com     # Server prefix
```

#### Command
Either numeric (3 digits) or string:
```
PRIVMSG                 # String command
001                     # Numeric reply
```

#### Parameters
Space-separated arguments with optional trailing:
```
PRIVMSG #channel :Hello, world!
```

## Core Commands Implementation

### Connection Registration

#### NICK
```rust
pub fn handle_nick(params: Vec<String>) -> Result<Command> {
    let nickname = params.get(0)
        .ok_or(Error::MissingParameter)?;
    
    validate_nickname(nickname)?;
    
    Ok(Command::Nick {
        nickname: nickname.to_string(),
        hopcount: params.get(1).map(|s| s.parse().ok()).flatten(),
    })
}
```

Validation rules:
- Must not start with: # & + ! ~ @ %
- Must not contain: space, comma, asterisk, question mark
- Maximum length: 30 characters (configurable)

#### USER
```rust
pub fn handle_user(params: Vec<String>) -> Result<Command> {
    if params.len() < 4 {
        return Err(Error::NeedMoreParams);
    }
    
    Ok(Command::User {
        username: params[0].clone(),
        hostname: params[1].clone(),
        servername: params[2].clone(),
        realname: params[3].clone(),
    })
}
```

#### PASS
Optional server password:
```rust
pub fn handle_pass(params: Vec<String>) -> Result<Command> {
    Ok(Command::Pass {
        password: params.get(0)
            .ok_or(Error::MissingParameter)?
            .to_string(),
    })
}
```

### Channel Operations

#### JOIN
```rust
pub fn handle_join(params: Vec<String>) -> Result<Command> {
    let channels = params.get(0)
        .ok_or(Error::MissingParameter)?
        .split(',');
    
    let keys = params.get(1)
        .map(|k| k.split(',').collect::<Vec<_>>())
        .unwrap_or_default();
    
    let mut join_list = Vec::new();
    for (idx, channel) in channels.enumerate() {
        validate_channel_name(channel)?;
        join_list.push((
            channel.to_string(),
            keys.get(idx).map(|k| k.to_string()),
        ));
    }
    
    Ok(Command::Join { channels: join_list })
}
```

Channel types:
- `#` - Standard channels
- `&` - Local channels
- `+` - Modeless channels
- `!` - Safe channels

#### PART
```rust
pub fn handle_part(params: Vec<String>) -> Result<Command> {
    let channels = params.get(0)
        .ok_or(Error::MissingParameter)?
        .split(',')
        .map(String::from)
        .collect();
    
    Ok(Command::Part {
        channels,
        message: params.get(1).map(String::from),
    })
}
```

#### MODE
Complex command supporting both channel and user modes:
```rust
pub enum ModeChange {
    Add(char, Option<String>),
    Remove(char, Option<String>),
}

pub fn parse_mode_changes(mode_string: &str, params: &[String]) -> Vec<ModeChange> {
    let mut changes = Vec::new();
    let mut adding = true;
    let mut param_idx = 0;
    
    for ch in mode_string.chars() {
        match ch {
            '+' => adding = true,
            '-' => adding = false,
            mode => {
                let takes_param = mode_takes_parameter(mode, adding);
                let param = if takes_param {
                    let p = params.get(param_idx).cloned();
                    param_idx += 1;
                    p
                } else {
                    None
                };
                
                changes.push(if adding {
                    ModeChange::Add(mode, param)
                } else {
                    ModeChange::Remove(mode, param)
                });
            }
        }
    }
    
    changes
}
```

### Messaging Commands

#### PRIVMSG
```rust
pub fn handle_privmsg(params: Vec<String>) -> Result<Command> {
    if params.len() < 2 {
        return Err(Error::NeedMoreParams);
    }
    
    let targets = params[0].split(',').map(String::from).collect();
    let message = params[1].clone();
    
    // Check for CTCP
    let is_ctcp = message.starts_with('\x01') && message.ends_with('\x01');
    
    Ok(Command::Privmsg {
        targets,
        message,
        is_ctcp,
    })
}
```

#### NOTICE
Similar to PRIVMSG but should not generate automatic replies:
```rust
pub fn handle_notice(params: Vec<String>) -> Result<Command> {
    // Similar to PRIVMSG but sets is_notice flag
}
```

### Server Queries

#### WHOIS
```rust
pub fn handle_whois(params: Vec<String>) -> Result<Command> {
    let (server, nicknames) = match params.len() {
        1 => (None, params[0].split(',').map(String::from).collect()),
        2 => (Some(params[0].clone()), 
              params[1].split(',').map(String::from).collect()),
        _ => return Err(Error::NeedMoreParams),
    };
    
    Ok(Command::Whois { server, nicknames })
}
```

## Numeric Replies

### Reply Categories

#### Connection Registration (001-005)
```rust
pub enum RegistrationReply {
    RplWelcome = 001,        // :Welcome to the network
    RplYourHost = 002,       // :Your host is...
    RplCreated = 003,        // :This server was created...
    RplMyInfo = 004,         // Server version info
    RplISupport = 005,       // ISUPPORT tokens
}
```

#### Command Responses (200-399)
```rust
pub enum CommandReply {
    RplStatsCommands = 212,
    RplEndOfStats = 219,
    RplUserModeIs = 221,
    RplNoTopic = 331,
    RplTopic = 332,
    // ... many more
}
```

#### Error Replies (400-599)
```rust
pub enum ErrorReply {
    ErrNoSuchNick = 401,
    ErrNoSuchServer = 402,
    ErrNoSuchChannel = 403,
    ErrCannotSendToChan = 404,
    ErrTooManyChannels = 405,
    // ... many more
}
```

## CTCP (Client-To-Client Protocol)

### CTCP Message Format
```
\x01<command> [params]\x01
```

### Standard CTCP Commands

#### ACTION (/me)
```rust
pub fn create_action(target: &str, action: &str) -> String {
    format!("PRIVMSG {} :\x01ACTION {}\x01", target, action)
}
```

#### VERSION
```rust
pub fn handle_ctcp_version() -> String {
    format!("\x01VERSION RustIRC {} {}\x01", 
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_TARGET_OS"))
}
```

#### PING
```rust
pub fn handle_ctcp_ping(timestamp: &str) -> String {
    format!("\x01PING {}\x01", timestamp)
}
```

## Message Parsing

### Parser Implementation
```rust
pub struct IrcParser {
    max_params: usize,
    strict_mode: bool,
}

impl IrcParser {
    pub fn parse(&self, input: &str) -> Result<IrcMessage> {
        let mut parser = MessageParser::new(input);
        
        // Parse tags if present
        let tags = if parser.peek() == Some('@') {
            parser.advance();
            Some(self.parse_tags(&mut parser)?)
        } else {
            None
        };
        
        // Parse prefix if present
        let prefix = if parser.peek() == Some(':') {
            parser.advance();
            Some(self.parse_prefix(&mut parser)?)
        } else {
            None
        };
        
        // Parse command (required)
        let command = self.parse_command(&mut parser)?;
        
        // Parse parameters
        let params = self.parse_params(&mut parser)?;
        
        Ok(IrcMessage {
            tags,
            prefix,
            command,
            params,
        })
    }
}
```

### Tag Parsing
```rust
fn parse_tags(&self, parser: &mut MessageParser) -> Result<HashMap<String, String>> {
    let mut tags = HashMap::new();
    
    for tag_pair in parser.read_until(' ').split(';') {
        let parts: Vec<&str> = tag_pair.splitn(2, '=').collect();
        let key = parts[0];
        let value = parts.get(1)
            .map(|v| unescape_tag_value(v))
            .unwrap_or_default();
        
        tags.insert(key.to_string(), value);
    }
    
    Ok(tags)
}

fn unescape_tag_value(value: &str) -> String {
    value
        .replace("\\s", " ")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\:", ";")
        .replace("\\\\", "\\")
}
```

## Capability Negotiation

### CAP Commands
```rust
pub enum CapSubcommand {
    Ls(Option<u32>),    // CAP LS [version]
    List,               // CAP LIST
    Req(Vec<String>),   // CAP REQ :capabilities
    Ack(Vec<String>),   // CAP ACK :capabilities
    Nak(Vec<String>),   // CAP NAK :capabilities
    End,                // CAP END
}
```

### Capability Handler
```rust
pub struct CapabilityHandler {
    version: u32,
    available: HashSet<String>,
    enabled: HashSet<String>,
    
    // Capabilities we support
    supported: HashSet<String>,
}

impl CapabilityHandler {
    pub fn new() -> Self {
        let mut supported = HashSet::new();
        
        // Core capabilities
        supported.insert("message-tags".to_string());
        supported.insert("server-time".to_string());
        supported.insert("multi-prefix".to_string());
        supported.insert("sasl".to_string());
        
        // Extended capabilities
        supported.insert("account-notify".to_string());
        supported.insert("away-notify".to_string());
        supported.insert("extended-join".to_string());
        supported.insert("invite-notify".to_string());
        
        Self {
            version: 302,
            available: HashSet::new(),
            enabled: HashSet::new(),
            supported,
        }
    }
}
```

## ISUPPORT (005) Parsing

### Token Parser
```rust
pub struct ISupportParser {
    pub network: Option<String>,
    pub chantypes: String,
    pub prefix: HashMap<char, char>,
    pub chanmodes: ChannelModes,
    pub nicklen: usize,
    pub channellen: usize,
    // ... many more
}

impl ISupportParser {
    pub fn parse_token(&mut self, token: &str) {
        if let Some((key, value)) = token.split_once('=') {
            match key {
                "NETWORK" => self.network = Some(value.to_string()),
                "CHANTYPES" => self.chantypes = value.to_string(),
                "PREFIX" => self.parse_prefix(value),
                "CHANMODES" => self.parse_chanmodes(value),
                "NICKLEN" => self.nicklen = value.parse().unwrap_or(30),
                // ... handle other tokens
                _ => {}
            }
        }
    }
}
```

## Character Encoding

### UTF-8 Support
```rust
pub fn validate_utf8(input: &[u8]) -> Result<&str> {
    std::str::from_utf8(input)
        .map_err(|_| Error::InvalidEncoding)
}
```

### Fallback Encoding
```rust
pub fn decode_with_fallback(input: &[u8]) -> String {
    match std::str::from_utf8(input) {
        Ok(s) => s.to_string(),
        Err(_) => {
            // Try Windows-1252 or ISO-8859-1
            // Fall back to lossy UTF-8
            String::from_utf8_lossy(input).to_string()
        }
    }
}
```

## Rate Limiting

### Command Throttling
```rust
pub struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_update: Instant,
}

impl RateLimiter {
    pub fn can_send(&mut self) -> bool {
        self.update_tokens();
        
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
    
    fn update_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        
        self.tokens = (self.tokens + elapsed * self.refill_rate)
            .min(self.max_tokens);
        self.last_update = now;
    }
}
```

## Testing Compliance

### Protocol Test Suite
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_message() {
        let msg = parse("PRIVMSG #test :Hello, world!").unwrap();
        assert_eq!(msg.command, "PRIVMSG");
        assert_eq!(msg.params, vec!["#test", "Hello, world!"]);
    }
    
    #[test]
    fn test_parse_tagged_message() {
        let msg = parse("@time=2023-01-01T00:00:00Z PRIVMSG #test :Hi").unwrap();
        assert!(msg.tags.is_some());
        assert_eq!(msg.tags.unwrap().get("time"), 
                   Some(&"2023-01-01T00:00:00Z".to_string()));
    }
}
```