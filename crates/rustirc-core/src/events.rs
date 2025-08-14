//! Event system

use rustirc_protocol::Message;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub enum Event {
    Connected { server: String },
    Disconnected { server: String },
    MessageReceived(Message),
    MessageSent(Message),
    ChannelJoined { channel: String },
    ChannelLeft { channel: String },
    UserJoined { channel: String, user: String },
    UserLeft { channel: String, user: String },
    NickChanged { old: String, new: String },
    TopicChanged { channel: String, topic: String },
    Error(String),
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