//! Scripting API for IRC client automation and customization
//!
//! This module provides the core API that scripts can use to interact with
//! the IRC client. Scripts can handle events, send messages, manage connections,
//! and access client state through this interface.
//!
//! # Examples
//!
//! ```rust
//! use rustirc_scripting::api::ScriptApi;
//!
//! // Create API instance (typically provided by script engine)
//! let api = ScriptApi::new();
//!
//! // Scripts will use API methods to interact with client
//! // api.send_message("#channel", "Hello from script!").await;
//! // api.join_channel("#scripting").await;
//! ```
//!
//! # Script Capabilities
//!
//! When fully implemented, scripts will be able to:
//! - Send and receive IRC messages
//! - Handle connection events
//! - Manage channels and users
//! - Access and modify client settings
//! - Create custom commands and responses
//! - Integrate with external services

use rustirc_core::events::{Event, EventBus};
use std::sync::Arc;

/// Main scripting API interface
///
/// Provides methods for scripts to interact with the IRC client.
/// This is the primary interface exposed to Lua and Python scripts.
///
/// # Examples
///
/// ```rust
/// use rustirc_scripting::api::ScriptApi;
///
/// let api = ScriptApi::new();
///
/// // Future usage in scripts:
/// // api.on_message(|sender, target, text| {
/// //     if text.contains("!help") {
/// //         api.send_message(target, "Available commands: !help, !time");
/// //     }
/// // });
/// ```
pub struct ScriptApi {
    /// Event bus for receiving and sending events.
    ///
    /// Reserved for Phase 4 implementation. Will be used to:
    /// - Subscribe to IRC events (messages, joins, parts, etc.)
    /// - Publish script-generated events to the client
    /// - Enable bidirectional communication between scripts and the IRC engine
    #[allow(dead_code)]
    event_bus: Option<Arc<EventBus>>,

    /// Connection ID for script context.
    ///
    /// Reserved for Phase 4 implementation. Will be used to:
    /// - Associate scripts with specific server connections
    /// - Route script commands to the correct IRC connection
    /// - Enable per-connection script state and configuration
    #[allow(dead_code)]
    connection_id: Option<String>,
}

impl ScriptApi {
    /// Create a new script API instance
    ///
    /// In Phase 4 implementation, this will be connected to the actual IRC client.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_scripting::api::ScriptApi;
    ///
    /// let api = ScriptApi::new();
    /// ```
    pub fn new() -> Self {
        Self {
            event_bus: None,
            connection_id: None,
        }
    }

    /// Create API instance with event bus connection
    ///
    /// Used internally by the script engine to provide scripts with
    /// access to the client's event system.
    ///
    /// # Arguments
    ///
    /// * `event_bus` - Shared event bus for client communication
    /// * `connection_id` - ID of the connection this script is associated with
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_scripting::api::ScriptApi;
    /// use rustirc_core::events::EventBus;
    /// use std::sync::Arc;
    ///
    /// let event_bus = Arc::new(EventBus::new());
    /// let api = ScriptApi::with_event_bus(event_bus, "irc.example.com".to_string());
    /// ```
    pub fn with_event_bus(event_bus: Arc<EventBus>, connection_id: String) -> Self {
        Self {
            event_bus: Some(event_bus),
            connection_id: Some(connection_id),
        }
    }

    /// Send a message to a channel or user
    ///
    /// This is a placeholder for Phase 4 implementation.
    ///
    /// # Arguments
    ///
    /// * `target` - Channel name (e.g., "#rust") or nickname
    /// * `message` - Text to send
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rustirc_scripting::api::ScriptApi;
    ///
    /// # async fn example() -> Result<(), String> {
    /// let api = ScriptApi::new();
    ///
    /// // Will be implemented in Phase 4
    /// api.send_message("#rust", "Hello from script!").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if message was sent successfully, or an error if sending failed.
    pub async fn send_message(&self, _target: &str, _message: &str) -> Result<(), String> {
        // Phase 4: Send PRIVMSG command through event bus
        Err("Scripting API will be implemented in Phase 4".to_string())
    }

    /// Join an IRC channel
    ///
    /// This is a placeholder for Phase 4 implementation.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel name to join (e.g., "#rust")
    /// * `key` - Optional channel key/password
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rustirc_scripting::api::ScriptApi;
    ///
    /// # async fn example() -> Result<(), String> {
    /// let api = ScriptApi::new();
    ///
    /// // Will be implemented in Phase 4
    /// api.join_channel("#rust", None).await?;
    /// api.join_channel("#private", Some("secret123")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn join_channel(&self, _channel: &str, _key: Option<&str>) -> Result<(), String> {
        // Phase 4: Send JOIN command through event bus
        Err("Scripting API will be implemented in Phase 4".to_string())
    }

    /// Leave an IRC channel
    ///
    /// This is a placeholder for Phase 4 implementation.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel name to leave
    /// * `reason` - Optional part message
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rustirc_scripting::api::ScriptApi;
    ///
    /// # async fn example() -> Result<(), String> {
    /// let api = ScriptApi::new();
    ///
    /// // Will be implemented in Phase 4
    /// api.leave_channel("#rust", Some("Goodbye!")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn leave_channel(&self, _channel: &str, _reason: Option<&str>) -> Result<(), String> {
        // Phase 4: Send PART command through event bus
        Err("Scripting API will be implemented in Phase 4".to_string())
    }

    /// Register an event handler callback
    ///
    /// This is a placeholder for Phase 4 implementation.
    /// Scripts will use this to respond to IRC events.
    ///
    /// # Arguments
    ///
    /// * `event_type` - Type of event to handle ("message", "join", "part", etc.)
    /// * `callback` - Function to call when event occurs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_scripting::api::ScriptApi;
    ///
    /// let api = ScriptApi::new();
    ///
    /// // Will be implemented in Phase 4
    /// // api.on_event("message", |event| {
    /// //     println!("Received message: {:?}", event);
    /// // });
    /// ```
    pub fn on_event(
        &self,
        _event_type: &str,
        _callback: Box<dyn Fn(&Event)>,
    ) -> Result<(), String> {
        // Phase 4: Register callback with event bus
        Err("Scripting API will be implemented in Phase 4".to_string())
    }

    /// Get list of channels the client is currently in
    ///
    /// This is a placeholder for Phase 4 implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_scripting::api::ScriptApi;
    ///
    /// let api = ScriptApi::new();
    ///
    /// // Will be implemented in Phase 4
    /// // let channels = api.get_channels();
    /// // println!("Currently in {} channels", channels.len());
    /// ```
    pub fn get_channels(&self) -> Vec<String> {
        // Phase 4: Return list of joined channels
        vec![] // Placeholder
    }

    /// Get list of users in a channel
    ///
    /// This is a placeholder for Phase 4 implementation.
    ///
    /// # Arguments
    ///
    /// * `channel` - Channel name to get users from
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_scripting::api::ScriptApi;
    ///
    /// let api = ScriptApi::new();
    ///
    /// // Will be implemented in Phase 4
    /// // let users = api.get_channel_users("#rust");
    /// // println!("Channel has {} users", users.len());
    /// ```
    pub fn get_channel_users(&self, _channel: &str) -> Vec<String> {
        // Phase 4: Return list of users in channel
        vec![] // Placeholder
    }
}

impl Default for ScriptApi {
    fn default() -> Self {
        Self::new()
    }
}
