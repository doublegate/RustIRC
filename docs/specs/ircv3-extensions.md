# IRCv3 Extensions Specification

## Overview

RustIRC implements comprehensive support for IRCv3 extensions, providing modern IRC features while maintaining backward compatibility. This document details our implementation of each IRCv3 capability and extension.

## Capability Negotiation

### CAP Protocol
The foundation of IRCv3 support, enabling feature discovery and negotiation.

```
Client: CAP LS 302
Server: CAP * LS * :multi-prefix extended-join account-notify batch invite-notify tls
Server: CAP * LS * :cap-notify server-time example.com/dummy-cap=dummyvalue example.com/second-dummy-cap
Server: CAP * LS :userhost-in-names sasl=EXTERNAL,PLAIN,ECDSA-NIST256P-CHALLENGE,SCRAM-SHA-256 message-tags account-tag
Client: CAP REQ :multi-prefix message-tags server-time account-notify
Server: CAP * ACK :multi-prefix message-tags server-time account-notify
Client: CAP END
```

### Implementation
```rust
pub struct CapabilityNegotiator {
    version: u16, // 302 or higher
    available: HashMap<String, Option<String>>,
    enabled: HashSet<String>,
    requested: HashSet<String>,
}

impl CapabilityNegotiator {
    pub fn negotiate(&mut self, available: Vec<Capability>) -> Vec<String> {
        let mut to_request = Vec::new();
        
        for cap in available {
            if self.is_supported(&cap) && self.should_enable(&cap) {
                to_request.push(cap.name);
            }
        }
        
        to_request
    }
}
```

## Core Capabilities

### message-tags
Enables tagged messages with metadata.

**Tag Format**: `@key=value;key2=value2`

**Escaping Rules**:
- `\s` → space
- `\n` → newline  
- `\r` → carriage return
- `\:` → semicolon
- `\\` → backslash

**Implementation**:
```rust
pub fn parse_tags(input: &str) -> HashMap<String, String> {
    let mut tags = HashMap::new();
    
    for pair in input.split(';') {
        if let Some((key, value)) = pair.split_once('=') {
            tags.insert(
                key.to_string(),
                unescape_tag_value(value)
            );
        } else {
            tags.insert(pair.to_string(), String::new());
        }
    }
    
    tags
}
```

### server-time
Adds accurate timestamps to messages.

**Tag**: `time=2011-10-19T16:40:51.620Z`

**Usage**:
- Historical message playback
- Accurate message ordering
- Bouncer integration

### account-notify
Notifies when users log in/out of services.

**Messages**:
```
:nick!user@host ACCOUNT accountname
:nick!user@host ACCOUNT *
```

### extended-join
Provides account info in JOIN messages.

**Format**: `:nick!user@host JOIN #channel accountname :Real Name`

### away-notify
Real-time away status updates.

**Messages**:
```
:nick!user@host AWAY :Gone to lunch
:nick!user@host AWAY
```

## Advanced Capabilities

### batch
Groups related messages together.

**Format**:
```
BATCH +sxtUfAeXBgNoD netsplit irc.example.com
:irc.example.com QUIT :irc.example.com irc2.example.com
:irc.example.com QUIT :irc.example.com irc2.example.com
BATCH -sxtUfAeXBgNoD
```

**Implementation**:
```rust
pub struct BatchManager {
    active_batches: HashMap<String, Batch>,
}

pub struct Batch {
    id: String,
    batch_type: String,
    params: Vec<String>,
    messages: Vec<IrcMessage>,
}

impl BatchManager {
    pub fn process_message(&mut self, msg: &IrcMessage) -> BatchResult {
        if let Some(batch_tag) = msg.tags.get("batch") {
            // Add to existing batch
            self.active_batches
                .get_mut(batch_tag)
                .unwrap()
                .messages.push(msg.clone());
            BatchResult::Buffered
        } else if msg.command == "BATCH" {
            self.handle_batch_command(msg)
        } else {
            BatchResult::Immediate(msg.clone())
        }
    }
}
```

### labeled-response
Correlates commands with their responses.

**Usage**:
```
@label=pQraCjj82e PRIVMSG #channel :Hello
@label=pQraCjj82e :server BATCH +labeled-response
@batch=labeled-response :nick!user@host PRIVMSG #channel :Hello
@batch=labeled-response BATCH -labeled-response
```

### echo-message
Echoes sent messages back to the client.

**Purpose**:
- Confirm message delivery
- Get server-assigned message ID
- Synchronize multiple clients

### CHATHISTORY
Query message history from the server.

**Commands**:
```
CHATHISTORY BEFORE #channel timestamp=2019-01-01T00:00:00.000Z 100
CHATHISTORY AFTER #channel timestamp=2019-01-01T00:00:00.000Z 100
CHATHISTORY LATEST #channel * 100
CHATHISTORY BETWEEN #channel timestamp1 timestamp2 100
```

