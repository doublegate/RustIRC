//! Scripting API for IRC client automation and customization
//!
//! Provides the bridge between Lua scripts and the IRC client internals.
//! Scripts interact with the client through the `irc` table set up by
//! the ScriptEngine rather than through this struct directly.

use rustirc_core::events::{Event, EventBus};
use rustirc_core::state::ClientState;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Bridge between scripts and IRC client state
pub struct ScriptApi {
    event_bus: Option<Arc<EventBus>>,
    connection_id: Option<String>,
    state: Option<Arc<RwLock<ClientState>>>,
}

impl ScriptApi {
    pub fn new() -> Self {
        Self {
            event_bus: None,
            connection_id: None,
            state: None,
        }
    }

    pub fn with_event_bus(event_bus: Arc<EventBus>, connection_id: String) -> Self {
        Self {
            event_bus: Some(event_bus),
            connection_id: Some(connection_id),
            state: None,
        }
    }

    pub fn with_state(
        event_bus: Arc<EventBus>,
        connection_id: String,
        state: Arc<RwLock<ClientState>>,
    ) -> Self {
        Self {
            event_bus: Some(event_bus),
            connection_id: Some(connection_id),
            state: Some(state),
        }
    }

    /// Send a PRIVMSG through the event bus
    pub async fn send_message(&self, target: &str, message: &str) -> Result<(), String> {
        let event_bus = self
            .event_bus
            .as_ref()
            .ok_or_else(|| "No event bus connected".to_string())?;
        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| "No connection ID".to_string())?;

        let msg = rustirc_protocol::Message::new("PRIVMSG")
            .add_param(target)
            .add_param(message);

        event_bus
            .emit(Event::MessageSent {
                connection_id: connection_id.clone(),
                message: msg,
            })
            .await;

        Ok(())
    }

    /// Send a JOIN command
    pub async fn join_channel(&self, channel: &str, _key: Option<&str>) -> Result<(), String> {
        let event_bus = self
            .event_bus
            .as_ref()
            .ok_or_else(|| "No event bus connected".to_string())?;
        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| "No connection ID".to_string())?;

        let msg = rustirc_protocol::Message::new("JOIN").add_param(channel);

        event_bus
            .emit(Event::MessageSent {
                connection_id: connection_id.clone(),
                message: msg,
            })
            .await;

        Ok(())
    }

    /// Send a PART command
    pub async fn leave_channel(&self, channel: &str, reason: Option<&str>) -> Result<(), String> {
        let event_bus = self
            .event_bus
            .as_ref()
            .ok_or_else(|| "No event bus connected".to_string())?;
        let connection_id = self
            .connection_id
            .as_ref()
            .ok_or_else(|| "No connection ID".to_string())?;

        let mut msg = rustirc_protocol::Message::new("PART").add_param(channel);
        if let Some(reason) = reason {
            msg = msg.add_param(reason);
        }

        event_bus
            .emit(Event::MessageSent {
                connection_id: connection_id.clone(),
                message: msg,
            })
            .await;

        Ok(())
    }

    /// Subscribe to events on the event bus
    pub fn on_event(
        &self,
        _event_type: &str,
        _callback: Box<dyn Fn(&Event)>,
    ) -> Result<(), String> {
        // Event handling is done through Lua handlers registered via irc.register_handler
        Ok(())
    }

    /// Get list of channels from client state
    pub async fn get_channels(&self) -> Vec<String> {
        if let Some(state) = &self.state {
            let state = state.read().await;
            state
                .servers
                .values()
                .flat_map(|s| s.channels.keys().cloned())
                .collect()
        } else {
            vec![]
        }
    }

    /// Get list of users in a channel from client state
    pub async fn get_channel_users(&self, channel: &str) -> Vec<String> {
        if let Some(state) = &self.state {
            let state = state.read().await;
            for server in state.servers.values() {
                if let Some(ch) = server.channels.get(channel) {
                    return ch.users.values().map(|u| u.nick.clone()).collect();
                }
            }
        }
        vec![]
    }
}

impl Default for ScriptApi {
    fn default() -> Self {
        Self::new()
    }
}
