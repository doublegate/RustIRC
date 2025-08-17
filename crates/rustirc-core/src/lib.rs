//! Core functionality for RustIRC client
//!
//! This crate provides the core IRC client functionality including:
//! - Connection management
//! - Message handling
//! - State management
//! - Event system

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod client;
pub mod config;
pub mod connection;
pub mod error;
pub mod events;
pub mod recovery;
pub mod router;
pub mod state;
pub mod ui;

pub use client::IrcClient;
pub use config::Config;
pub use connection::{ConnectionManager, IrcConnection, ConnectionConfig, ConnectionState};
pub use error::{Error, Result};
pub use events::{Event, EventHandler};
pub use recovery::{RecoveryManager, ReconnectConfig, RecoveryStats};
pub use router::{MessageRouter, MessageHandler, MessageContext, CommandProcessor};
pub use state::{ClientState, ServerState, ChannelState, StateManager, User, ChannelUser, TopicInfo};
pub use ui::{UserInterface, UiEvent, StateChange, ViewManager, View, ViewId, ViewType};

/// Global client instance manager
pub struct ClientManager {
    clients: Arc<RwLock<HashMap<String, Arc<IrcClient>>>>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_client(&self, id: String, config: Config) -> Result<Arc<IrcClient>> {
        let client = Arc::new(IrcClient::new(config));
        self.clients.write().await.insert(id.clone(), client.clone());
        Ok(client)
    }

    pub async fn get_client(&self, id: &str) -> Option<Arc<IrcClient>> {
        self.clients.read().await.get(id).cloned()
    }

    pub async fn remove_client(&self, id: &str) -> Option<Arc<IrcClient>> {
        self.clients.write().await.remove(id)
    }

    pub async fn list_clients(&self) -> Vec<String> {
        self.clients.read().await.keys().cloned().collect()
    }
}

impl Default for ClientManager {
    fn default() -> Self {
        Self::new()
    }
}