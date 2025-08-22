//! Mock IRC server for testing
//!
//! Provides a mock IRC server that can be used for testing IRC client functionality
//! without requiring a real IRC server connection.

#![allow(
    unused_variables,
    dead_code,
    clippy::uninlined_format_args,
    clippy::map_identity,
    clippy::manual_unwrap_or_default,
    clippy::unnecessary_to_owned,
    clippy::get_first
)]

use anyhow::Result;
use rustirc_protocol::Message;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

/// Mock IRC server configuration
#[derive(Debug, Clone)]
pub struct MockServerConfig {
    pub server_name: String,
    pub motd: Vec<String>,
    pub channels: Vec<String>,
    pub operators: Vec<String>,
    pub max_clients: usize,
    pub ping_interval: Duration,
}

impl Default for MockServerConfig {
    fn default() -> Self {
        Self {
            server_name: "mock.irc.server".to_string(),
            motd: vec![
                "Welcome to the Mock IRC Server".to_string(),
                "This is a test server for RustIRC".to_string(),
                "Enjoy your stay!".to_string(),
            ],
            channels: vec!["#test".to_string(), "#general".to_string()],
            operators: vec!["admin".to_string()],
            max_clients: 100,
            ping_interval: Duration::from_secs(60),
        }
    }
}

/// Mock IRC client state
#[derive(Debug, Clone)]
pub struct MockClient {
    pub nickname: Option<String>,
    pub username: Option<String>,
    pub realname: Option<String>,
    pub hostname: String,
    pub channels: HashSet<String>,
    pub is_registered: bool,
    pub is_operator: bool,
    pub last_ping: std::time::Instant,
    pub pending_messages: Option<Vec<String>>,
}

impl MockClient {
    pub fn new(hostname: String) -> Self {
        Self {
            nickname: None,
            username: None,
            realname: None,
            hostname,
            channels: HashSet::new(),
            is_registered: false,
            is_operator: false,
            last_ping: std::time::Instant::now(),
            pending_messages: None,
        }
    }

    pub fn full_mask(&self) -> String {
        if let (Some(nick), Some(user)) = (&self.nickname, &self.username) {
            format!("{}!{}@{}", nick, user, self.hostname)
        } else {
            self.hostname.clone()
        }
    }
}

