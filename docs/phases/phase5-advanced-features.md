# Phase 5: Advanced Features

**Duration**: 4-6 weeks  
**Goal**: Implement DCC protocol, complete IRCv3 support, and advanced features

## Overview

Phase 5 focuses on implementing the advanced features that distinguish a full-featured IRC client from a basic one. This includes the complete DCC protocol suite for direct file transfers and chats, comprehensive IRCv3 extension support, and enhanced security features. These features are critical for feature parity with established clients.

## Objectives

1. Implement complete DCC protocol suite
2. Add all remaining IRCv3 extensions
3. Enhance SASL with additional mechanisms
4. Implement proxy support
5. Add native OS notifications
6. Create advanced UI features

## DCC Implementation

### DCC Architecture
```rust
// rustirc-dcc/src/lib.rs
pub struct DccManager {
    connections: HashMap<DccId, DccConnection>,
    transfers: HashMap<TransferId, DccTransfer>,
    config: DccConfig,
    port_range: RangeInclusive<u16>,
    upnp_client: Option<UpnpClient>,
}

pub enum DccConnection {
    Chat(DccChat),
    Send(DccSend),
    Get(DccGet),
}

pub struct DccConfig {
    pub auto_accept: DccAutoAccept,
    pub download_dir: PathBuf,
    pub port_range: RangeInclusive<u16>,
    pub use_upnp: bool,
    pub passive_dcc: bool,
    pub resume_support: bool,
}
```

### DCC CHAT Implementation
```rust
// rustirc-dcc/src/chat.rs
pub struct DccChat {
    id: DccId,
    peer_nick: String,
    peer_address: SocketAddr,
    socket: TcpStream,
    state: DccChatState,
    encryption: Option<DccEncryption>,
}

impl DccChat {
    pub async fn initiate(peer: &str, our_ip: IpAddr) -> Result<Self> {
        // Bind to random port in range
        let listener = TcpListener::bind((our_ip, 0)).await?;
        let port = listener.local_addr()?.port();
        
        // Send CTCP DCC CHAT request
        let request = format!(
            "\x01DCC CHAT chat {} {}\x01",
            ip_to_u32(our_ip),
            port
        );
        
        // Wait for connection
        let (socket, peer_addr) = listener.accept().await?;
        
        Ok(Self {
            socket,
            peer_address: peer_addr,
            // ... initialize other fields
        })
    }
    
    pub async fn accept(offer: DccOffer) -> Result<Self> {
        let addr = SocketAddr::new(
            u32_to_ip(offer.address),
            offer.port
        );
        
        let socket = TcpStream::connect(addr).await?;
        
        Ok(Self {
            socket,
            peer_address: addr,
            // ... initialize other fields
        })
    }
}
```

### DCC SEND/GET Implementation
```rust
// rustirc-dcc/src/transfer.rs
pub struct DccTransfer {
    id: TransferId,
    file_path: PathBuf,
    file_size: u64,
    transferred: AtomicU64,
    state: Arc<Mutex<TransferState>>,
    speed_tracker: SpeedTracker,
}

pub struct DccSend {
    transfer: Arc<DccTransfer>,
    listener: Option<TcpListener>,
    connection: Option<TcpStream>,
}

impl DccSend {
    pub async fn send_file(
        peer: &str,
        file_path: &Path,
        passive: bool
    ) -> Result<Self> {
        let metadata = tokio::fs::metadata(file_path).await?;
        let file_size = metadata.len();
        
        if passive {
            // Passive DCC - we connect to them
            self.initiate_passive(peer, file_path, file_size).await
        } else {
            // Active DCC - they connect to us
            self.initiate_active(peer, file_path, file_size).await
        }
    }
    
    async fn transfer_loop(&mut self) -> Result<()> {
        let mut file = tokio::fs::File::open(&self.transfer.file_path).await?;
        let mut buffer = vec![0u8; 8192];
        let mut ack_buffer = [0u8; 4];
        
        while self.transfer.transferred.load(Ordering::Relaxed) < self.transfer.file_size {
            // Read chunk from file
            let n = file.read(&mut buffer).await?;
            if n == 0 { break; }
            
            // Send to peer
            self.connection.write_all(&buffer[..n]).await?;
            
            // Wait for acknowledgment (4-byte position)
            self.connection.read_exact(&mut ack_buffer).await?;
            let acked = u32::from_be_bytes(ack_buffer);
            
            // Update progress
            self.transfer.transferred.store(acked as u64, Ordering::Relaxed);
            self.speed_tracker.update(n);
        }
        
        Ok(())
    }
}
```

