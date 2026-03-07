//! Event bridge between rustirc-core EventBus and Dioxus
//!
//! Implements an EventHandler that forwards events to a tokio channel,
//! which the Dioxus coroutine reads from to update Signal<AppState>.

use async_trait::async_trait;
use rustirc_core::events::{Event, EventHandler};
use tokio::sync::mpsc::UnboundedSender;

/// Bridges the core EventBus (trait-based) to a channel the Dioxus coroutine reads from.
pub struct ChannelEventHandler {
    sender: UnboundedSender<Event>,
}

impl ChannelEventHandler {
    pub fn new(sender: UnboundedSender<Event>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl EventHandler for ChannelEventHandler {
    async fn handle(&self, event: &Event) {
        let _ = self.sender.send(event.clone());
    }
}
