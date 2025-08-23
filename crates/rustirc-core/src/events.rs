//! Event system for IRC client communication
//!
//! This module provides a comprehensive event system for handling IRC protocol events,
//! connection state changes, and user interactions. The event system uses an async
//! publish-subscribe pattern with priority-based handler ordering.
//!
//! # Examples
//!
//! ```rust
//! use rustirc_core::events::{EventBus, Event, EventHandler};
//! use async_trait::async_trait;
//!
//! #[tokio::main]
//! async fn main() {
//!     let event_bus = EventBus::new();
//!     
//!     // Emit a connection event
//!     let event = Event::Connected {
//!         connection_id: "irc.libera.chat:6697".to_string(),
//!     };
//!     
//!     event_bus.emit(event).await;
//! }
//! ```

use async_trait::async_trait;
use rustirc_protocol::Message;
use std::sync::Arc;
use tokio::sync::RwLock;

/// IRC client events that can be emitted throughout the application
///
/// Events represent state changes, protocol messages, user actions, and system notifications
/// that occur during IRC client operation. Each event includes relevant context data.
///
/// # Examples
///
/// ```rust
/// use rustirc_core::events::Event;
/// use rustirc_protocol::Message;
///
/// // Create a connection event
/// let event = Event::Connected {
///     connection_id: "server1".to_string(),
/// };
///
/// // Create a message event
/// let message = Message {
///     tags: None,
///     prefix: None,
///     command: "PRIVMSG".to_string(),
///     params: vec!["#channel".to_string(), "Hello, world!".to_string()],
/// };
///
/// let event = Event::MessageReceived {
///     connection_id: "server1".to_string(),
///     message,
/// };
/// ```
#[derive(Debug, Clone)]
pub enum Event {
    // Connection events
    Connected {
        connection_id: String,
    },
    Disconnected {
        connection_id: String,
        reason: String,
    },
    StateChanged {
        connection_id: String,
        state: crate::connection::ConnectionState,
    },

    // Message events
    MessageReceived {
        connection_id: String,
        message: Message,
    },
    MessageSent {
        connection_id: String,
        message: Message,
    },

    // Channel events
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

    // User events
    NickChanged {
        connection_id: String,
        old: String,
        new: String,
    },
    TopicChanged {
        connection_id: String,
        channel: String,
        topic: String,
    },

    // Error events
    Error {
        connection_id: Option<String>,
        error: String,
    },

    // Protocol events
    PongRequired {
        connection_id: String,
        server: String,
    },
}

/// Trait for handling IRC events asynchronously
///
/// Event handlers process events emitted by the EventBus. Handlers can specify
/// a priority to control the order of execution - higher priority handlers run first.
///
/// # Examples
///
/// ```rust
/// use rustirc_core::events::{Event, EventHandler};
/// use async_trait::async_trait;
///
/// struct MyHandler;
///
/// #[async_trait]
/// impl EventHandler for MyHandler {
///     async fn handle(&self, event: &Event) {
///         match event {
///             Event::Connected { connection_id } => {
///                 println!("Connected to: {}", connection_id);
///             },
///             _ => {}
///         }
///     }
///     
///     fn priority(&self) -> i32 {
///         10 // Higher priority than default (0)
///     }
/// }
/// ```
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event asynchronously
    ///
    /// This method is called for each event emitted by the EventBus.
    /// Implementations should be efficient and avoid blocking operations.
    async fn handle(&self, event: &Event);

    /// Return the priority of this handler (higher values = higher priority)
    ///
    /// Handlers are sorted by priority in descending order, so handlers with
    /// higher priority values are executed first.
    fn priority(&self) -> i32 {
        0
    }
}

/// Asynchronous event bus for managing event handlers and publishing events
///
/// The EventBus provides a centralized mechanism for event distribution throughout
/// the IRC client. It supports registering multiple handlers per event type and
/// executes handlers in priority order.
///
/// # Examples
///
/// ```rust
/// use rustirc_core::events::{EventBus, Event, EventHandler};
/// use async_trait::async_trait;
///
/// struct Logger;
///
/// #[async_trait]
/// impl EventHandler for Logger {
///     async fn handle(&self, event: &Event) {
///         println!("Event: {:?}", event);
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let event_bus = EventBus::new();
///     
///     // Register a handler
///     event_bus.register(Logger).await;
///     
///     // Emit an event
///     let event = Event::Connected {
///         connection_id: "test".to_string(),
///     };
///     event_bus.emit(event).await;
/// }
/// ```
pub struct EventBus {
    handlers: Arc<RwLock<Vec<Box<dyn EventHandler>>>>,
}

impl EventBus {
    /// Create a new event bus with no registered handlers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_core::events::EventBus;
    ///
    /// let event_bus = EventBus::new();
    /// ```
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new event handler
    ///
    /// The handler will be added to the handler list and sorted by priority.
    /// Handlers with higher priority values are executed first.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_core::events::{EventBus, Event, EventHandler};
    /// use async_trait::async_trait;
    ///
    /// struct MyHandler;
    ///
    /// #[async_trait]
    /// impl EventHandler for MyHandler {
    ///     async fn handle(&self, event: &Event) {
    ///         // Handle the event
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let event_bus = EventBus::new();
    ///     event_bus.register(MyHandler).await;
    /// }
    /// ```
    pub async fn register<H: EventHandler + 'static>(&self, handler: H) {
        let mut handlers = self.handlers.write().await;
        handlers.push(Box::new(handler));
        handlers.sort_by_key(|h| -h.priority());
    }

    /// Emit an event to all registered handlers
    ///
    /// All handlers will be called in priority order (highest to lowest).
    /// This method waits for each handler to complete before calling the next.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_core::events::{EventBus, Event};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let event_bus = EventBus::new();
    ///     
    ///     let event = Event::Connected {
    ///         connection_id: "irc.libera.chat:6697".to_string(),
    ///     };
    ///     
    ///     event_bus.emit(event).await;
    /// }
    /// ```
    pub async fn emit(&self, event: Event) {
        let handlers = self.handlers.read().await;
        for handler in handlers.iter() {
            handler.handle(&event).await;
        }
    }

    /// Publish an event (alias for emit)
    ///
    /// This method is identical to `emit` and is provided for convenience.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_core::events::{EventBus, Event};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let event_bus = EventBus::new();
    ///     
    ///     let event = Event::Error {
    ///         connection_id: Some("server1".to_string()),
    ///         error: "Connection timeout".to_string(),
    ///     };
    ///     
    ///     event_bus.publish(event).await;
    /// }
    /// ```
    pub async fn publish(&self, event: Event) {
        self.emit(event).await;
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
