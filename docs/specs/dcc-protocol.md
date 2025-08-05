# DCC Protocol Specification

## Overview

Direct Client-to-Client (DCC) is a protocol extension for IRC that enables direct connections between users, bypassing the IRC server. This document specifies RustIRC's implementation of the DCC protocol, including file transfers, direct chat, and security considerations.

## Protocol Basics

### DCC Negotiation

DCC connections are initiated through CTCP (Client-To-Client Protocol) messages sent via the IRC server:

```
PRIVMSG target :\x01DCC type arguments\x01
```

Where:
- `target` is the recipient's nickname
- `type` is the DCC request type (SEND, CHAT, RESUME, ACCEPT)
- `arguments` are type-specific parameters

### Connection Types

DCC supports several connection methods:
1. **Active (Direct)**: Sender listens, receiver connects
2. **Passive (Reverse)**: Receiver listens, sender connects
3. **Server**: Both connect to a proxy server

## DCC SEND (File Transfer)

### Request Format

#### Active DCC SEND
```
DCC SEND filename address port size [token]
```

- `filename`: Name of the file (spaces converted to underscores)
- `address`: IP address as 32-bit integer (network byte order)
- `port`: TCP port number
- `size`: File size in bytes
- `token`: Optional unique identifier for passive DCC

#### Passive DCC SEND
```
DCC SEND filename 0 0 size token
```

The receiver responds with:
```
DCC SEND filename address port size token
```

### Implementation

```rust
pub struct DccSendRequest {
    pub filename: String,
    pub address: IpAddr,
    pub port: u16,
    pub size: u64,
    pub token: Option<String>,
    pub passive: bool,
}

impl DccSendRequest {
    /// Create active DCC SEND request
    pub fn active(filename: String, address: IpAddr, port: u16, size: u64) -> Self {
        Self {
            filename,
            address,
            port,
            size,
            token: None,
            passive: false,
        }
    }
    
    /// Create passive DCC SEND request
    pub fn passive(filename: String, size: u64) -> Self {
        let token = generate_token();
        Self {
            filename,
            address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: 0,
            size,
            token: Some(token),
            passive: true,
        }
    }
    
    /// Format as CTCP message
    pub fn to_ctcp(&self) -> String {
        if self.passive {
            format!("DCC SEND {} 0 0 {} {}", 
                sanitize_filename(&self.filename),
                self.size,
                self.token.as_ref().unwrap()
            )
        } else {
            format!("DCC SEND {} {} {} {}",
                sanitize_filename(&self.filename),
                ip_to_u32(&self.address),
                self.port,
                self.size
            )
        }
    }
}

fn sanitize_filename(filename: &str) -> String {
    filename.replace(' ', "_")
        .chars()
        .filter(|c| c.is_ascii() && !c.is_control())
        .collect()
}

fn ip_to_u32(ip: &IpAddr) -> u32 {
    match ip {
        IpAddr::V4(ipv4) => u32::from_be_bytes(ipv4.octets()),
        IpAddr::V6(_) => panic!("IPv6 not supported in classic DCC"),
    }
}
```

### Transfer Protocol

1. **Connection Establishment**
   - Active: Receiver connects to sender's IP:port
   - Passive: Sender connects to receiver's IP:port

2. **Data Transfer**
   - Sender transmits file data in chunks
   - Receiver acknowledges bytes received
   - Acknowledgments are 4-byte integers (network byte order)

3. **Transfer Flow**
```rust
pub async fn handle_dcc_send(mut stream: TcpStream, file: File, resume_pos: u64) -> Result<()> {
    let mut file = BufReader::new(file);
    file.seek(SeekFrom::Start(resume_pos))?;
    
    let mut total_sent = resume_pos;
    let mut total_acked = resume_pos;
    let mut buffer = vec![0u8; 8192];
    
    loop {
        // Read file chunk
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // EOF
        }
        
        // Send data
        stream.write_all(&buffer[..bytes_read]).await?;
        total_sent += bytes_read as u64;
        
        // Check for acknowledgments
        while let Ok(ack_bytes) = stream.try_read(&mut [0u8; 4]) {
            if ack_bytes == 4 {
                let acked = u32::from_be_bytes([
                    ack_bytes[0], ack_bytes[1], ack_bytes[2], ack_bytes[3]
                ]) as u64;
                total_acked = acked;
            }
        }
        
        // Verify acknowledgments
        if total_sent - total_acked > MAX_UNACKED_BYTES {
            // Wait for more acks before continuing
            wait_for_acks(&mut stream, total_sent, &mut total_acked).await?;
        }
    }
    
    // Wait for final acknowledgment
    wait_for_final_ack(&mut stream, total_sent).await?;
    
    Ok(())
}
```