/// Mock IRC server
pub struct MockIrcServer {
    config: MockServerConfig,
    clients: Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
    channels: Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    listener: Option<TcpListener>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl MockIrcServer {
    pub fn new(config: MockServerConfig) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            listener: None,
            shutdown_tx: None,
        }
    }

    /// Start the mock IRC server
    pub async fn start(&mut self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        info!("Mock IRC server listening on {}", local_addr);

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let clients = Arc::clone(&self.clients);
        let channels = Arc::clone(&self.channels);
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                info!("New client connected: {}", addr);

                                let client_handler = ClientHandler {
                                    stream,
                                    addr,
                                    config: config.clone(),
                                    clients: Arc::clone(&clients),
                                    channels: Arc::clone(&channels),
                                };

                                tokio::spawn(async move {
                                    if let Err(e) = client_handler.handle().await {
                                        error!("Client handler error: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                error!("Failed to accept connection: {}", e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Mock IRC server shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the mock IRC server
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }
        Ok(())
    }

    /// Get current client count
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get clients in a channel
    pub async fn channel_users(&self, channel: &str) -> Vec<String> {
        let channels = self.channels.read().await;
        let clients = self.clients.read().await;

        if let Some(addrs) = channels.get(channel) {
            addrs
                .iter()
                .filter_map(|addr| clients.get(addr))
                .filter_map(|client| client.nickname.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Client connection handler
struct ClientHandler {
    stream: TcpStream,
    addr: SocketAddr,
    config: MockServerConfig,
    clients: Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
    channels: Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
}

impl ClientHandler {
    async fn handle(self) -> Result<()> {
        let addr = self.addr;
        let config = self.config;
        let clients = self.clients;
        let channels = self.channels;

        let (reader, mut writer) = self.stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        // Add client to tracking
        {
            let mut clients_guard = clients.write().await;
            clients_guard.insert(
                addr,
                MockClient::new(format!("user-{}.mock.server", addr.port())),
            );
        }

        // Send server welcome
        Self::send_notice_static(&mut writer, "Welcome to Mock IRC Server").await?;

        loop {
            line.clear();

            match timeout(config.ping_interval * 5, reader.read_line(&mut line)).await {
                Ok(Ok(0)) => {
                    debug!("Client {} disconnected", addr);
                    break;
                }
                Ok(Ok(_)) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    debug!("Received from {}: {}", addr, line);

                    if let Err(e) =
                        Self::process_message_static(&mut writer, line, addr, &clients, &channels)
                            .await
                    {
                        warn!("Error processing message from {}: {}", addr, e);
                    }

                    // Deliver any pending messages for this client
                    Self::deliver_pending_messages(&mut writer, addr, &clients).await?;
                }
                Ok(Err(e)) => {
                    error!("Read error from {}: {}", addr, e);
                    break;
                }
                Err(_) => {
                    debug!("Timeout from client {}", addr);
                    break;
                }
            }
        }

        // Remove client from tracking
        {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.remove(&addr) {
                // Remove from all channels
                let mut channels_guard = channels.write().await;
                for channel_name in &client.channels {
                    if let Some(users) = channels_guard.get_mut(channel_name) {
                        users.retain(|user_addr| *user_addr != addr);
                        if users.is_empty() {
                            channels_guard.remove(channel_name);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // Static helper methods
    async fn send_notice_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        text: &str,
    ) -> Result<()> {
        let notice = format!("NOTICE AUTH :{}\r\n", text);
        writer.write_all(notice.as_bytes()).await?;
        Ok(())
    }

    async fn process_message_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        line: &str,
        addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
        channels: &Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    ) -> Result<()> {
        // Parse IRC message
        let message = rustirc_protocol::Parser::parse_message(line)?;

        match message.command.to_uppercase().as_str() {
            "NICK" => Self::handle_nick_static(writer, &message, addr, clients).await?,
            "USER" => Self::handle_user_static(writer, &message, addr, clients).await?,
            "JOIN" => Self::handle_join_static(writer, &message, addr, clients, channels).await?,
            "PART" => Self::handle_part_static(writer, &message, addr, clients, channels).await?,
            "PRIVMSG" => {
                Self::handle_privmsg_static(writer, &message, addr, clients, channels).await?
            }
            "LIST" => Self::handle_list_static(writer, &message, addr, channels).await?,
            "NAMES" => Self::handle_names_static(writer, &message, addr, clients, channels).await?,
            "QUIT" => Self::handle_quit_static(writer, &message, addr, clients, channels).await?,
            _ => {
                Self::send_numeric_static(writer, "421", &[&message.command, "Unknown command"])
                    .await?;
            }
        }

        Ok(())
    }

    // Static helper for sending numerics
    async fn send_numeric_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        code: &str,
        params: &[&str],
    ) -> Result<()> {
        let response = format!(":{} {} {}\r\n", "mock.server", code, params.join(" "));
        writer.write_all(response.as_bytes()).await?;
        Ok(())
    }

    // Static handlers
    async fn handle_nick_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
        addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
    ) -> Result<()> {
        if message.params.is_empty() {
            Self::send_numeric_static(writer, "431", &["No nickname given"]).await?;
            return Ok(());
        }

        let new_nick = &message.params[0];

        // Check if nickname is already taken
        {
            let clients_guard = clients.read().await;
            for (client_addr, client) in clients_guard.iter() {
                if *client_addr != addr && client.nickname.as_ref() == Some(new_nick) {
                    Self::send_numeric_static(
                        writer,
                        "433",
                        &[new_nick, "Nickname is already in use"],
                    )
                    .await?;
                    return Ok(());
                }
            }
        }

        // Update client nickname
        {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                let _old_nick = client.nickname.clone();
                client.nickname = Some(new_nick.clone());

                // Check if both nick and user are set for registration
                if client.nickname.is_some() && client.username.is_some() && !client.is_registered {
                    client.is_registered = true;
                    Self::send_numeric_static(
                        writer,
                        "001",
                        &[new_nick, "Welcome to Mock IRC Server"],
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_user_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
        addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
    ) -> Result<()> {
        if message.params.len() < 4 {
            Self::send_numeric_static(writer, "461", &["USER", "Not enough parameters"]).await?;
            return Ok(());
        }

        // Update client user info
        {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.username = Some(message.params[0].clone());
                client.realname = Some(message.params[3].clone());

                // Check if both nick and user are set for registration
                if client.nickname.is_some() && client.username.is_some() && !client.is_registered {
                    client.is_registered = true;
                    if let Some(nick) = &client.nickname {
                        Self::send_numeric_static(
                            writer,
                            "001",
                            &[nick, "Welcome to Mock IRC Server"],
                        )
                        .await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_join_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
        addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
        channels: &Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    ) -> Result<()> {
        if message.params.is_empty() {
            Self::send_numeric_static(writer, "461", &["JOIN", "Not enough parameters"]).await?;
            return Ok(());
        }

        let channel_name = &message.params[0];

        // Get client info
        let client_mask = {
            let clients_guard = clients.read().await;
            if let Some(client) = clients_guard.get(&addr) {
                client.full_mask()
            } else {
                return Ok(());
            }
        };

        // Add to channel
        {
            let mut channels_guard = channels.write().await;
            channels_guard
                .entry(channel_name.clone())
                .or_insert_with(Vec::new)
                .push(addr);
        }

        // Update client's channel list
        {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.channels.insert(channel_name.clone());
            }
        }

        // Send JOIN confirmation
        let join_msg = format!(":{} JOIN :{}\r\n", client_mask, channel_name);
        writer.write_all(join_msg.as_bytes()).await?;

        // Send NAMES list
        let names = Self::get_channel_users_static(channel_name, clients, channels).await;
        let names_str = names.join(" ");
        Self::send_numeric_static(writer, "353", &["=", channel_name, &names_str]).await?;
        Self::send_numeric_static(writer, "366", &[channel_name, "End of /NAMES list"]).await?;

        Ok(())
    }

    async fn handle_part_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
        addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
        channels: &Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    ) -> Result<()> {
        if message.params.is_empty() {
            Self::send_numeric_static(writer, "461", &["PART", "Not enough parameters"]).await?;
            return Ok(());
        }

        let channel_name = &message.params[0];

        // Get client info
        let client_mask = {
            let clients_guard = clients.read().await;
            if let Some(client) = clients_guard.get(&addr) {
                client.full_mask()
            } else {
                return Ok(());
            }
        };

        // Remove from channel
        {
            let mut channels_guard = channels.write().await;
            if let Some(users) = channels_guard.get_mut(channel_name) {
                users.retain(|user_addr| *user_addr != addr);
                if users.is_empty() {
                    channels_guard.remove(channel_name);
                }
            }
        }

        // Update client's channel list
        {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                client.channels.remove(channel_name);
            }
        }

        // Send PART confirmation
        let part_msg = format!(":{} PART :{}\r\n", client_mask, channel_name);
        writer.write_all(part_msg.as_bytes()).await?;

        Ok(())
    }

    async fn handle_privmsg_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
        addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
        channels: &Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    ) -> Result<()> {
        if message.params.len() < 2 {
            Self::send_numeric_static(writer, "461", &["PRIVMSG", "Not enough parameters"]).await?;
            return Ok(());
        }

        let target = &message.params[0];
        let text = &message.params[1];

        // Get client info
        let client_mask = {
            let clients_guard = clients.read().await;
            if let Some(client) = clients_guard.get(&addr) {
                client.full_mask()
            } else {
                return Ok(());
            }
        };

        if target.starts_with('#') {
            // Channel message - broadcast to channel members
            let privmsg = format!(":{} PRIVMSG {} :{}\r\n", client_mask, target, text);

            // Store the message for each channel member (except sender)
            let channels_guard = channels.read().await;
            if let Some(users) = channels_guard.get(target) {
                let mut clients_guard = clients.write().await;
                for user_addr in users {
                    if *user_addr != addr {
                        if let Some(client) = clients_guard.get_mut(user_addr) {
                            // Add message to client's pending messages queue
                            if client.pending_messages.is_none() {
                                client.pending_messages = Some(Vec::new());
                            }
                            if let Some(ref mut messages) = client.pending_messages {
                                messages.push(privmsg.clone());
                            }
                        }
                    }
                }
            }

            // Send acknowledgment to sender
            writer
                .write_all(format!(":{} PRIVMSG {} :{}\r\n", client_mask, target, text).as_bytes())
                .await?;
        } else {
            // Private message to user
            let privmsg = format!(":{} PRIVMSG {} :{}\r\n", client_mask, target, text);

            // Find target user and queue message
            let mut clients_guard = clients.write().await;
            for (user_addr, client) in clients_guard.iter_mut() {
                if client.nickname.as_ref() == Some(target) && *user_addr != addr {
                    if client.pending_messages.is_none() {
                        client.pending_messages = Some(Vec::new());
                    }
                    if let Some(ref mut messages) = client.pending_messages {
                        messages.push(privmsg.clone());
                    }
                    break;
                }
            }

            // Send acknowledgment to sender
            writer.write_all(privmsg.as_bytes()).await?;
        }

        Ok(())
    }

    async fn handle_list_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        _message: &Message,
        _addr: SocketAddr,
        channels: &Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    ) -> Result<()> {
        Self::send_numeric_static(writer, "321", &["Channel", "Users Name"]).await?;

        let channels_guard = channels.read().await;
        for (channel_name, users) in channels_guard.iter() {
            let user_count = users.len().to_string();
            Self::send_numeric_static(writer, "322", &[channel_name, &user_count, channel_name])
                .await?;
        }

        Self::send_numeric_static(writer, "323", &["End of /LIST"]).await?;
        Ok(())
    }

    async fn handle_names_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
        _addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
        channels: &Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    ) -> Result<()> {
        if message.params.is_empty() {
            Self::send_numeric_static(writer, "461", &["NAMES", "Not enough parameters"]).await?;
            return Ok(());
        }

        let channel_name = &message.params[0];
        let names = Self::get_channel_users_static(channel_name, clients, channels).await;
        let names_str = names.join(" ");

        Self::send_numeric_static(writer, "353", &["=", channel_name, &names_str]).await?;
        Self::send_numeric_static(writer, "366", &[channel_name, "End of /NAMES list"]).await?;

        Ok(())
    }

    async fn handle_quit_static(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
        addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
        channels: &Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    ) -> Result<()> {
        let quit_msg = message
            .params
            .get(0)
            .map(|s| s.as_str())
            .unwrap_or("Client Quit");

        // Get client info before removal
        let client_channels = {
            let clients_guard = clients.read().await;
            if let Some(client) = clients_guard.get(&addr) {
                client.channels.clone()
            } else {
                return Ok(());
            }
        };

        // Remove from all channels
        {
            let mut channels_guard = channels.write().await;
            for channel_name in &client_channels {
                if let Some(users) = channels_guard.get_mut(channel_name) {
                    users.retain(|user_addr| *user_addr != addr);
                    if users.is_empty() {
                        channels_guard.remove(channel_name);
                    }
                }
            }
        }

        // Remove client
        {
            let mut clients_guard = clients.write().await;
            clients_guard.remove(&addr);
        }

        // Send quit confirmation
        let quit_response = format!("ERROR :Closing Link: {}\r\n", quit_msg);
        writer.write_all(quit_response.as_bytes()).await?;

        Ok(())
    }

    async fn get_channel_users_static(
        channel: &str,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
        channels: &Arc<RwLock<HashMap<String, Vec<SocketAddr>>>>,
    ) -> Vec<String> {
        let clients_guard = clients.read().await;
        let channels_guard = channels.read().await;

        if let Some(addrs) = channels_guard.get(channel) {
            addrs
                .iter()
                .filter_map(|addr| clients_guard.get(addr))
                .filter_map(|client| client.nickname.clone())
                .map(|nick| nick)
                .collect()
        } else {
            Vec::new()
        }
    }

    // Message delivery system
    async fn deliver_pending_messages(
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        addr: SocketAddr,
        clients: &Arc<RwLock<HashMap<SocketAddr, MockClient>>>,
    ) -> Result<()> {
        let messages_to_send = {
            let mut clients_guard = clients.write().await;
            if let Some(client) = clients_guard.get_mut(&addr) {
                if let Some(messages) = client.pending_messages.take() {
                    messages
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };

        for message in messages_to_send {
            writer.write_all(message.as_bytes()).await?;
        }

        Ok(())
    }

    async fn process_message(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        line: &str,
    ) -> Result<()> {
        // Parse IRC message
        let message = rustirc_protocol::Parser::parse_message(line)?;

        match message.command.to_uppercase().as_str() {
            "NICK" => self.handle_nick(writer, &message).await?,
            "USER" => self.handle_user(writer, &message).await?,
            "JOIN" => self.handle_join(writer, &message).await?,
            "PART" => self.handle_part(writer, &message).await?,
            "PRIVMSG" => self.handle_privmsg(writer, &message).await?,
            "NOTICE" => self.handle_notice(writer, &message).await?,
            "LIST" => self.handle_list(writer, &message).await?,
            "NAMES" => self.handle_names(writer, &message).await?,
            "WHOIS" => self.handle_whois(writer, &message).await?,
            "PING" => self.handle_ping(writer, &message).await?,
            "PONG" => self.handle_pong(writer, &message).await?,
            "QUIT" => self.handle_quit(writer, &message).await?,
            _ => {
                self.send_numeric(writer, "421", &[&message.command, "Unknown command"])
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_nick(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.is_empty() {
            self.send_numeric(writer, "431", &["No nickname given"])
                .await?;
            return Ok(());
        }

        let new_nick = &message.params[0];

        // Check if nickname is already taken
        {
            let clients = self.clients.read().await;
            for (addr, client) in clients.iter() {
                if *addr != self.addr && client.nickname.as_ref() == Some(new_nick) {
                    self.send_numeric(writer, "433", &[new_nick, "Nickname is already in use"])
                        .await?;
                    return Ok(());
                }
            }
        }

        // Update client nickname
        {
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.get_mut(&self.addr) {
                let old_nick = client.nickname.clone();
                client.nickname = Some(new_nick.clone());

                // If client is registered and nickname changed, notify channels
                if client.is_registered {
                    if let Some(old_nick_str) = old_nick {
                        // Notify all channels about nick change using old nick in mask
                        let old_mask = if let Some(username) = &client.username {
                            format!("{}!{}@{}", old_nick_str, username, client.hostname)
                        } else {
                            format!("{}@{}", old_nick_str, client.hostname)
                        };
                        self.broadcast_to_client_channels(&format!(
                            ":{} NICK :{}",
                            old_mask, new_nick
                        ))
                        .await?;
                    }
                }

                // Check if client is now fully registered
                if !client.is_registered && client.username.is_some() {
                    client.is_registered = true;
                    self.send_welcome_sequence(writer).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_user(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.len() < 4 {
            self.send_numeric(writer, "461", &["USER", "Not enough parameters"])
                .await?;
            return Ok(());
        }

        {
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.get_mut(&self.addr) {
                if client.is_registered {
                    self.send_numeric(writer, "462", &["You may not reregister"])
                        .await?;
                    return Ok(());
                }

                client.username = Some(message.params[0].clone());
                client.realname = Some(message.params[3].clone());

                // Check if client is now fully registered
                if !client.is_registered && client.nickname.is_some() {
                    client.is_registered = true;
                    self.send_welcome_sequence(writer).await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_join(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.is_empty() {
            self.send_numeric(writer, "461", &["JOIN", "Not enough parameters"])
                .await?;
            return Ok(());
        }

        let channels_str = &message.params[0];
        let channel_names: Vec<&str> = channels_str.split(',').collect();

        for channel_name in channel_names {
            if !channel_name.starts_with('#') && !channel_name.starts_with('&') {
                self.send_numeric(writer, "403", &[channel_name, "No such channel"])
                    .await?;
                continue;
            }

            // Add client to channel
            {
                let mut clients = self.clients.write().await;
                let mut channels = self.channels.write().await;

                if let Some(client) = clients.get_mut(&self.addr) {
                    if !client.channels.contains(&channel_name.to_string()) {
                        client.channels.insert(channel_name.to_string());

                        channels
                            .entry(channel_name.to_string())
                            .or_insert_with(Vec::new)
                            .push(self.addr);

                        // Send JOIN message to all users in channel
                        let join_msg = format!(":{} JOIN :{}", client.full_mask(), channel_name);
                        self.broadcast_to_channel(channel_name, &join_msg).await?;

                        // Send NAMES list
                        self.send_names_list(writer, channel_name).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_part(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.is_empty() {
            self.send_numeric(writer, "461", &["PART", "Not enough parameters"])
                .await?;
            return Ok(());
        }

        let channel_name = &message.params[0];
        let part_msg = message.params.get(1).cloned().unwrap_or_default();

        {
            let mut clients = self.clients.write().await;
            let mut channels = self.channels.write().await;

            if let Some(client) = clients.get_mut(&self.addr) {
                if client.channels.contains(channel_name) {
                    client.channels.remove(channel_name);

                    if let Some(users) = channels.get_mut(channel_name) {
                        users.retain(|addr| *addr != self.addr);

                        // Send PART message to remaining users
                        let part_message = if part_msg.is_empty() {
                            format!(":{} PART {}", client.full_mask(), channel_name)
                        } else {
                            format!(
                                ":{} PART {} :{}",
                                client.full_mask(),
                                channel_name,
                                part_msg
                            )
                        };
                        self.broadcast_to_channel(channel_name, &part_message)
                            .await?;

                        if users.is_empty() {
                            channels.remove(channel_name);
                        }
                    }
                } else {
                    self.send_numeric(writer, "442", &[channel_name, "You're not on that channel"])
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_privmsg(
        &self,
        _writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.len() < 2 {
            return Ok(());
        }

        let target = &message.params[0];
        let text = &message.params[1];

        let clients = self.clients.read().await;
        if let Some(client) = clients.get(&self.addr) {
            let privmsg = format!(":{} PRIVMSG {} :{}", client.full_mask(), target, text);

            if target.starts_with('#') || target.starts_with('&') {
                // Channel message
                self.broadcast_to_channel_except(target, &privmsg, self.addr)
                    .await?;
            } else {
                // Private message
                self.send_to_nick(target, &privmsg).await?;
            }
        }

        Ok(())
    }

    async fn handle_notice(
        &self,
        _writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.len() < 2 {
            return Ok(());
        }

        let target = &message.params[0];
        let text = &message.params[1];

        let clients = self.clients.read().await;
        if let Some(client) = clients.get(&self.addr) {
            let notice_msg = format!(":{} NOTICE {} :{}", client.full_mask(), target, text);

            if target.starts_with('#') || target.starts_with('&') {
                self.broadcast_to_channel_except(target, &notice_msg, self.addr)
                    .await?;
            } else {
                self.send_to_nick(target, &notice_msg).await?;
            }
        }

        Ok(())
    }

    async fn handle_list(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        _message: &Message,
    ) -> Result<()> {
        self.send_numeric(writer, "321", &["Channel", "Users", "Name"])
            .await?;

        let channels = self.channels.read().await;
        for (channel_name, users) in channels.iter() {
            self.send_numeric(
                writer,
                "322",
                &[channel_name, &users.len().to_string(), "Mock channel"],
            )
            .await?;
        }

        self.send_numeric(writer, "323", &["End of /LIST"]).await?;
        Ok(())
    }

    async fn handle_names(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.is_empty() {
            return Ok(());
        }

        let channel_name = &message.params[0];
        self.send_names_list(writer, channel_name).await?;
        Ok(())
    }

    async fn handle_whois(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.is_empty() {
            self.send_numeric(writer, "431", &["No nickname given"])
                .await?;
            return Ok(());
        }

        let target_nick = &message.params[0];

        let clients = self.clients.read().await;
        for client in clients.values() {
            if client.nickname.as_ref() == Some(target_nick) {
                // Send WHOIS response
                let realname = client.realname.as_deref().unwrap_or("Unknown");
                let username = client.username.as_deref().unwrap_or("unknown");

                self.send_numeric(
                    writer,
                    "311",
                    &[target_nick, username, &client.hostname, "*", realname],
                )
                .await?;
                self.send_numeric(
                    writer,
                    "312",
                    &[target_nick, &self.config.server_name, "Mock IRC Server"],
                )
                .await?;

                if !client.channels.is_empty() {
                    let channels_vec: Vec<&String> = client.channels.iter().collect();
                    let channels_str = channels_vec
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.send_numeric(writer, "319", &[target_nick, &channels_str])
                        .await?;
                }

                self.send_numeric(writer, "318", &[target_nick, "End of /WHOIS list"])
                    .await?;
                return Ok(());
            }
        }

        self.send_numeric(writer, "401", &[target_nick, "No such nick/channel"])
            .await?;
        Ok(())
    }

    async fn handle_ping(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        if message.params.is_empty() {
            return Ok(());
        }

        let token = &message.params[0];
        let pong_msg = format!(
            ":{} PONG {} :{}\r\n",
            self.config.server_name, self.config.server_name, token
        );
        writer.write_all(pong_msg.as_bytes()).await?;
        Ok(())
    }

    async fn handle_pong(
        &self,
        _writer: &mut tokio::net::tcp::OwnedWriteHalf,
        _message: &Message,
    ) -> Result<()> {
        // Update last ping time
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(&self.addr) {
            client.last_ping = std::time::Instant::now();
        }
        Ok(())
    }

    async fn handle_quit(
        &self,
        _writer: &mut tokio::net::tcp::OwnedWriteHalf,
        message: &Message,
    ) -> Result<()> {
        let quit_msg = message
            .params
            .get(0)
            .cloned()
            .unwrap_or_else(|| "Client Quit".to_string());

        let clients = self.clients.read().await;
        if let Some(client) = clients.get(&self.addr) {
            let quit_message = format!(":{} QUIT :{}", client.full_mask(), quit_msg);
            self.broadcast_to_client_channels(&quit_message).await?;
        }

        Ok(())
    }

    async fn send_welcome_sequence(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
    ) -> Result<()> {
        let clients = self.clients.read().await;
        if let Some(client) = clients.get(&self.addr) {
            if let Some(nick) = &client.nickname {
                let full_mask = client.full_mask();

                self.send_numeric(
                    writer,
                    "001",
                    &[&format!("Welcome to the Mock IRC Network {}", full_mask)],
                )
                .await?;
                self.send_numeric(
                    writer,
                    "002",
                    &[&format!(
                        "Your host is {}, running version 1.0",
                        self.config.server_name
                    )],
                )
                .await?;
                self.send_numeric(writer, "003", &["This server was created today"])
                    .await?;
                self.send_numeric(writer, "004", &[&self.config.server_name, "1.0", "o", "o"])
                    .await?;

                // Send MOTD
                self.send_numeric(
                    writer,
                    "375",
                    &[&format!(
                        "- {} Message of the Day -",
                        self.config.server_name
                    )],
                )
                .await?;
                for line in &self.config.motd {
                    self.send_numeric(writer, "372", &[&format!("- {}", line)])
                        .await?;
                }
                self.send_numeric(writer, "376", &["End of /MOTD command"])
                    .await?;
            }
        }
        Ok(())
    }

    async fn send_names_list(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        channel: &str,
    ) -> Result<()> {
        let names = self.get_channel_names(channel).await;
        if !names.is_empty() {
            let names_str = names.join(" ");
            self.send_numeric(writer, "353", &["=", channel, &names_str])
                .await?;
        }
        self.send_numeric(writer, "366", &[channel, "End of /NAMES list"])
            .await?;
        Ok(())
    }

    async fn get_channel_names(&self, channel: &str) -> Vec<String> {
        let channels = self.channels.read().await;
        let clients = self.clients.read().await;

        if let Some(addrs) = channels.get(channel) {
            addrs
                .iter()
                .filter_map(|addr| clients.get(addr))
                .filter_map(|client| client.nickname.clone())
                .map(|nick| nick)
                .collect()
        } else {
            Vec::new()
        }
    }

    async fn send_numeric(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        code: &str,
        params: &[&str],
    ) -> Result<()> {
        let clients = self.clients.read().await;
        let nick = if let Some(client) = clients.get(&self.addr) {
            client.nickname.as_deref().unwrap_or("*")
        } else {
            "*"
        };

        let params_str = params.join(" ");
        let msg = format!(
            ":{} {} {} :{}\r\n",
            self.config.server_name, code, nick, params_str
        );
        writer.write_all(msg.as_bytes()).await?;
        Ok(())
    }

    async fn send_notice(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        text: &str,
    ) -> Result<()> {
        let msg = format!(":{} NOTICE * :{}\r\n", self.config.server_name, text);
        writer.write_all(msg.as_bytes()).await?;
        Ok(())
    }

    async fn broadcast_to_channel(&self, channel: &str, message: &str) -> Result<()> {
        // Implementation would send message to all clients in channel
        debug!("Broadcasting to {}: {}", channel, message);
        Ok(())
    }

    async fn broadcast_to_channel_except(
        &self,
        channel: &str,
        message: &str,
        except: SocketAddr,
    ) -> Result<()> {
        // Implementation would send message to all clients in channel except specified address
        debug!(
            "Broadcasting to {} (except {}): {}",
            channel, except, message
        );
        Ok(())
    }

    async fn broadcast_to_client_channels(&self, message: &str) -> Result<()> {
        // Implementation would send message to all channels the client is in
        debug!("Broadcasting to client channels: {}", message);
        Ok(())
    }

    async fn send_to_nick(&self, nick: &str, message: &str) -> Result<()> {
        // Implementation would send private message to specific nickname
        debug!("Sending to {}: {}", nick, message);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_mock_server_start_stop() {
        let config = MockServerConfig::default();
        let mut server = MockIrcServer::new(config);

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

        // Start server
        server.start(addr).await.expect("Failed to start server");

        // Stop server
        server.stop().await.expect("Failed to stop server");
    }

    #[tokio::test]
    async fn test_mock_client_creation() {
        let client = MockClient::new("test.hostname".to_string());
        assert_eq!(client.hostname, "test.hostname");
        assert_eq!(client.nickname, None);
        assert!(!client.is_registered);
    }

    #[test]
    fn test_mock_server_config_default() {
        let config = MockServerConfig::default();
        assert_eq!(config.server_name, "mock.irc.server");
        assert!(!config.motd.is_empty());
        assert!(!config.channels.is_empty());
    }
}
