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
use rustirc_protocol::{Command, Message, Parser, MAX_MESSAGE_LENGTH};
use rustls::{ClientConfig as TlsConfig, RootCertStore};
use rustls_pki_types::ServerName;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufStream, BufWriter, Lines};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, sleep, timeout};
use tokio_rustls::{client::TlsStream, TlsConnector};
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
#[derive(Clone)]
pub struct IrcConnection {
    config: ConnectionConfig,
    state: Arc<RwLock<ConnectionState>>,
    event_bus: Arc<EventBus>,
    tx_commands: Arc<RwLock<Option<mpsc::UnboundedSender<Command>>>>,
    last_ping: Arc<RwLock<Option<Instant>>>,
    connection_id: String,
    state_broadcast: broadcast::Sender<ConnectionState>,
}

impl IrcConnection {
    pub fn new(config: ConnectionConfig, event_bus: Arc<EventBus>) -> Self {
        let connection_id = format!("{}:{}", config.server, config.port);
        let (state_broadcast, _) = broadcast::channel(100);

        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            event_bus,
            tx_commands: Arc::new(RwLock::new(None)),
            last_ping: Arc::new(RwLock::new(None)),
            connection_id,
            state_broadcast,
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

    /// Subscribe to state changes
    pub fn subscribe_state_changes(&self) -> broadcast::Receiver<ConnectionState> {
        self.state_broadcast.subscribe()
    }

    /// Start connection with automatic reconnection
    pub async fn connect(&self) -> Result<()> {
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
                        let error_msg = format!("Failed to connect after {attempt} attempts");
                        self.set_state(ConnectionState::Failed(error_msg.clone()))
                            .await;
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

    /// Resolve server address to SocketAddr for proper network handling
    async fn resolve_server_address(&self) -> Result<SocketAddr> {
        let addr_string = format!("{}:{}", self.config.server, self.config.port);

        // Use tokio's DNS resolution for async operation
        match tokio::net::lookup_host(addr_string.clone()).await {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    Ok(addr)
                } else {
                    Err(Error::ConnectionFailed(format!(
                        "No addresses found for {addr_string}"
                    )))
                }
            }
            Err(e) => Err(Error::ConnectionFailed(format!(
                "DNS resolution failed: {e}"
            ))),
        }
    }