### DCC RESUME Support
```rust
// rustirc-dcc/src/resume.rs
pub struct DccResume {
    pub file_name: String,
    pub port: u16,
    pub position: u64,
}

impl DccManager {
    pub async fn handle_resume_request(&mut self, resume: DccResume) -> Result<()> {
        // Find matching transfer
        let transfer = self.find_transfer(&resume.file_name, resume.port)?;
        
        // Verify we can resume from position
        if resume.position <= transfer.file_size {
            // Send ACCEPT
            let accept = format!(
                "\x01DCC ACCEPT {} {} {}\x01",
                resume.file_name,
                resume.port,
                resume.position
            );
            
            // Seek to position
            transfer.seek_to(resume.position).await?;
            
            Ok(())
        } else {
            Err(Error::InvalidResumePosition)
        }
    }
}
```

### Reverse/Passive DCC
```rust
impl DccManager {
    pub async fn initiate_passive_send(
        &mut self,
        peer: &str,
        file_path: &Path
    ) -> Result<()> {
        // Send with port 0 to indicate passive
        let request = format!(
            "\x01DCC SEND {} {} 0 {}\x01",
            file_path.file_name().unwrap().to_str().unwrap(),
            0, // IP address 0 for passive
            file_size
        );
        
        // Wait for reverse connection offer
        // Peer will send us their IP and port
    }
}
```

## IRCv3 Extensions

### Complete IRCv3 Support
```rust
// rustirc-protocol/src/ircv3/mod.rs
pub struct IrcV3Handler {
    capabilities: CapabilityHandler,
    extensions: ExtensionRegistry,
}

pub struct ExtensionRegistry {
    message_tags: MessageTagHandler,
    labeled_response: LabeledResponseHandler,
    echo_message: EchoMessageHandler,
    batch: BatchHandler,
    chathistory: ChatHistoryHandler,
    monitor: MonitorHandler,
    sasl_v3_2: SaslV32Handler,
}
```

### Message Tags Handler
```rust
// rustirc-protocol/src/ircv3/tags.rs
pub struct MessageTagHandler {
    supported_tags: HashSet<String>,
    client_only_tags: HashSet<String>,
}

impl MessageTagHandler {
    pub fn new() -> Self {
        let mut handler = Self {
            supported_tags: HashSet::new(),
            client_only_tags: HashSet::new(),
        };
        
        // Standard tags
        handler.supported_tags.insert("time".to_string());
        handler.supported_tags.insert("msgid".to_string());
        handler.supported_tags.insert("account".to_string());
        handler.supported_tags.insert("batch".to_string());
        handler.supported_tags.insert("label".to_string());
        handler.supported_tags.insert("reaction".to_string());
        handler.supported_tags.insert("reply-to".to_string());
        
        // Client-only tags
        handler.client_only_tags.insert("+draft/typing".to_string());
        handler.client_only_tags.insert("+draft/react".to_string());
        
        handler
    }
}
```

