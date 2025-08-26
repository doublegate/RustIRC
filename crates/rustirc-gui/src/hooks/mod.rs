//! Custom React-like hooks for RustIRC Dioxus components
//!
//! This module provides reusable hooks for common IRC client functionality:
//! - Connection state management
//! - Message history and scrolling
//! - User list management
//! - Theme and preferences
//! - Keyboard shortcuts and input handling

pub mod use_input_handler;
pub mod use_irc_connection;
pub mod use_message_history;
pub mod use_scroll_manager;
pub mod use_theme;
pub mod use_user_list;

pub use use_input_handler::*;
pub use use_irc_connection::*;
pub use use_message_history::*;
pub use use_scroll_manager::*;
pub use use_theme::*;
pub use use_user_list::*;
