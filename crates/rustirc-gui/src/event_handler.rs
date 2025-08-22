//! Event handler for bridging core IRC events to GUI messages
//!
//! This module provides the EventHandler implementation that subscribes to
//! core IRC events and translates them into GUI messages for the application.

use crate::app::Message;
use async_trait::async_trait;
use rustirc_core::events::{Event, EventHandler};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// GUI event handler that translates core events to GUI messages
pub struct GuiEventHandler {
    message_sender: mpsc::UnboundedSender<Message>,
}

impl GuiEventHandler {
    pub fn new(message_sender: mpsc::UnboundedSender<Message>) -> Self {
        Self { message_sender }
    }

    /// Send a GUI message
    fn send_message(&self, message: Message) {
        if let Err(e) = self.message_sender.send(message) {
            error!("Failed to send GUI message: {}", e);
        }
    }
}

#[async_trait]
impl EventHandler for GuiEventHandler {
    async fn handle(&self, event: &Event) {
        debug!("Handling core event: {:?}", event);

        match event {
            Event::Connected { connection_id } => {
                info!("Connection established: {}", connection_id);
                self.send_message(Message::CoreEvent(CoreEventMessage::Connected {
                    connection_id: connection_id.clone(),
                }));
            }

            Event::Disconnected {
                connection_id,
                reason,
            } => {
                info!("Connection lost: {} - {}", connection_id, reason);
                self.send_message(Message::CoreEvent(CoreEventMessage::Disconnected {
                    connection_id: connection_id.clone(),
                    reason: reason.clone(),
                }));
            }

            Event::MessageReceived {
                connection_id,
                message,
            } => {
                debug!("Message received from {}: {:?}", connection_id, message);
                self.send_message(Message::CoreEvent(CoreEventMessage::MessageReceived {
                    connection_id: connection_id.clone(),
                    message: message.clone(),
                }));
            }

            Event::ChannelJoined {
                connection_id,
                channel,
            } => {
                info!("Joined channel {} on {}", channel, connection_id);
                self.send_message(Message::CoreEvent(CoreEventMessage::ChannelJoined {
                    connection_id: connection_id.clone(),
                    channel: channel.clone(),
                }));
            }

            Event::ChannelLeft {
                connection_id,
                channel,
            } => {
                info!("Left channel {} on {}", channel, connection_id);
                self.send_message(Message::CoreEvent(CoreEventMessage::ChannelLeft {
                    connection_id: connection_id.clone(),
                    channel: channel.clone(),
                }));
            }

            Event::UserJoined {
                connection_id,
                channel,
                user,
            } => {
                debug!("User {} joined {} on {}", user, channel, connection_id);
                self.send_message(Message::CoreEvent(CoreEventMessage::UserJoined {
                    connection_id: connection_id.clone(),
                    channel: channel.clone(),
                    user: user.clone(),
                }));
            }

            Event::UserLeft {
                connection_id,
                channel,
                user,
            } => {
                debug!("User {} left {} on {}", user, channel, connection_id);
                self.send_message(Message::CoreEvent(CoreEventMessage::UserLeft {
                    connection_id: connection_id.clone(),
                    channel: channel.clone(),
                    user: user.clone(),
                }));
            }

            Event::NickChanged {
                connection_id,
                old,
                new,
            } => {
                info!("Nick changed from {} to {} on {}", old, new, connection_id);
                self.send_message(Message::CoreEvent(CoreEventMessage::NickChanged {
                    connection_id: connection_id.clone(),
                    old_nick: old.clone(),
                    new_nick: new.clone(),
                }));
            }

            Event::TopicChanged {
                connection_id,
                channel,
                topic,
            } => {
                info!(
                    "Topic changed in {} on {}: {}",
                    channel, connection_id, topic
                );
                self.send_message(Message::CoreEvent(CoreEventMessage::TopicChanged {
                    connection_id: connection_id.clone(),
                    channel: channel.clone(),
                    topic: topic.clone(),
                }));
            }

            Event::Error {
                connection_id,
                error,
            } => {
                error!("IRC error on {:?}: {}", connection_id, error);
                self.send_message(Message::CoreEvent(CoreEventMessage::Error {
                    connection_id: connection_id.clone(),
                    error: error.clone(),
                }));
            }

            Event::StateChanged {
                connection_id,
                state,
            } => {
                debug!(
                    "Connection state changed for {}: {:?}",
                    connection_id, state
                );
                self.send_message(Message::CoreEvent(CoreEventMessage::StateChanged {
                    connection_id: connection_id.clone(),
                    state: state.clone(),
                }));
            }

            Event::MessageSent {
                connection_id,
                message,
            } => {
                debug!("Message sent to {}: {:?}", connection_id, message);
                self.send_message(Message::CoreEvent(CoreEventMessage::MessageSent {
                    connection_id: connection_id.clone(),
                    message: message.clone(),
                }));
            }

            Event::PongRequired {
                connection_id,
                server,
            } => {
                debug!("Pong required for {} to server {}", connection_id, server);
                // Pong is automatically handled by the IRC client, just log it
            }
        }
    }

    fn priority(&self) -> i32 {
        100 // High priority for GUI updates
    }
}

/// Core event messages that can be sent to the GUI
#[derive(Debug, Clone)]
pub enum CoreEventMessage {
    Connected {
        connection_id: String,
    },
    Disconnected {
        connection_id: String,
        reason: String,
    },
    MessageReceived {
        connection_id: String,
        message: rustirc_protocol::Message,
    },
    MessageSent {
        connection_id: String,
        message: rustirc_protocol::Message,
    },
    ChannelJoined {
        connection_id: String,
        channel: String,
    },
    ChannelLeft {
        connection_id: String,
        channel: String,
    },
    UserJoined {
        connection_id: String,
        channel: String,
        user: String,
    },
    UserLeft {
        connection_id: String,
        channel: String,
        user: String,
    },
    NickChanged {
        connection_id: String,
        old_nick: String,
        new_nick: String,
    },
    TopicChanged {
        connection_id: String,
        channel: String,
        topic: String,
    },
    Error {
        connection_id: Option<String>,
        error: String,
    },
    StateChanged {
        connection_id: String,
        state: rustirc_core::connection::ConnectionState,
    },
}
