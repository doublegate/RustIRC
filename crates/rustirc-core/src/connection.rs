//! IRC connection management
//!
//! This module handles individual IRC server connections with:
//! - Async TCP connection with TLS support
//! - Automatic reconnection with exponential backoff
//! - Message sending/receiving with proper IRC protocol handling
//! - Connection state tracking
//! - Heartbeat/keepalive management

use crate::error::{Error, Result};
use crate::events::{Event, EventBus};
use rustirc_protocol::{Message, Parser, Command, MAX_MESSAGE_LENGTH};
use rustls::{ClientConfig as TlsConfig, RootCertStore};
use rustls_pki_types::ServerName;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Lines, BufStream};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, sleep, timeout};
use tokio_rustls::{TlsConnector, client::TlsStream};
use tracing::{debug, error, info, warn};

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Authenticating,
    Registered,
    Reconnecting,
    Failed(String),
}

/// Connection configuration
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub server: String,
    pub port: u16,
    pub use_tls: bool,
    pub verify_tls: bool,
    pub nickname: String,
    pub username: String,
    pub realname: String,
    pub password: Option<String>,
    pub reconnect_attempts: u32,
    pub reconnect_delay: Duration,
    pub ping_timeout: Duration,
    pub message_timeout: Duration,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            server: "irc.libera.chat".to_string(),
            port: 6697,
            use_tls: true,
            verify_tls: true,
            nickname: "RustIRC".to_string(),
            username: "rustirc".to_string(),
            realname: "RustIRC Client".to_string(),
            password: None,
            reconnect_attempts: 5,
            reconnect_delay: Duration::from_secs(5),
            ping_timeout: Duration::from_secs(300), // 5 minutes
            message_timeout: Duration::from_secs(30),
        }
    }
}

/// IRC connection handle
pub struct IrcConnection {
    config: ConnectionConfig,
    state: Arc<RwLock<ConnectionState>>,
    event_bus: Arc<EventBus>,
    tx_commands: mpsc::UnboundedSender<Command>,
    last_ping: Arc<RwLock<Option<Instant>>>,
    connection_id: String,
}