### CHATHISTORY Implementation
```rust
// rustirc-protocol/src/ircv3/chathistory.rs
pub struct ChatHistoryHandler {
    pending_requests: HashMap<String, HistoryRequest>,
    max_messages_per_request: usize,
}

pub enum HistoryRequest {
    Before { target: String, timestamp: DateTime<Utc>, limit: usize },
    After { target: String, timestamp: DateTime<Utc>, limit: usize },
    Between { target: String, start: DateTime<Utc>, end: DateTime<Utc> },
    Around { target: String, timestamp: DateTime<Utc>, limit: usize },
}

impl ChatHistoryHandler {
    pub fn request_history(&mut self, req: HistoryRequest) -> String {
        let label = generate_label();
        self.pending_requests.insert(label.clone(), req.clone());
        
        match req {
            HistoryRequest::Before { target, timestamp, limit } => {
                format!("CHATHISTORY BEFORE {} {} {}", 
                    target, 
                    timestamp.to_rfc3339(), 
                    limit
                )
            }
            // ... other variants
        }
    }
}
```

### Batch Message Handling
```rust
// rustirc-protocol/src/ircv3/batch.rs
pub struct BatchHandler {
    active_batches: HashMap<String, Batch>,
    completed_batches: VecDeque<CompletedBatch>,
}

pub struct Batch {
    id: String,
    batch_type: String,
    params: Vec<String>,
    messages: Vec<IrcMessage>,
    started: Instant,
}

impl BatchHandler {
    pub fn handle_batch(&mut self, msg: &IrcMessage) -> Result<BatchResult> {
        let batch_tag = msg.tags.get("batch")?;
        
        if batch_tag.starts_with('+') {
            // Start of batch
            let id = batch_tag[1..].to_string();
            self.start_batch(id, msg)
        } else if batch_tag.starts_with('-') {
            // End of batch
            let id = batch_tag[1..].to_string();
            self.end_batch(id)
        } else {
            // Message in batch
            self.add_to_batch(batch_tag.clone(), msg)
        }
    }
}
```

## Enhanced SASL

### SCRAM-SHA-256 Implementation
```rust
// rustirc-auth/src/sasl/scram.rs
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};

pub struct ScramSha256 {
    username: String,
    password: String,
    client_nonce: String,
    state: ScramState,
}

enum ScramState {
    Initial,
    ClientFirst,
    ServerFirst { 
        server_nonce: String,
        salt: Vec<u8>,
        iterations: u32,
    },
    ClientFinal,
    Complete,
}

impl ScramSha256 {
    pub fn client_first_message(&mut self) -> String {
        self.client_nonce = generate_nonce();
        let msg = format!("n,,n={},r={}", 
            self.username, 
            self.client_nonce
        );
        self.state = ScramState::ClientFirst;
        base64::encode(msg)
    }
    
    pub fn handle_server_first(&mut self, data: &str) -> Result<String> {
        let decoded = base64::decode(data)?;
        let msg = String::from_utf8(decoded)?;
        
        // Parse server response
        let parts = parse_scram_attributes(&msg);
        let server_nonce = parts.get("r").ok_or(Error::InvalidResponse)?;
        let salt = base64::decode(parts.get("s").ok_or(Error::InvalidResponse)?)?;
        let iterations = parts.get("i").ok_or(Error::InvalidResponse)?.parse()?;
        
        // Calculate proof
        let salted_password = pbkdf2_hmac_sha256(&self.password, &salt, iterations);
        let client_key = hmac_sha256(&salted_password, b"Client Key");
        let stored_key = sha256(&client_key);
        
        // ... rest of SCRAM calculation
        
        Ok(base64::encode(client_final_message))
    }
}
```

### SASL EXTERNAL (Certificate Auth)
```rust
// rustirc-auth/src/sasl/external.rs
pub struct SaslExternal {
    client_cert: Option<Certificate>,
}

impl SaslExternal {
    pub fn new(cert_path: Option<&Path>) -> Result<Self> {
        let client_cert = if let Some(path) = cert_path {
            Some(load_certificate(path)?)
        } else {
            None
        };
        
        Ok(Self { client_cert })
    }
    
    pub fn authenticate(&self) -> Result<String> {
        // EXTERNAL just sends an empty response
        // The actual auth is done via TLS client cert
        Ok(base64::encode(""))
    }
}
```

