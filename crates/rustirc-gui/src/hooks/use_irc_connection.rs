//! Hook for managing IRC connections

use crate::context::{ConnectionInfo, IrcState};
use dioxus::prelude::*;
use rustirc_core::{ClientManager, Config, ConnectionState};
use std::sync::Arc;

/// Connection management hook
#[allow(non_snake_case)]
pub fn use_irc_connection() -> IrcConnectionHook {
    let irc_state = use_context::<IrcState>();
    let client_manager = use_signal(|| Arc::new(ClientManager::new()));
    
    IrcConnectionHook {
        irc_state,
        client_manager: client_manager(),
    }
}

/// IRC connection hook interface
pub struct IrcConnectionHook {
    pub irc_state: IrcState,
    pub client_manager: Arc<ClientManager>,
}

impl IrcConnectionHook {
    /// Connect to an IRC server
    pub async fn connect(
        &self,
        server: String,
        port: u16,
        nickname: String,
        password: Option<String>,
        use_tls: bool,
    ) -> Result<String, String> {
        crate::providers::irc_provider::connect_to_server(
            server,
            port,
            nickname,
            password,
            use_tls,
            &self.irc_state,
            self.client_manager.clone(),
        ).await
    }

    /// Disconnect from an IRC server
    pub async fn disconnect(&self, server_id: String) -> Result<(), String> {
        crate::providers::irc_provider::disconnect_from_server(
            server_id,
            &self.irc_state,
            self.client_manager.clone(),
        ).await
    }

    /// Join a channel
    pub async fn join_channel(&self, server_id: String, channel: String) -> Result<(), String> {
        crate::providers::irc_provider::join_channel(
            server_id,
            channel,
            &self.irc_state,
            self.client_manager.clone(),
        ).await
    }

    /// Send a message
    pub async fn send_message(
        &self,
        server_id: String,
        target: String,
        message: String,
    ) -> Result<(), String> {
        crate::providers::irc_provider::send_message(
            server_id,
            target,
            message,
            &self.irc_state,
            self.client_manager.clone(),
        ).await
    }

    /// Get connection status
    pub fn get_connection_status(&self, server_id: &str) -> Option<ConnectionState> {
        let connections = self.irc_state.connections.read();
        connections.get(server_id).map(|conn| conn.state.clone())
    }

    /// Get all connections
    pub fn get_connections(&self) -> std::collections::HashMap<String, ConnectionInfo> {
        self.irc_state.connections.read().clone()
    }

    /// Get current server ID
    pub fn get_current_server(&self) -> Option<String> {
        self.irc_state.current_server.read().clone()
    }

    /// Get current channel
    pub fn get_current_channel(&self) -> Option<String> {
        self.irc_state.current_channel.read().clone()
    }

    /// Switch to a server/channel
    pub fn switch_to(&self, server_id: Option<String>, channel: Option<String>) {
        if let Some(server) = server_id {
            self.irc_state.current_server.set(Some(server.clone()));
            
            if let Some(ch) = channel {
                self.irc_state.current_channel.set(Some(ch.clone()));
                self.irc_state.active_tab.set(format!("{}:{}", server, ch));
            } else {
                self.irc_state.current_channel.set(None);
                self.irc_state.active_tab.set(server);
            }
        }
    }
}

/// Hook for monitoring connection state changes
#[allow(non_snake_case)]
pub fn use_connection_status(server_id: String) -> ConnectionState {
    let irc_state = use_context::<IrcState>();
    let connections = irc_state.connections.read();
    
    connections
        .get(&server_id)
        .map(|conn| conn.state.clone())
        .unwrap_or(ConnectionState::Disconnected)
}

/// Hook for getting connection info
#[allow(non_snake_case)]
pub fn use_connection_info(server_id: String) -> Option<ConnectionInfo> {
    let irc_state = use_context::<IrcState>();
    let connections = irc_state.connections.read();
    
    connections.get(&server_id).cloned()
}

/// Hook for managing server list
#[allow(non_snake_case)]
pub fn use_server_list() -> Vec<(String, ConnectionInfo)> {
    let irc_state = use_context::<IrcState>();
    let connections = irc_state.connections.read();
    
    connections.iter().map(|(id, info)| (id.clone(), info.clone())).collect()
}

/// Hook for auto-reconnection logic
#[allow(non_snake_case)]
pub fn use_auto_reconnect(server_id: String, enabled: bool) {
    let connection_hook = use_irc_connection();
    
    use_effect(move || {
        if !enabled {
            return move || {};
        }
        
        let connection_hook = connection_hook.clone();
        let server_id = server_id.clone();
        
        spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                
                let status = connection_hook.get_connection_status(&server_id);
                if matches!(status, Some(ConnectionState::Disconnected)) {
                    // TODO: Attempt reconnection with stored credentials
                    tracing::info!("Auto-reconnect attempt for server: {}", server_id);
                    // connection_hook.reconnect(server_id.clone()).await;
                }
            }
        });
        
        move || {
            // Cleanup if needed
        }
    });
}