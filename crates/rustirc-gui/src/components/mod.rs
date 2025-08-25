//! Dioxus Components for RustIRC
//!
//! Modern React-like components with virtual DOM architecture

// Dioxus components
pub mod context_menu;
pub mod dialogs;
pub mod input_area;
pub mod message_view;
pub mod sidebar;
pub mod status_bar;
pub mod tab_bar;
pub mod user_list;

// Re-export commonly used Dioxus components
pub use context_menu::ContextMenu;
pub use dialogs::DialogManager;
pub use input_area::InputArea;
pub use message_view::MessageView;
pub use sidebar::Sidebar;
pub use status_bar::StatusBar;
pub use tab_bar::TabBar;
pub use user_list::UserList;
