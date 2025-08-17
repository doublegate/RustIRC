//! Event system

use rustirc_protocol::Message;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub enum Event {
    // Connection events
    Connected { connection_id: String },
    Disconnected { connection_id: String, reason: String },
    StateChanged { connection_id: String, state: crate::connection::ConnectionState },
    
    // Message events
    MessageReceived { connection_id: String, message: Message },
    MessageSent { connection_id: String, message: Message },
    
    // Channel events
    ChannelJoined { connection_id: String, channel: String },
    ChannelLeft { connection_id: String, channel: String },
    UserJoined { connection_id: String, channel: String, user: String },
    UserLeft { connection_id: String, channel: String, user: String },
    
    // User events  
    NickChanged { connection_id: String, old: String, new: String },
    TopicChanged { connection_id: String, channel: String, topic: String },
    
    // Error events
    Error { connection_id: Option<String>, error: String },
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event);
    fn priority(&self) -> i32 { 0 }
}

pub struct EventBus {
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register<H: EventHandler + 'static>(&self, handler: H) {
        let mut handlers = self.handlers.write().await;
        handlers.push(Box::new(handler));
        handlers.sort_by_key(|h| -h.priority());
    }

    pub async fn emit(&self, event: Event) {
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            handler.handle(&event).await;
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}