## DCC RESUME

### Resume Negotiation

When a partial file exists, the receiver can request resumption:

```
DCC RESUME filename port position [token]
```

The sender accepts with:
```
DCC ACCEPT filename port position [token]
```

### Implementation

```rust
pub struct DccResumeRequest {
    pub filename: String,
    pub port: u16,
    pub position: u64,
    pub token: Option<String>,
}

impl DccResumeRequest {
    pub fn new(filename: String, port: u16, position: u64, token: Option<String>) -> Self {
        Self { filename, port, position, token }
    }
    
    pub fn to_ctcp(&self) -> String {
        match &self.token {
            Some(token) => format!("DCC RESUME {} {} {} {}", 
                sanitize_filename(&self.filename), self.port, self.position, token),
            None => format!("DCC RESUME {} {} {}", 
                sanitize_filename(&self.filename), self.port, self.position),
        }
    }
}

pub async fn handle_resume_request(
    transfer: &mut DccTransfer,
    resume_req: DccResumeRequest
) -> Result<()> {
    // Verify the request matches our transfer
    if transfer.filename != resume_req.filename {
        return Err(DccError::FilenameMismatch);
    }
    
    if transfer.port != resume_req.port && transfer.token != resume_req.token {
        return Err(DccError::InvalidResume);
    }
    
    // Validate resume position
    if resume_req.position > transfer.size {
        return Err(DccError::InvalidPosition);
    }
    
    transfer.resume_position = Some(resume_req.position);
    
    // Send ACCEPT response
    let accept_msg = format!("DCC ACCEPT {} {} {}",
        sanitize_filename(&transfer.filename),
        transfer.port,
        resume_req.position
    );
    
    Ok(())
}
```

## DCC CHAT

### Chat Request

```
DCC CHAT chat address port [token]
```

- `chat`: Literal string "chat"
- `address`: IP address as 32-bit integer
- `port`: TCP port number
- `token`: Optional for passive DCC

### Chat Protocol

```rust
pub struct DccChat {
    stream: TcpStream,
    buffer: String,
}

impl DccChat {
    pub async fn send_message(&mut self, message: &str) -> Result<()> {
        // DCC CHAT messages are terminated with \n (not \r\n)
        let formatted = format!("{}\n", message);
        self.stream.write_all(formatted.as_bytes()).await?;
        Ok(())
    }
    
    pub async fn receive_message(&mut self) -> Result<Option<String>> {
        let mut buf = vec![0u8; 1024];
        
        match self.stream.read(&mut buf).await {
            Ok(0) => Ok(None), // Connection closed
            Ok(n) => {
                self.buffer.push_str(&String::from_utf8_lossy(&buf[..n]));
                
                if let Some(newline_pos) = self.buffer.find('\n') {
                    let message = self.buffer[..newline_pos].to_string();
                    self.buffer.drain(..=newline_pos);
                    Ok(Some(message))
                } else {
                    Ok(None) // Incomplete message
                }
            }
            Err(e) => Err(e.into()),
        }
    }
}
```

## Security Considerations

### IP Address Disclosure

DCC reveals IP addresses. Mitigation strategies:

1. **IP Masking**: Use IRC bouncer or proxy
2. **Passive DCC**: Receiver's IP not revealed in request
3. **IPv6 Privacy**: Use temporary addresses

```rust
pub fn should_use_passive_dcc(config: &DccConfig, peer: &str) -> bool {
    // Use passive DCC if behind NAT or privacy mode enabled
    config.force_passive || 
    config.privacy_mode ||
    is_behind_nat() ||
    !is_trusted_peer(peer)
}
```

### File Transfer Security

```rust
pub struct SecureDccConfig {
    /// Maximum file size to accept (default: 2GB)
    pub max_file_size: u64,
    
    /// Allowed file extensions (empty = all allowed)
    pub allowed_extensions: HashSet<String>,
    
    /// Blocked file extensions
    pub blocked_extensions: HashSet<String>,
    
    /// Require TLS for DCC connections
    pub require_tls: bool,
    
    /// Verify file hashes after transfer
    pub verify_checksums: bool,
    
    /// Quarantine directory for received files
    pub quarantine_dir: Option<PathBuf>,
    
    /// Auto-accept from trusted users only
    pub trusted_users: HashSet<String>,
}

impl SecureDccConfig {
    pub fn validate_transfer(&self, request: &DccSendRequest, from: &str) -> Result<()> {
        // Check file size
        if request.size > self.max_file_size {
            return Err(DccError::FileTooLarge);
        }
        
        // Check file extension
        let ext = Path::new(&request.filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
            
        if !self.allowed_extensions.is_empty() && 
           !self.allowed_extensions.contains(ext) {
            return Err(DccError::ExtensionNotAllowed);
        }
        
        if self.blocked_extensions.contains(ext) {
            return Err(DccError::ExtensionBlocked);
        }
        
        // Check trusted users for auto-accept
        if !self.trusted_users.contains(from) {
            return Err(DccError::ManualAcceptRequired);
        }
        
        Ok(())
    }
}
```

