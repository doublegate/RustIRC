use anyhow::{Context, Result};
use rustls::pki_types::ServerName;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use tracing::{error, info, warn};

#[derive(Debug)]
struct IrcMessage {
    prefix: Option<String>,
    command: String,
    params: Vec<String>,
}

impl IrcMessage {
    fn parse(line: &str) -> Option<Self> {
        let mut chars = line.chars().peekable();
        let mut prefix = None;
        let mut command = String::new();
        let mut params = Vec::new();

        // Parse prefix
        if chars.peek() == Some(&':') {
            chars.next();
            let mut pfx = String::new();
            while let Some(ch) = chars.next() {
                if ch == ' ' {
                    break;
                }
                pfx.push(ch);
            }
            prefix = Some(pfx);
            // Skip whitespace
            while chars.peek() == Some(&' ') {
                chars.next();
            }
        }

        // Parse command
        while let Some(ch) = chars.next() {
            if ch == ' ' {
                break;
            }
            command.push(ch);
        }

        // Skip whitespace
        while chars.peek() == Some(&' ') {
            chars.next();
        }

        // Parse parameters
        while chars.peek().is_some() {
            if chars.peek() == Some(&':') {
                // Trailing parameter
                chars.next();
                params.push(chars.collect());
                break;
            } else {
                // Regular parameter
                let mut param = String::new();
                while let Some(ch) = chars.next() {
                    if ch == ' ' {
                        break;
                    }
                    param.push(ch);
                }
                if !param.is_empty() {
                    params.push(param);
                }
                // Skip whitespace
                while chars.peek() == Some(&' ') {
                    chars.next();
                }
            }
        }

        if command.is_empty() {
            None
        } else {
            Some(IrcMessage {
                prefix,
                command,
                params,
            })
        }
    }

    fn to_string(&self) -> String {
        let mut result = String::new();
        
        if let Some(ref prefix) = self.prefix {
            result.push(':');
            result.push_str(prefix);
            result.push(' ');
        }
        
        result.push_str(&self.command);
        
        for (i, param) in self.params.iter().enumerate() {
            result.push(' ');
            if i == self.params.len() - 1 && (param.contains(' ') || param.starts_with(':')) {
                result.push(':');
            }
            result.push_str(param);
        }
        
        result
    }
}

struct IrcConnection {
    reader: BufReader<tokio::io::ReadHalf<tokio_rustls::client::TlsStream<TcpStream>>>,
    writer: tokio::io::WriteHalf<tokio_rustls::client::TlsStream<TcpStream>>,
    server: String,
}

impl IrcConnection {
    async fn connect(server: &str, port: u16, use_tls: bool) -> Result<Self> {
        info!("Connecting to {}:{} (TLS: {})", server, port, use_tls);
        
        let stream = TcpStream::connect((server, port))
            .await
            .context("Failed to connect to server")?;

        if use_tls {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

            let config = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            let connector = TlsConnector::from(Arc::new(config));
            let server_name = ServerName::try_from(server.to_string())
                .context("Invalid server name")?;

            let tls_stream = connector.connect(server_name, stream).await?;
            let (reader, writer) = tokio::io::split(tls_stream);
            
            Ok(IrcConnection {
                reader: BufReader::new(reader),
                writer,
                server: server.to_string(),
            })
        } else {
            // For non-TLS, we'd need a different type
            // This is simplified for the prototype
            unimplemented!("Non-TLS connections not implemented in this prototype")
        }
    }

    async fn send_message(&mut self, msg: &IrcMessage) -> Result<()> {
        let line = format!("{}\r\n", msg.to_string());
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.flush().await?;
        info!(">>> {}", msg.to_string());
        Ok(())
    }

