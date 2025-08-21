//! IRC Client implementation

use crate::config::Config;
use crate::error::{Error, Result};
use crate::events::EventBus;
use crate::state::ClientState;
use crate::connection::{ConnectionManager, ConnectionConfig};
use rustirc_protocol::{Command, Message};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct IrcClient {
    config: Config,
    state: Arc<RwLock<ClientState>>,
    event_bus: Arc<EventBus>,
    connection_manager: Arc<ConnectionManager>,
}

impl IrcClient {
    pub fn new(config: Config) -> Self {
        let event_bus = Arc::new(EventBus::new());
        let connection_manager = Arc::new(ConnectionManager::new(event_bus.clone()));
        
        Self {
            config,
            state: Arc::new(RwLock::new(ClientState::default())),
            event_bus,
            connection_manager,
        }
    }
    
    /// Get client configuration
    pub fn get_config(&self) -> &Config {
        &self.config
    }
    
    /// Get server configuration
    pub fn get_server_config(&self, server_id: &str) -> Option<&crate::config::ServerConfig> {
        self.config.servers.iter().find(|s| s.name == server_id)
    }
    
    /// Get UI configuration
    pub fn get_ui_config(&self) -> &crate::config::UiConfig {
        &self.config.ui
    }

    pub async fn connect(&self, server: &str, port: u16) -> Result<()> {
        tracing::info!("Connecting to {}:{}", server, port);
        
        // Find server configuration or use defaults
        let server_config = self.get_server_config(server);
        
        let connection_config = if let Some(srv_config) = server_config {
            ConnectionConfig {
                server: srv_config.address.clone(),
                port: srv_config.port,
                use_tls: srv_config.use_tls,
                verify_tls: true,
                nickname: self.config.user.nickname.clone(),
                username: self.config.user.username.clone(),
                realname: self.config.user.realname.clone(),
                password: srv_config.password.clone(),
                ..Default::default()
            }
        } else {
            ConnectionConfig {
                server: server.to_string(),
                port,
                nickname: "RustIRC".to_string(),
                username: "rustirc".to_string(),
                realname: "RustIRC Client".to_string(),
                ..Default::default()
            }
        };
        
        // Create connection ID
        let connection_id = format!("{}:{}", server, port);
        
        // Add connection to manager
        let connection = self.connection_manager
            .add_connection(connection_id.clone(), connection_config)
            .await?;
        
        // Spawn connection task - now connect() works with &self
        let connection_manager = self.connection_manager.clone();
        let connection_id_clone = connection_id.clone();
        
        tokio::spawn(async move {
            match connection.connect().await {
                Ok(()) => {
                    tracing::info!("Connection established for {}", connection_id_clone);
                    // Connection is now active and will be managed by the connection manager
                }
                Err(e) => {
                    tracing::error!("Connection failed for {}: {}", connection_id_clone, e);
                    // Remove failed connection from manager
                    connection_manager.remove_connection(&connection_id_clone).await;
                }
            }
        });
        
        tracing::info!("Real IRC connection initiated for {}", connection_id);
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        tracing::info!("Disconnecting from all servers");
        self.connection_manager.disconnect_all().await?;
        Ok(())
    }

    pub async fn send_command(&self, command: Command) -> Result<()> {
        tracing::debug!("Sending command: {:?}", command);
        
        // Get the first available connection (in a real app, we'd specify which connection)
        let connections = self.connection_manager.list_connections().await;
        if let Some(connection_id) = connections.first() {
            if let Some(connection) = self.connection_manager.get_connection(connection_id).await {
                connection.send_command(command).await?;
            } else {
                return Err(Error::Protocol("No active connections found".to_string()));
            }
        } else {
            return Err(Error::Protocol("No connections available".to_string()));
        }
        
        Ok(())
    }
    
    pub async fn send_raw_message(&self, message: Message) -> Result<()> {
        tracing::debug!("Sending raw message: {}", message);
        
        // Get the first available connection (in a real app, we'd specify which connection)
        let connections = self.connection_manager.list_connections().await;
        if let Some(connection_id) = connections.first() {
            if let Some(connection) = self.connection_manager.get_connection(connection_id).await {
                connection.send_raw_message(message).await?;
            } else {
                return Err(Error::Protocol("No active connections found".to_string()));
            }
        } else {
            return Err(Error::Protocol("No connections available".to_string()));
        }
        
        Ok(())
    }

    pub async fn join_channel(&self, channel: &str) -> Result<()> {
        self.send_command(Command::Join {
            channels: vec![channel.to_string()],
            keys: vec![],
        }).await
    }

    pub async fn send_message(&self, target: &str, text: &str) -> Result<()> {
        self.send_command(Command::PrivMsg {
            target: target.to_string(),
            text: text.to_string(),
        }).await
    }

    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    pub async fn get_state(&self) -> ClientState {
        self.state.read().await.clone()
    }
    
    pub fn connection_manager(&self) -> Arc<ConnectionManager> {
        self.connection_manager.clone()
    }
    
    /// Connect to a specific server with custom configuration
    pub async fn connect_with_config(&self, connection_config: ConnectionConfig) -> Result<String> {
        let connection_id = format!("{}:{}", connection_config.server, connection_config.port);
        
        // Add connection to manager
        let connection = self.connection_manager
            .add_connection(connection_id.clone(), connection_config)
            .await?;
        
        // Start the connection - removed Arc::try_unwrap since connect() now takes &self
        let connection_clone = connection.clone();
        
        // Spawn connection task
        let connection_id_clone = connection_id.clone();
        tokio::spawn(async move {
            if let Err(e) = connection_clone.connect().await {
                tracing::error!("Connection failed for {}: {}", connection_id_clone, e);
            }
        });
        
        Ok(connection_id)
    }
}