### Connection Security

```rust
pub async fn secure_dcc_connect(
    address: SocketAddr,
    config: &DccConfig
) -> Result<DccStream> {
    let stream = TcpStream::connect(address).await?;
    
    if config.require_tls {
        // Upgrade to TLS
        let tls_config = create_tls_config()?;
        let tls_stream = tls_config.connect("dcc.local", stream).await?;
        Ok(DccStream::Tls(tls_stream))
    } else {
        Ok(DccStream::Plain(stream))
    }
}

pub enum DccStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}
```

## Advanced Features

### DCC Server

Some networks support DCC Server to bypass NAT:

```
DCC SEND filename 0 port size token S
```

The 'S' flag indicates server mode. Both parties connect to the DCC server.

### Fast DCC

Optimization for high-latency connections:
- Receiver doesn't acknowledge every packet
- Periodic acknowledgments only
- Final acknowledgment required

```rust
pub struct FastDccConfig {
    /// Acknowledge every N bytes (default: 1MB)
    pub ack_interval: u64,
    
    /// Maximum unacknowledged bytes (default: 10MB)
    pub max_unacked: u64,
}
```

### DCC XMIT

Extended DCC for additional features:
- Compression support
- Encryption
- Multiple files
- Directory transfers

```
DCC XMIT type filename address port size [options]
```

## Error Handling

### DCC-Specific Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum DccError {
    #[error("Connection refused by peer")]
    ConnectionRefused,
    
    #[error("Transfer timeout")]
    Timeout,
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid DCC request")]
    InvalidRequest,
    
    #[error("File size mismatch")]
    SizeMismatch,
    
    #[error("Checksum verification failed")]
    ChecksumFailed,
    
    #[error("Resume position invalid")]
    InvalidPosition,
    
    #[error("Port already in use")]
    PortInUse,
    
    #[error("Too many connections")]
    ConnectionLimit,
}
```

### Recovery Strategies

```rust
pub async fn transfer_with_retry(
    transfer: &mut DccTransfer,
    max_retries: u32
) -> Result<()> {
    let mut retries = 0;
    let mut last_position = transfer.resume_position.unwrap_or(0);
    
    loop {
        match execute_transfer(transfer).await {
            Ok(()) => return Ok(()),
            Err(e) if retries < max_retries => {
                retries += 1;
                
                // Exponential backoff
                let delay = Duration::from_secs(2u64.pow(retries));
                tokio::time::sleep(delay).await;
                
                // Update resume position
                transfer.resume_position = Some(last_position);
                
                warn!("Transfer failed, retrying ({}/{}): {}", 
                    retries, max_retries, e);
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Testing DCC Implementation

### Test Scenarios

```rust
#[cfg(test)]
mod dcc_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_dcc_send_negotiation() {
        let request = DccSendRequest::active(
            "test.txt".to_string(),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)),
            5000,
            1024
        );
        
        let ctcp = request.to_ctcp();
        assert_eq!(ctcp, "DCC SEND test.txt 3232235876 5000 1024");
    }
    
    #[tokio::test]
    async fn test_passive_dcc() {
        let request = DccSendRequest::passive("test.txt".to_string(), 1024);
        let ctcp = request.to_ctcp();
        
        assert!(ctcp.starts_with("DCC SEND test.txt 0 0 1024 "));
        assert!(request.token.is_some());
    }
    
    #[tokio::test]
    async fn test_resume_handling() {
        let mut transfer = create_test_transfer();
        let resume = DccResumeRequest::new(
            "test.txt".to_string(),
            5000,
            512,
            None
        );
        
        handle_resume_request(&mut transfer, resume).await.unwrap();
        assert_eq!(transfer.resume_position, Some(512));
    }
}
```

## Best Practices

1. **Always validate file transfers** before accepting
2. **Use passive DCC** when behind NAT
3. **Implement timeouts** for all operations
4. **Verify file integrity** with checksums
5. **Limit concurrent connections** to prevent DoS
6. **Sanitize filenames** to prevent path traversal
7. **Use TLS** for sensitive transfers
8. **Implement rate limiting** for large files

## Compatibility Notes

- Some clients use different byte orders for acknowledgments
- Filename encoding may vary (UTF-8 vs ASCII)
- Port 0 sometimes indicates passive DCC without token
- Some networks block DCC on certain ports
- IPv6 support varies widely