## Proxy Support

### SOCKS5 Proxy
```rust
// rustirc-network/src/proxy/socks5.rs
pub struct Socks5Proxy {
    proxy_addr: SocketAddr,
    auth: Option<Socks5Auth>,
}

impl Socks5Proxy {
    pub async fn connect(&self, target: &str, port: u16) -> Result<TcpStream> {
        let mut stream = TcpStream::connect(&self.proxy_addr).await?;
        
        // SOCKS5 handshake
        stream.write_all(&[0x05, 0x01, 0x00]).await?; // Version, 1 method, no auth
        
        let mut response = [0u8; 2];
        stream.read_exact(&mut response).await?;
        
        if response[0] != 0x05 || response[1] != 0x00 {
            return Err(Error::ProxyHandshakeFailed);
        }
        
        // Connection request
        let mut request = vec![0x05, 0x01, 0x00, 0x03]; // CONNECT, reserved, domain name
        request.push(target.len() as u8);
        request.extend_from_slice(target.as_bytes());
        request.extend_from_slice(&port.to_be_bytes());
        
        stream.write_all(&request).await?;
        
        // Read response
        let mut response = [0u8; 10];
        stream.read_exact(&mut response).await?;
        
        if response[1] != 0x00 {
            return Err(Error::ProxyConnectionFailed(response[1]));
        }
        
        Ok(stream)
    }
}
```

### HTTP CONNECT Proxy
```rust
// rustirc-network/src/proxy/http.rs
pub struct HttpProxy {
    proxy_url: Url,
    auth: Option<HttpAuth>,
}

impl HttpProxy {
    pub async fn connect(&self, target: &str, port: u16) -> Result<TcpStream> {
        let proxy_addr = (self.proxy_url.host_str().unwrap(), self.proxy_url.port().unwrap_or(8080));
        let mut stream = TcpStream::connect(proxy_addr).await?;
        
        // Send CONNECT request
        let mut request = format!("CONNECT {}:{} HTTP/1.1\r\n", target, port);
        request.push_str(&format!("Host: {}:{}\r\n", target, port));
        
        if let Some(auth) = &self.auth {
            let credentials = base64::encode(format!("{}:{}", auth.username, auth.password));
            request.push_str(&format!("Proxy-Authorization: Basic {}\r\n", credentials));
        }
        
        request.push_str("\r\n");
        stream.write_all(request.as_bytes()).await?;
        
        // Read response
        let mut response = String::new();
        let mut reader = BufReader::new(&mut stream);
        reader.read_line(&mut response).await?;
        
        if !response.contains("200") {
            return Err(Error::ProxyConnectionFailed);
        }
        
        // Skip headers
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            if line == "\r\n" { break; }
        }
        
        Ok(stream)
    }
}
```

## Native Notifications

### Cross-Platform Notifications
```rust
// rustirc-notifications/src/lib.rs
pub struct NotificationManager {
    #[cfg(target_os = "windows")]
    toast: WindowsToast,
    #[cfg(target_os = "macos")]
    center: MacNotificationCenter,
    #[cfg(target_os = "linux")]
    dbus: DbusNotifier,
    
    rules: NotificationRules,
    history: NotificationHistory,
}

pub struct Notification {
    pub title: String,
    pub body: String,
    pub icon: Option<Icon>,
    pub sound: Option<Sound>,
    pub actions: Vec<NotificationAction>,
    pub timeout: Option<Duration>,
}

impl NotificationManager {
    pub async fn show(&mut self, notification: Notification) -> Result<NotificationId> {
        // Check rules
        if !self.rules.should_notify(&notification) {
            return Ok(NotificationId::default());
        }
        
        // Platform-specific implementation
        #[cfg(target_os = "windows")]
        {
            self.toast.show(notification).await
        }
        
        #[cfg(target_os = "macos")]
        {
            self.center.show(notification).await
        }
        
        #[cfg(target_os = "linux")]
        {
            self.dbus.show(notification).await
        }
    }
}
```