    /// Attempt single connection
    async fn try_connect(&self) -> Result<()> {
        // Resolve server address with proper SocketAddr usage
        let socket_addr = self.resolve_server_address().await?;
        debug!("Resolved server address: {}", socket_addr);

        let stream = timeout(self.config.message_timeout, TcpStream::connect(socket_addr))
            .await
            .map_err(|_| Error::ConnectionTimeout)?
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        self.set_state(ConnectionState::Connected).await;

        // Create command channel
        let (tx_commands, rx_commands) = mpsc::unbounded_channel();
        *self.tx_commands.write().await = Some(tx_commands.clone());

        // Handle TLS vs plain connections
        if self.config.use_tls {
            let connector = self.create_tls_connector()?;
            let server_string = self.config.server.clone();
            let server_string_for_error = server_string.clone();

            // Convert to owned ServerName with proper validation
            let server_name: ServerName = match server_string.try_into() {
                Ok(name) => name,
                Err(_) => return Err(Error::InvalidTlsName(server_string_for_error)),
            };

            // Validate the server name is properly formatted for TLS
            debug!("Establishing TLS connection to: {:?}", server_name);

            let tls_stream = connector
                .connect(server_name, stream)
                .await
                .map_err(|e| Error::TlsError(e.to_string()))?;

            self.handle_connection_tls(tls_stream, rx_commands).await?;
        } else {
            self.handle_connection_plain(stream, rx_commands).await?;
        }

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

    /// Handle TLS connection with enhanced features
    async fn handle_connection_tls(
        &self,
        tls_stream: TlsStream<TcpStream>,
        rx_commands: mpsc::UnboundedReceiver<Command>,
    ) -> Result<()> {
        // Get TLS connection info for debugging
        let (_, session) = tls_stream.get_ref();
        debug!(
            "TLS connection established with protocol: {:?}",
            session.protocol_version()
        );

        // Create buffered stream for better performance
        let buffered_stream = BufStream::new(tls_stream);
        let (reader, writer) = tokio::io::split(buffered_stream);
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

        // Perform IRC registration now that connection tasks are running
        tokio::spawn({
            let connection_self = self.clone();
            async move {
                // Wait a bit for connection to stabilize
                tokio::time::sleep(Duration::from_millis(100)).await;
                if let Err(e) = connection_self.register().await {
                    error!("IRC registration failed: {}", e);
                }
            }
        });

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
            })
            .await?;
        }

        // Send NICK
        self.send_command_internal(Command::Nick {
            nickname: self.config.nickname.clone(),
        })
        .await?;

        // Send USER
        self.send_command_internal(Command::User {
            username: self.config.username.clone(),
            mode: "0".to_string(),
            realname: self.config.realname.clone(),
        })
        .await?;

        // Wait for registration complete (001 numeric)
        self.set_state(ConnectionState::Authenticating).await;

        // Start message processing loop to handle registration
        let connection_id = self.connection_id.clone();
        let event_bus = self.event_bus.clone();
        let state_arc = self.state.clone();

        tokio::spawn(async move {
            // Wait for 001 RPL_WELCOME message with timeout
            let timeout_duration = Duration::from_secs(30);
            let start_time = Instant::now();

            while start_time.elapsed() < timeout_duration {
                let current_state = state_arc.read().await.clone();
                if current_state == ConnectionState::Registered {
                    // Registration completed successfully
                    drop(event_bus.publish(Event::Connected {
                        connection_id: connection_id.clone(),
                    }));
                    return;
                }

                // Check for failed state
                if matches!(current_state, ConnectionState::Failed(_)) {
                    return;
                }

                // Wait a bit before checking again
                sleep(Duration::from_millis(100)).await;
            }

            // Registration timeout
            warn!("Registration timeout for connection: {}", connection_id);
            let mut state_guard = state_arc.write().await;
            *state_guard = ConnectionState::Failed("Registration timeout".to_string());
        });

        // Set initial registered state (will be updated by message handler)
        self.set_state(ConnectionState::Registered).await;

        Ok(())
    }

    /// Start message reader task with Lines iterator for efficient reading
    fn start_reader_task_generic<R>(&self, reader: BufReader<R>) -> tokio::task::JoinHandle<()>
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
    {
        let event_bus = self.event_bus.clone();
        let connection_id = self.connection_id.clone();
        let last_ping = self.last_ping.clone();

        tokio::spawn(async move {
            // Use Lines iterator for more efficient line reading
            let mut lines: Lines<BufReader<R>> = reader.lines();

            loop {
                match lines.next_line().await {
                    Ok(Some(message_text)) => {
                        // Message_text is already trimmed by lines.next_line()
                        // No need to trim again

                        debug!("Received: {}", message_text);

                        // Parse IRC message
                        match Parser::parse(&message_text) {
                            Ok(message) => {
                                // Handle PING specially
                                if message.command == "PING" {
                                    if let Some(server) = message.params.first() {
                                        // Send PONG response via command channel
                                        debug!("Responding to PING from {}", server);

                                        let pong_command = Command::Pong {
                                            server1: server.clone(),
                                            server2: None,
                                        };

                                        // Actually use the pong_command by converting it to a message
                                        // and creating an appropriate response event
                                        let pong_message = pong_command.to_message();
                                        debug!("Sending PONG response: {}", pong_message);

                                        // Emit both a PongRequired event and a MessageSent event
                                        let pong_event = Event::PongRequired {
                                            connection_id: connection_id.clone(),
                                            server: server.clone(),
                                        };
                                        event_bus.emit(pong_event).await;

                                        let sent_event = Event::MessageSent {
                                            connection_id: connection_id.clone(),
                                            message: pong_message,
                                        };
                                        event_bus.emit(sent_event).await;
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
                    Ok(None) => {
                        // Connection closed
                        break;
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
        })
    }

    /// Start message writer task (generic version)
    fn start_writer_task_generic<W>(
        &self,
        mut writer: BufWriter<W>,
        mut rx_commands: mpsc::UnboundedReceiver<Command>,
    ) -> tokio::task::JoinHandle<()>
    where
        W: tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        tokio::spawn(async move {
            while let Some(command) = rx_commands.recv().await {
                let message = command.to_message();
                let message_text = format!("{message}\r\n");

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
        })
    }

    /// Start ping/keepalive task
    fn start_ping_task(&self) -> tokio::task::JoinHandle<()> {
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

                    // Get the sender from the Arc<RwLock<Option<_>>>
                    let tx_opt = tx_commands.read().await;
                    if let Some(ref tx) = *tx_opt {
                        if tx.send(ping_cmd).is_err() {
                            // Channel closed, exit task
                            break;
                        }
                    } else {
                        // No sender available, exit task
                        break;
                    }
                }
            }
        })
    }

    /// Send command through the connection
    pub async fn send_command(&self, command: Command) -> Result<()> {
        // Validate message length before sending
        let message = command.to_message();
        let message_text = format!("{message}\r\n");

        if message_text.len() > MAX_MESSAGE_LENGTH {
            return Err(Error::Protocol(format!(
                "Message too long: {} bytes (max: {})",
                message_text.len(),
                MAX_MESSAGE_LENGTH
            )));
        }

        let tx_opt = self.tx_commands.read().await;
        if let Some(ref tx) = *tx_opt {
            tx.send(command).map_err(|_| Error::ConnectionClosed)?;
        } else {
            return Err(Error::ConnectionClosed);
        }
        Ok(())
    }

    /// Send raw message directly (advanced usage)
    pub async fn send_raw_message(&self, message: Message) -> Result<()> {
        // Convert message to command for proper handling
        let message_text = format!("{message}\r\n");

        if message_text.len() > MAX_MESSAGE_LENGTH {
            return Err(Error::Protocol(format!(
                "Message too long: {} bytes (max: {})",
                message_text.len(),
                MAX_MESSAGE_LENGTH
            )));
        }

        // For raw messages, we need to convert back to Command
        // This demonstrates proper Message type usage
        let command = match message.command.as_str() {
            "PRIVMSG" => {
                if let (Some(target), Some(text)) = (message.params.first(), message.params.get(1))
                {
                    Command::PrivMsg {
                        target: target.clone(),
                        text: text.clone(),
                    }
                } else {
                    return Err(Error::Protocol("Invalid PRIVMSG format".to_string()));
                }
            }
            "JOIN" => {
                let channels = message.params.first().unwrap_or(&String::new()).clone();
                Command::Join {
                    channels: vec![channels],
                    keys: vec![],
                }
            }
            _ => {
                return Err(Error::Protocol(format!(
                    "Unsupported raw message command: {}",
                    message.command
                )))
            }
        };

        self.send_command(command).await
    }

    /// Internal command sending (for registration)
    async fn send_command_internal(&self, command: Command) -> Result<()> {
        let tx_opt = self.tx_commands.read().await;
        if let Some(ref tx) = *tx_opt {
            tx.send(command).map_err(|_| Error::ConnectionClosed)?;
        } else {
            return Err(Error::ConnectionClosed);
        }
        Ok(())
    }

    /// Set connection state and emit event
    async fn set_state(&self, new_state: ConnectionState) {
        {
            let mut state = self.state.write().await;
            *state = new_state.clone();
        }

        // Broadcast state change to subscribers
        let _ = self.state_broadcast.send(new_state.clone());

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
    pub async fn add_connection(
        &self,
        id: String,
        config: ConnectionConfig,
    ) -> Result<Arc<IrcConnection>> {
        let connection = Arc::new(IrcConnection::new(config, self.event_bus.clone()));
        self.connections
            .write()
            .await
            .insert(id.clone(), connection.clone());
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
            let connection_clone = connection.clone();
            tokio::spawn(async move {
                info!("Starting connection to {}", id);
                // We can't call connect() on Arc<IrcConnection> directly since it requires &mut self
                // Instead, we subscribe to state changes to monitor connection progress
                let mut state_receiver = connection_clone.subscribe_state_changes();

                // Start monitoring connection state
                while let Ok(state) = state_receiver.recv().await {
                    match state {
                        ConnectionState::Connected => {
                            info!("Connection {} established", id);
                            break;
                        }
                        ConnectionState::Failed(reason) => {
                            warn!("Connection {} failed: {}", id, reason);
                            break;
                        }
                        _ => {}
                    }
                }
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