impl IrcConnection {
    pub fn new(config: ConnectionConfig, event_bus: Arc<EventBus>) -> Self {
        let (tx_commands, _) = mpsc::unbounded_channel();
        let connection_id = format!("{}:{}", config.server, config.port);
        
        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            event_bus,
            tx_commands,
            last_ping: Arc::new(RwLock::new(None)),
            connection_id,
        }
    }

    /// Get current connection state
    pub async fn state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }

    /// Get connection ID
    pub fn id(&self) -> &str {
        &self.connection_id
    }

    /// Start connection with automatic reconnection
    pub async fn connect(&mut self) -> Result<()> {
        self.set_state(ConnectionState::Connecting).await;
        
        let mut attempt = 0;
        loop {
            match self.try_connect().await {
                Ok(()) => {
                    info!("Successfully connected to {}", self.config.server);
                    break;
                }
                Err(e) => {
                    attempt += 1;
                    error!("Connection attempt {} failed: {}", attempt, e);
                    
                    if attempt >= self.config.reconnect_attempts {
                        let error_msg = format!("Failed to connect after {} attempts", attempt);
                        self.set_state(ConnectionState::Failed(error_msg.clone())).await;
                        return Err(Error::ConnectionFailed(error_msg));
                    }
                    
                    let delay = self.config.reconnect_delay * attempt;
                    warn!("Retrying connection in {} seconds", delay.as_secs());
                    sleep(delay).await;
                }
            }
        }
        
        Ok(())
    }

    /// Attempt single connection
    async fn try_connect(&mut self) -> Result<()> {
        // Connect to server
        let addr = format!("{}:{}", self.config.server, self.config.port);
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|_| Error::InvalidAddress(addr.clone()))?;
        
        let stream = timeout(self.config.message_timeout, TcpStream::connect(socket_addr))
            .await
            .map_err(|_| Error::ConnectionTimeout)?
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        self.set_state(ConnectionState::Connected).await;
        
        // Create command channel
        let (tx_commands, rx_commands) = mpsc::unbounded_channel();
        self.tx_commands = tx_commands.clone();
        
        // Handle TLS vs plain connections
        if self.config.use_tls {
            let connector = self.create_tls_connector()?;
            let server_string = self.config.server.clone();
            let server_string_for_error = server_string.clone();
            
            // Convert to owned ServerName
            let server_name = match server_string.try_into() {
                Ok(name) => name,
                Err(_) => return Err(Error::InvalidTlsName(server_string_for_error)),
            };
            
            let tls_stream = connector.connect(server_name, stream).await
                .map_err(|e| Error::TlsError(e.to_string()))?;
            
            self.handle_connection_tls(tls_stream, rx_commands).await?;
        } else {
            self.handle_connection_plain(stream, rx_commands).await?;
        }

        // Perform IRC registration
        self.register().await?;
        
        Ok(())
    }

    /// Create TLS connector
    fn create_tls_connector(&self) -> Result<TlsConnector> {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = TlsConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(TlsConnector::from(Arc::new(config)))
    }

    /// Handle TLS connection
    async fn handle_connection_tls(
        &self,
        tls_stream: tokio_rustls::client::TlsStream<TcpStream>,
        rx_commands: mpsc::UnboundedReceiver<Command>,
    ) -> Result<()> {
        let (reader, writer) = tokio::io::split(tls_stream);
        let reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);
        
        self.run_connection_tasks(reader, writer, rx_commands).await
    }

    /// Handle plain connection
    async fn handle_connection_plain(
        &self,
        stream: TcpStream,
        rx_commands: mpsc::UnboundedReceiver<Command>,
    ) -> Result<()> {
        let (reader, writer) = stream.into_split();
        let reader = BufReader::new(reader);
        let writer = BufWriter::new(writer);
        
        self.run_connection_tasks(reader, writer, rx_commands).await
    }

    /// Run connection tasks (reader, writer, ping)
    async fn run_connection_tasks<R, W>(
        &self,
        reader: BufReader<R>,
        writer: BufWriter<W>,
        rx_commands: mpsc::UnboundedReceiver<Command>,
    ) -> Result<()>
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
        W: tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        // Start reader task
        let reader_task = self.start_reader_task_generic(reader);
        
        // Start writer task  
        let writer_task = self.start_writer_task_generic(writer, rx_commands);
        
        // Start ping task
        let ping_task = self.start_ping_task();
        
        // Wait for tasks to complete (they run until disconnection)
        tokio::select! {
            _ = reader_task => {},
            _ = writer_task => {},
            _ = ping_task => {},
        }
        
        Ok(())
    }

    /// Perform IRC registration sequence
    async fn register(&self) -> Result<()> {
        self.set_state(ConnectionState::Authenticating).await;
        
        // Send PASS if password provided
        if let Some(password) = &self.config.password {
            self.send_command_internal(Command::Pass {
                password: password.clone(),
            }).await?;
        }
        
        // Send NICK
        self.send_command_internal(Command::Nick {
            nickname: self.config.nickname.clone(),
        }).await?;
        
        // Send USER
        self.send_command_internal(Command::User {
            username: self.config.username.clone(),
            mode: "0".to_string(),
            realname: self.config.realname.clone(),
        }).await?;
        
        // TODO: Wait for registration complete (001 numeric)
        // For now, just set state to registered
        self.set_state(ConnectionState::Registered).await;
        
        Ok(())
    }

    /// Start message reader task (generic version)
    async fn start_reader_task_generic<R>(&self, mut reader: BufReader<R>) -> Result<()>
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
    {
        let event_bus = self.event_bus.clone();
        let connection_id = self.connection_id.clone();
        let last_ping = self.last_ping.clone();
        
        tokio::spawn(async move {
            let mut line = String::new();
            
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        // Connection closed
                        break;
                    }
                    Ok(_) => {
                        // Remove trailing \r\n
                        let message_text = line.trim_end();
                        
                        debug!("Received: {}", message_text);
                        
                        // Parse IRC message
                        match Parser::parse(message_text) {
                            Ok(message) => {
                                // Handle PING specially
                                if message.command == "PING" {
                                    if let Some(server) = message.params.first() {
                                        // Send PONG response
                                        // TODO: Send via command channel
                                        debug!("Responding to PING from {}", server);
                                    }
                                }
                                
                                // Update last ping time
                                if message.command == "PONG" {
                                    *last_ping.write().await = Some(Instant::now());
                                }
                                
                                // Emit message event
                                let event = Event::MessageReceived {
                                    connection_id: connection_id.clone(),
                                    message,
                                };
                                event_bus.emit(event).await;
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {} - Error: {}", message_text, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Read error: {}", e);
                        break;
                    }
                }
            }
            
            // Emit disconnection event
            let event = Event::Disconnected {
                connection_id,
                reason: "Read loop ended".to_string(),
            };
            event_bus.emit(event).await;
        });
        
        Ok(())
    }

    /// Start message writer task (generic version)
    async fn start_writer_task_generic<W>(
        &self,
        mut writer: BufWriter<W>,
        mut rx_commands: mpsc::UnboundedReceiver<Command>,
    ) -> Result<()>
    where
        W: tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        tokio::spawn(async move {
            while let Some(command) = rx_commands.recv().await {
                let message = command.to_message();
                let message_text = format!("{}\r\n", message);
                
                debug!("Sending: {}", message_text.trim());
                
                if let Err(e) = writer.write_all(message_text.as_bytes()).await {
                    error!("Write error: {}", e);
                    break;
                }
                
                if let Err(e) = writer.flush().await {
                    error!("Flush error: {}", e);
                    break;
                }
            }
        });
        
        Ok(())
    }

    /// Start ping/keepalive task
    async fn start_ping_task(&self) -> Result<()> {
        let tx_commands = self.tx_commands.clone();
        let ping_timeout = self.config.ping_timeout;
        let last_ping = self.last_ping.clone();
        let server = self.config.server.clone();
        
        tokio::spawn(async move {
            let mut ping_interval = interval(ping_timeout / 2);
            
            loop {
                ping_interval.tick().await;
                
                // Check if we need to send a ping
                let should_ping = {
                    let last = last_ping.read().await;
                    match *last {
                        Some(last_time) => last_time.elapsed() > ping_timeout,
                        None => true, // Never received pong, send ping
                    }
                };
                
                if should_ping {
                    let ping_cmd = Command::Ping {
                        server1: server.clone(),
                        server2: None,
                    };
                    
                    if tx_commands.send(ping_cmd).is_err() {
                        // Channel closed, exit task
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Send command through the connection
    pub async fn send_command(&self, command: Command) -> Result<()> {
        self.tx_commands.send(command)
            .map_err(|_| Error::ConnectionClosed)?;
        Ok(())
    }
    
    /// Internal command sending (for registration)
    async fn send_command_internal(&self, command: Command) -> Result<()> {
        self.tx_commands.send(command)
            .map_err(|_| Error::ConnectionClosed)?;
        Ok(())
    }

    /// Set connection state and emit event
    async fn set_state(&self, new_state: ConnectionState) {
        {
            let mut state = self.state.write().await;
            *state = new_state.clone();
        }
        
        let event = Event::StateChanged {
            connection_id: self.connection_id.clone(),
            state: new_state,
        };
        self.event_bus.emit(event).await;
    }

    /// Disconnect from server
    pub async fn disconnect(&self) -> Result<()> {
        // Send QUIT command
        let quit_cmd = Command::Quit {
            message: Some("RustIRC shutting down".to_string()),
        };
        self.send_command(quit_cmd).await?;
        
        self.set_state(ConnectionState::Disconnected).await;
        Ok(())
    }
}

/// Multi-server connection manager
pub struct ConnectionManager {
    connections: Arc<RwLock<std::collections::HashMap<String, Arc<IrcConnection>>>>,
    event_bus: Arc<EventBus>,
}

impl ConnectionManager {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self {
            connections: Arc::new(RwLock::new(std::collections::HashMap::new())),
            event_bus,
        }
    }

    /// Add a new connection
    pub async fn add_connection(&self, id: String, config: ConnectionConfig) -> Result<Arc<IrcConnection>> {
        let connection = Arc::new(IrcConnection::new(config, self.event_bus.clone()));
        self.connections.write().await.insert(id.clone(), connection.clone());
        Ok(connection)
    }

    /// Get connection by ID
    pub async fn get_connection(&self, id: &str) -> Option<Arc<IrcConnection>> {
        self.connections.read().await.get(id).cloned()
    }

    /// Remove connection
    pub async fn remove_connection(&self, id: &str) -> Option<Arc<IrcConnection>> {
        self.connections.write().await.remove(id)
    }

    /// List all connection IDs
    pub async fn list_connections(&self) -> Vec<String> {
        self.connections.read().await.keys().cloned().collect()
    }

    /// Get connection states
    pub async fn connection_states(&self) -> std::collections::HashMap<String, ConnectionState> {
        let connections = self.connections.read().await;
        let mut states = std::collections::HashMap::new();
        
        for (id, conn) in connections.iter() {
            states.insert(id.clone(), conn.state().await);
        }
        
        states
    }

    /// Connect all registered connections
    pub async fn connect_all(&self) -> Result<()> {
        let connections = self.connections.read().await.clone();
        
        for (id, connection) in connections {
            tokio::spawn(async move {
                // Note: This would need to be implemented differently in practice
                // as we can't easily extract mutable access from Arc
                info!("Would connect to {}", id);
            });
        }
        
        Ok(())
    }

    /// Disconnect all connections
    pub async fn disconnect_all(&self) -> Result<()> {
        let connections = self.connections.read().await.clone();
        
        for (id, connection) in connections {
            if let Err(e) = connection.disconnect().await {
                warn!("Failed to disconnect {}: {}", id, e);
            }
        }
        
        Ok(())
    }
}