### Notification Rules
```rust
// rustirc-notifications/src/rules.rs
pub struct NotificationRules {
    pub highlight_words: Vec<String>,
    pub ignore_nicks: HashSet<String>,
    pub ignore_channels: HashSet<String>,
    pub quiet_hours: Option<QuietHours>,
    pub notification_types: NotificationTypes,
}

#[derive(Debug, Clone)]
pub struct NotificationTypes {
    pub private_messages: bool,
    pub highlights: bool,
    pub channel_messages: bool,
    pub joins_parts: bool,
    pub connection_events: bool,
}

impl NotificationRules {
    pub fn should_notify(&self, notification: &Notification) -> bool {
        // Check quiet hours
        if let Some(quiet) = &self.quiet_hours {
            if quiet.is_active() {
                return false;
            }
        }
        
        // Check ignore lists
        // Check notification type settings
        // etc.
        
        true
    }
}
```

## Advanced UI Features

### Multi-Window Support
```rust
// rustirc-gui/src/windows/mod.rs
pub struct WindowManager {
    windows: HashMap<WindowId, Window>,
    main_window: WindowId,
}

pub enum Window {
    Main(MainWindow),
    Channel(ChannelWindow),
    Private(PrivateWindow),
    DccChat(DccChatWindow),
}

impl WindowManager {
    pub fn detach_tab(&mut self, tab_id: TabId) -> Result<WindowId> {
        // Create new window with the tab
        let window = Window::Channel(ChannelWindow::new(tab_id));
        let window_id = WindowId::new();
        
        self.windows.insert(window_id, window);
        
        Ok(window_id)
    }
}
```

### Advanced Search
```rust
// rustirc-gui/src/search/mod.rs
pub struct SearchEngine {
    index: SearchIndex,
    query_parser: QueryParser,
}

pub struct SearchQuery {
    pub text: Option<String>,
    pub from: Option<String>,
    pub channel: Option<String>,
    pub date_range: Option<DateRange>,
    pub has_url: Option<bool>,
    pub regex: Option<Regex>,
}

impl SearchEngine {
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let parsed = self.query_parser.parse(query)?;
        
        let results = self.index.search(parsed).await?;
        
        // Rank and sort results
        let ranked = self.rank_results(results);
        
        Ok(ranked)
    }
}
```

## Testing

### DCC Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dcc_send() {
        let manager = DccManager::new(DccConfig::default());
        
        // Create test file
        let file_path = temp_dir().join("test.txt");
        std::fs::write(&file_path, b"Hello, DCC!").unwrap();
        
        // Initiate send
        let send = manager.send_file("testnick", &file_path).await.unwrap();
        
        // Simulate accept and transfer
        // ...
    }
}
```

## Deliverables

By the end of Phase 5:

1. **Complete DCC Support**
   - CHAT, SEND, GET implemented
   - RESUME functionality
   - Passive DCC support
   - UPnP integration

2. **Full IRCv3 Compliance**
   - All standard capabilities
   - Message tags support
   - CHATHISTORY implementation
   - Batch message handling

3. **Enhanced Security**
   - SCRAM-SHA-256 SASL
   - Certificate authentication
   - Proxy support (SOCKS5, HTTP)

4. **Platform Integration**
   - Native notifications
   - System tray support
   - OS-specific features

## Success Criteria

Phase 5 is complete when:
- [ ] Can transfer files via DCC successfully
- [ ] DCC resume works reliably
- [ ] All IRCv3 3.2 specs implemented
- [ ] SASL mechanisms functional
- [ ] Proxy connections work
- [ ] Notifications appear natively

## Next Phase

With advanced features complete, Phase 6 will focus on comprehensive testing, performance optimization, and preparing for production release.