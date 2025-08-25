//! Hook for managing message history and filtering

use crate::context::{ChatMessage, IrcState, MessageType};
use dioxus::prelude::*;

/// Message history management hook
#[allow(non_snake_case)]
pub fn use_message_history(server_id: String, channel: String) -> MessageHistoryHook {
    let irc_state = use_context::<IrcState>();
    
    MessageHistoryHook {
        irc_state,
        server_id,
        channel,
    }
}

/// Message history hook interface
pub struct MessageHistoryHook {
    pub irc_state: IrcState,
    pub server_id: String,
    pub channel: String,
}

impl MessageHistoryHook {
    /// Get messages for the current channel
    pub fn get_messages(&self) -> Vec<ChatMessage> {
        let connections = self.irc_state.connections.read();
        
        if let Some(connection) = connections.get(&self.server_id) {
            if let Some(channel_info) = connection.channels.get(&self.channel) {
                return channel_info.messages.clone();
            }
        }
        
        Vec::new()
    }

    /// Get filtered messages based on preferences
    pub fn get_filtered_messages(&self, show_system: bool, show_joins_parts: bool) -> Vec<ChatMessage> {
        let messages = self.get_messages();
        
        messages.into_iter()
            .filter(|msg| self.should_show_message(msg, show_system, show_joins_parts))
            .collect()
    }

    /// Add a new message
    pub fn add_message(
        &self,
        sender: Option<String>,
        content: String,
        msg_type: MessageType,
    ) {
        self.irc_state.add_message(
            self.server_id.clone(),
            self.channel.clone(),
            sender,
            content,
            msg_type,
        );
    }

    /// Search messages by content
    pub fn search_messages(&self, query: &str) -> Vec<ChatMessage> {
        let messages = self.get_messages();
        let query_lower = query.to_lowercase();
        
        messages.into_iter()
            .filter(|msg| {
                msg.content.to_lowercase().contains(&query_lower) ||
                msg.sender.as_ref().map_or(false, |s| s.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get message count
    pub fn get_message_count(&self) -> usize {
        self.get_messages().len()
    }

    /// Get unread count
    pub fn get_unread_count(&self) -> usize {
        let connections = self.irc_state.connections.read();
        
        if let Some(connection) = connections.get(&self.server_id) {
            if let Some(channel_info) = connection.channels.get(&self.channel) {
                return channel_info.unread_count;
            }
        }
        
        0
    }

    /// Clear unread count
    pub fn clear_unread_count(&self) {
        if let Some(connection) = self.irc_state.connections.write().get_mut(&self.server_id) {
            if let Some(channel_info) = connection.channels.get_mut(&self.channel) {
                channel_info.unread_count = 0;
            }
        }
    }

    /// Get messages from a specific time range
    pub fn get_messages_in_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<ChatMessage> {
        let messages = self.get_messages();
        
        messages.into_iter()
            .filter(|msg| msg.timestamp >= start && msg.timestamp <= end)
            .collect()
    }

    /// Get messages from a specific user
    pub fn get_messages_from_user(&self, username: &str) -> Vec<ChatMessage> {
        let messages = self.get_messages();
        
        messages.into_iter()
            .filter(|msg| {
                msg.sender.as_ref().map_or(false, |s| s.eq_ignore_ascii_case(username))
            })
            .collect()
    }

    fn should_show_message(&self, message: &ChatMessage, show_system: bool, show_joins_parts: bool) -> bool {
        match message.message_type {
            MessageType::Normal | MessageType::Action | MessageType::Error => true,
            MessageType::Join | MessageType::Part | MessageType::Quit => show_joins_parts,
            MessageType::System | MessageType::Nick | MessageType::Topic => show_system,
        }
    }
}

/// Hook for real-time message updates
#[allow(non_snake_case)]
pub fn use_message_updates(server_id: String, channel: String) -> Signal<usize> {
    let irc_state = use_context::<IrcState>();
    let mut update_counter = use_signal(|| 0usize);
    
    // Watch for message updates
    use_effect(move || {
        let connections = irc_state.connections.read();
        
        if let Some(connection) = connections.get(&server_id) {
            if let Some(channel_info) = connection.channels.get(&channel) {
                let current_count = channel_info.messages.len();
                if current_count != update_counter() {
                    update_counter.set(current_count);
                }
            }
        }
    });
    
    update_counter
}

/// Hook for message pagination
#[allow(non_snake_case)]
pub fn use_message_pagination(server_id: String, channel: String, page_size: usize) -> MessagePaginationHook {
    let irc_state = use_context::<IrcState>();
    let mut current_page = use_signal(|| 0usize);
    
    MessagePaginationHook {
        irc_state,
        server_id,
        channel,
        current_page,
        page_size,
    }
}

/// Message pagination hook interface
pub struct MessagePaginationHook {
    pub irc_state: IrcState,
    pub server_id: String,
    pub channel: String,
    pub current_page: Signal<usize>,
    pub page_size: usize,
}

impl MessagePaginationHook {
    /// Get current page of messages
    pub fn get_current_page(&self) -> Vec<ChatMessage> {
        let history_hook = use_message_history(self.server_id.clone(), self.channel.clone());
        let messages = history_hook.get_messages();
        
        let start = self.current_page() * self.page_size;
        let end = (start + self.page_size).min(messages.len());
        
        if start < messages.len() {
            messages[start..end].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Go to next page
    pub fn next_page(&self) {
        let history_hook = use_message_history(self.server_id.clone(), self.channel.clone());
        let total_messages = history_hook.get_message_count();
        let total_pages = (total_messages + self.page_size - 1) / self.page_size;
        
        if self.current_page() + 1 < total_pages {
            self.current_page.set(self.current_page() + 1);
        }
    }

    /// Go to previous page
    pub fn prev_page(&self) {
        if self.current_page() > 0 {
            self.current_page.set(self.current_page() - 1);
        }
    }

    /// Go to specific page
    pub fn go_to_page(&self, page: usize) {
        let history_hook = use_message_history(self.server_id.clone(), self.channel.clone());
        let total_messages = history_hook.get_message_count();
        let total_pages = (total_messages + self.page_size - 1) / self.page_size;
        
        if page < total_pages {
            self.current_page.set(page);
        }
    }

    /// Get total page count
    pub fn get_total_pages(&self) -> usize {
        let history_hook = use_message_history(self.server_id.clone(), self.channel.clone());
        let total_messages = history_hook.get_message_count();
        (total_messages + self.page_size - 1) / self.page_size
    }

    /// Check if there are more pages
    pub fn has_next_page(&self) -> bool {
        self.current_page() + 1 < self.get_total_pages()
    }

    /// Check if there are previous pages
    pub fn has_prev_page(&self) -> bool {
        self.current_page() > 0
    }
}