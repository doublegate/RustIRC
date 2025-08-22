//! GUI widgets for RustIRC
//!
//! This module contains all the custom widgets used in the RustIRC GUI:
//! - ServerTree: Hierarchical server and channel tree view
//! - MessageView: IRC message display with formatting and scrolling
//! - InputArea: Message input with history and auto-completion
//! - UserList: Channel user list with modes and status
//! - TabBar: Tab management for channels and private messages
//! - StatusBar: Connection and channel status information

pub mod input_area;
pub mod message_view;
pub mod server_tree;
pub mod status_bar;
pub mod tab_bar;
pub mod user_list;

pub use input_area::{InputArea, InputAreaMessage};
pub use message_view::{MessageView, MessageViewMessage};
pub use server_tree::{ServerTree, ServerTreeMessage};
pub use status_bar::{StatusBar, StatusBarMessage};
pub use tab_bar::{TabBar, TabBarMessage};
pub use user_list::{UserList, UserListMessage};
