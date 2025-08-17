//! Event handler for bridging core IRC events to TUI state updates
//!
//! This module provides the EventHandler implementation that subscribes to
//! core IRC events and translates them into TUI state updates.

use crate::state::TuiState;
use rustirc_core::events::{Event, EventHandler};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// TUI event handler that translates core events to TUI state updates
pub struct TuiEventHandler {
    tui_state: Arc<RwLock<TuiState>>,
}

impl TuiEventHandler {
    pub fn new(tui_state: Arc<RwLock<TuiState>>) -> Self {
        Self { tui_state }
    }
}

#[async_trait]
impl EventHandler for TuiEventHandler {
    async fn handle(&self, event: &Event) {
        debug!("TUI handling core event: {:?}", event);
        
        let mut state = self.tui_state.write().await;
        
        match event {
            Event::Connected { connection_id } => {
                info!("TUI: Connection established: {}", connection_id);
                state.add_server(connection_id.clone());
            }
            
            Event::Disconnected { connection_id, reason } => {
                info!("TUI: Connection lost: {} - {}", connection_id, reason);
                state.remove_server(connection_id);
            }
            
            Event::MessageReceived { connection_id, message } => {
                debug!("TUI: Message received from {}: {:?}", connection_id, message);
                
                // Convert IRC message to TUI message format
                match message.command.as_str() {
                    "PRIVMSG" => {
                        if !message.params.is_empty() {
                            let target = &message.params[0];
                            let content = message.params.get(1).unwrap_or(&String::new()).clone();
                            
                            if let Some(ref prefix) = message.prefix {
                                if let rustirc_protocol::message::Prefix::User { nick, .. } = prefix {
                                    if target.starts_with('#') {
                                        // Channel message
                                        state.add_message(
                                            connection_id.clone(),
                                            target.clone(),
                                            nick.clone(),
                                            content,
                                        );
                                    } else {
                                        // Private message
                                        state.add_message(
                                            connection_id.clone(),
                                            nick.clone(), // Use nick as target for PM
                                            nick.clone(),
                                            content,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    "NOTICE" => {
                        if !message.params.is_empty() {
                            let target = &message.params[0];
                            let content = format!("NOTICE: {}", message.params.get(1).unwrap_or(&String::new()));
                            
                            if let Some(ref prefix) = message.prefix {
                                if let rustirc_protocol::message::Prefix::User { nick, .. } = prefix {
                                    state.add_message(
                                        connection_id.clone(),
                                        target.clone(),
                                        nick.clone(),
                                        content,
                                    );
                                }
                            }
                        }
                    }
                    "JOIN" => {
                        if !message.params.is_empty() {
                            let channel = &message.params[0];
                            if let Some(ref prefix) = message.prefix {
                                if let rustirc_protocol::message::Prefix::User { nick, .. } = prefix {
                                    state.add_message(
                                        connection_id.clone(),
                                        channel.clone(),
                                        "*".to_string(),
                                        format!("{} has joined {}", nick, channel),
                                    );
                                }
                            }
                        }
                    }
                    "PART" => {
                        if !message.params.is_empty() {
                            let channel = &message.params[0];
                            let reason = message.params.get(1).cloned().unwrap_or_default();
                            if let Some(ref prefix) = message.prefix {
                                if let rustirc_protocol::message::Prefix::User { nick, .. } = prefix {
                                    let content = if reason.is_empty() {
                                        format!("{} has left {}", nick, channel)
                                    } else {
                                        format!("{} has left {} ({})", nick, channel, reason)
                                    };
                                    state.add_message(
                                        connection_id.clone(),
                                        channel.clone(),
                                        "*".to_string(),
                                        content,
                                    );
                                }
                            }
                        }
                    }
                    "QUIT" => {
                        let reason = message.params.get(0).cloned().unwrap_or_default();
                        if let Some(ref prefix) = message.prefix {
                            if let rustirc_protocol::message::Prefix::User { nick, .. } = prefix {
                                // Add quit message to all channels where the user was present
                                let content = if reason.is_empty() {
                                    format!("{} has quit", nick)
                                } else {
                                    format!("{} has quit ({})", nick, reason)
                                };
                                
                                // For now, just add to current channel
                                if let Some(current_channel) = state.current_channel().cloned() {
                                    state.add_message(
                                        connection_id.clone(),
                                        current_channel,
                                        "*".to_string(),
                                        content,
                                    );
                                }
                            }
                        }
                    }
                    _ => {
                        debug!("TUI: Unhandled IRC message command: {}", message.command);
                    }
                }
            }
            
            Event::ChannelJoined { connection_id, channel } => {
                info!("TUI: Joined channel {} on {}", channel, connection_id);
                state.add_channel(connection_id.clone(), channel.clone());
            }
            
            Event::ChannelLeft { connection_id, channel } => {
                info!("TUI: Left channel {} on {}", channel, connection_id);
                state.remove_channel(connection_id, channel);
            }
            
            Event::UserJoined { connection_id, channel, user } => {
                debug!("TUI: User {} joined {} on {}", user, channel, connection_id);
                // User list management could be added here
            }
            
            Event::UserLeft { connection_id, channel, user } => {
                debug!("TUI: User {} left {} on {}", user, channel, connection_id);
                // User list management could be added here
            }
            
            Event::NickChanged { connection_id, old, new } => {
                info!("TUI: Nick changed from {} to {} on {}", old, new, connection_id);
                // Nick change handling could be added here
            }
            
            Event::TopicChanged { connection_id, channel, topic } => {
                info!("TUI: Topic changed in {} on {}: {}", channel, connection_id, topic);
                // Topic handling could be added here
            }
            
            Event::Error { connection_id, error } => {
                error!("TUI: IRC error on {:?}: {}", connection_id, error);
                if let Some(current_channel) = state.current_channel().cloned() {
                    state.add_message(
                        connection_id.clone().unwrap_or_default(),
                        current_channel,
                        "*".to_string(),
                        format!("ERROR: {}", error),
                    );
                }
            }
            
            Event::StateChanged { connection_id, state: _state } => {
                debug!("TUI: Connection state changed for {}", connection_id);
                // Connection state changes could be reflected in the UI
            }
            
            Event::MessageSent { connection_id, message } => {
                debug!("TUI: Message sent to {}: {:?}", connection_id, message);
                // Sent messages are typically already handled by the input system
            }
        }
    }
    
    fn priority(&self) -> i32 {
        50 // Medium priority for TUI updates
    }
}