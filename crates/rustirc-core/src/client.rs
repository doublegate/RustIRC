//! IRC Client implementation

use crate::config::Config;
use crate::error::Result;
use crate::events::{Event, EventBus};
use crate::state::ClientState;
use rustirc_protocol::{Command, Message};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct IrcClient {
    config: Config,
    state: Arc<RwLock<ClientState>>,
    event_bus: Arc<EventBus>,
}

impl IrcClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ClientState::default())),
            event_bus: Arc::new(EventBus::new()),
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
        // Implementation will be added in Phase 2
        tracing::info!("Connecting to {}:{}", server, port);
        
        // Emit connection event
        let connection_id = format!("{}:{}", server, port);
        let event = Event::Connected { connection_id };
        self.event_bus.emit(event).await;
        
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        tracing::info!("Disconnecting from server");
        Ok(())
    }

    pub async fn send_command(&self, command: Command) -> Result<()> {
        tracing::debug!("Sending command: {:?}", command);
        
        // Convert command to message for proper protocol handling
        let message = command.to_message();
        self.send_raw_message(message).await
    }
    
    pub async fn send_raw_message(&self, message: Message) -> Result<()> {
        tracing::debug!("Sending raw message: {}", message);
        
        // In a real implementation, this would send through the connection
        // For now, we'll emit an event to demonstrate Message usage
        let event = Event::MessageSent {
            connection_id: "default".to_string(),
            message,
        };
        self.event_bus.emit(event).await;
        
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
}