    async fn read_message(&mut self) -> Result<Option<IrcMessage>> {
        let mut line = String::new();
        match timeout(Duration::from_secs(300), self.reader.read_line(&mut line)).await {
            Ok(Ok(0)) => Ok(None), // Connection closed
            Ok(Ok(_)) => {
                line = line.trim_end().to_string();
                if !line.is_empty() {
                    info!("<<< {}", line);
                    Ok(IrcMessage::parse(&line))
                } else {
                    Ok(Some(IrcMessage {
                        prefix: None,
                        command: String::new(),
                        params: vec![],
                    }))
                }
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => {
                warn!("Read timeout - sending PING");
                Ok(Some(IrcMessage {
                    prefix: None,
                    command: "PING".to_string(),
                    params: vec![self.server.clone()],
                }))
            }
        }
    }

    async fn register(&mut self, nick: &str, user: &str, realname: &str) -> Result<()> {
        // Send CAP LS for IRCv3
        self.send_message(&IrcMessage {
            prefix: None,
            command: "CAP".to_string(),
            params: vec!["LS".to_string(), "302".to_string()],
        }).await?;

        // Send NICK
        self.send_message(&IrcMessage {
            prefix: None,
            command: "NICK".to_string(),
            params: vec![nick.to_string()],
        }).await?;

        // Send USER
        self.send_message(&IrcMessage {
            prefix: None,
            command: "USER".to_string(),
            params: vec![
                user.to_string(),
                "0".to_string(),
                "*".to_string(),
                realname.to_string(),
            ],
        }).await?;

        Ok(())
    }
}

async fn benchmark_parser() -> Result<()> {
    info!("Starting parser benchmark...");
    
    let test_messages = vec![
        ":server.example.com 001 nick :Welcome to the Internet Relay Chat Network",
        "PING :server.example.com",
        ":nick!user@host PRIVMSG #channel :Hello, world!",
        ":server.example.com 353 nick = #channel :@op +voice regular",
        ":nick!user@host JOIN #channel",
        "CAP * LS :multi-prefix extended-join sasl",
        ":server.example.com 005 nick CHANTYPES=# EXCEPTS INVEX CHANMODES=eIbq,k,flj,CFLMPQScgimnprstz :are supported",
    ];

    let iterations = 1_000_000;
    let start = Instant::now();

    for _ in 0..iterations {
        for msg in &test_messages {
            let _ = IrcMessage::parse(msg);
        }
    }

    let elapsed = start.elapsed();
    let messages_per_sec = (iterations * test_messages.len()) as f64 / elapsed.as_secs_f64();
    
    info!(
        "Parsed {} messages in {:?} ({:.0} messages/sec)",
        iterations * test_messages.len(),
        elapsed,
        messages_per_sec
    );

    Ok(())
}

async fn test_multi_connection() -> Result<()> {
    info!("Testing multiple concurrent connections...");
    
    let servers = vec![
        ("irc.libera.chat", 6697, true),
        ("irc.oftc.net", 6697, true),
        // Add more servers as needed
    ];

    let mut handles = vec![];

    for (server, port, use_tls) in servers {
        let handle = tokio::spawn(async move {
            match IrcConnection::connect(server, port, use_tls).await {
                Ok(mut conn) => {
                    info!("Successfully connected to {}", server);
                    
                    // Try to register
                    if let Err(e) = conn.register(
                        &format!("rustirctest{}", rand::random::<u16>()),
                        "rustirc",
                        "RustIRC Test Client"
                    ).await {
                        error!("Failed to register on {}: {}", server, e);
                    }
                    
                    // Read a few messages
                    for _ in 0..5 {
                        match conn.read_message().await {
                            Ok(Some(msg)) => {
                                info!("{}: {:?}", server, msg);
                            }
                            Ok(None) => {
                                info!("{}: Connection closed", server);
                                break;
                            }
                            Err(e) => {
                                error!("{}: Read error: {}", server, e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to {}: {}", server, e);
                }
            }
        });
        
        handles.push(handle);
    }

    // Wait for all connections
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rustirc_network_prototype=info")
        .init();

    info!("RustIRC Network Prototype");
    info!("=========================");
    
    // Run parser benchmark
    benchmark_parser().await?;
    
    // Test multi-connection handling
    // Note: This will actually try to connect to IRC servers
    // Comment out if you don't want to connect
    // test_multi_connection().await?;

    info!("Network prototype testing complete!");
    
    Ok(())
}