**Implementation**:
```rust
pub enum HistoryQuery {
    Before { target: String, timestamp: DateTime<Utc>, limit: usize },
    After { target: String, timestamp: DateTime<Utc>, limit: usize },
    Latest { target: String, limit: usize },
    Between { target: String, start: DateTime<Utc>, end: DateTime<Utc>, limit: usize },
}

impl IrcClient {
    pub async fn query_history(&mut self, query: HistoryQuery) -> Result<Vec<Message>> {
        let label = self.generate_label();
        let command = format_history_command(query);
        
        self.send_labeled(label.clone(), command).await?;
        self.await_labeled_response(label).await
    }
}
```

## SASL Extensions

### SASL 3.2
Enhanced SASL with better mechanism negotiation.

**Mechanism Advertisement**: `CAP * LS :sasl=PLAIN,EXTERNAL,SCRAM-SHA-256`

### Authentication Flow
```rust
pub async fn authenticate(&mut self, mechanism: &str, credentials: Credentials) -> Result<()> {
    // Request SASL capability
    self.send("CAP REQ :sasl").await?;
    
    // Start authentication
    self.send(&format!("AUTHENTICATE {}", mechanism)).await?;
    
    match mechanism {
        "PLAIN" => self.auth_plain(credentials).await,
        "EXTERNAL" => self.auth_external().await,
        "SCRAM-SHA-256" => self.auth_scram(credentials).await,
        _ => Err(Error::UnsupportedMechanism),
    }
}
```

## Metadata Extensions

### metadata
Query and set user/channel metadata.

**Commands**:
```
METADATA * LIST
METADATA #channel GET topic-setter
METADATA nick SET mood :feeling great!
```

### monitor
Efficient online status tracking.

**Commands**:
```
MONITOR + nick1,nick2,nick3
MONITOR - nick1
MONITOR C
MONITOR L
MONITOR S
```

**Implementation**:
```rust
pub struct MonitorList {
    targets: HashSet<String>,
    online: HashMap<String, UserInfo>,
}

impl MonitorList {
    pub fn add(&mut self, nicks: Vec<String>) -> String {
        for nick in nicks {
            self.targets.insert(nick.clone());
        }
        format!("MONITOR + {}", self.targets.iter().join(","))
    }
}
```

## Message Extensions

### multiline
Send messages spanning multiple lines.

**Format**:
```
@draft/multiline-concat BATCH +123 draft/multiline #channel
@batch=123 PRIVMSG #channel :This is a
@batch=123 PRIVMSG #channel :multiline
@batch=123 PRIVMSG #channel :message
BATCH -123
```

### reply
Reference previous messages.

**Tag**: `+draft/reply=msgid`

### react
React to messages with emoji.

**Tag**: `+draft/react=👍`

## Client-Only Tags

### typing
Typing status notifications.

**Tag**: `+draft/typing=active`

**Values**: `active`, `paused`, `done`

### read-marker
Track read status across clients.

**Tag**: `+draft/read-marker=timestamp`

## Implementation Guidelines

### Tag Handling
```rust
pub struct MessageTags {
    server_tags: HashMap<String, String>,
    client_tags: HashMap<String, String>,
}

impl MessageTags {
    pub fn parse(input: &str) -> Self {
        let mut server_tags = HashMap::new();
        let mut client_tags = HashMap::new();
        
        for tag in input.split(';') {
            if let Some((key, value)) = tag.split_once('=') {
                if key.starts_with('+') {
                    client_tags.insert(key[1..].to_string(), value.to_string());
                } else {
                    server_tags.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        Self { server_tags, client_tags }
    }
}
```

### Capability Dependencies
```rust
const CAPABILITY_DEPS: &[(&str, &[&str])] = &[
    ("account-tag", &["message-tags"]),
    ("batch", &["message-tags"]),
    ("labeled-response", &["message-tags", "batch"]),
    ("chathistory", &["message-tags", "batch", "server-time"]),
];
```

### Feature Detection
```rust
impl IrcClient {
    pub fn supports(&self, feature: &str) -> bool {
        self.enabled_caps.contains(feature)
    }
    
    pub fn supports_all(&self, features: &[&str]) -> bool {
        features.iter().all(|&f| self.supports(f))
    }
}
```

## Best Practices

### Progressive Enhancement
- Always check capability support
- Provide fallbacks for missing features
- Handle partial implementations

### Performance Considerations
- Batch message processing
- Efficient tag parsing
- Lazy history loading

### Compatibility
- Support CAP 302 as minimum
- Handle vendor-specific extensions
- Graceful degradation

## Testing IRCv3 Support

### Test Servers
- testnet.oragono.io - Full IRCv3 support
- irc.ergo.chat - Modern IRCv3 implementation
- testnet.inspircd.org - Widespread server

### Compliance Testing
```rust
#[cfg(test)]
mod ircv3_tests {
    #[test]
    fn test_all_capabilities() {
        let caps = vec![
            "message-tags", "server-time", "account-notify",
            "away-notify", "batch", "labeled-response",
            "echo-message", "sasl", "multi-prefix",
        ];
        
        for cap in caps {
            assert!(test_capability_support(cap));
        }
    }
}
```