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

    pub async fn connect(&self, server: &str, port: u16) -> Result<()> {
        // Implementation will be added in Phase 2
        tracing::info!("Connecting to {}:{}", server, port);
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        tracing::info!("Disconnecting from server");
        Ok(())
    }

    pub async fn send_command(&self, command: Command) -> Result<()> {
        tracing::debug!("Sending command: {:?}", command);
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