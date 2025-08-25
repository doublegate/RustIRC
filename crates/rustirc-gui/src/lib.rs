//! GUI implementation for RustIRC

pub mod accessibility;
pub mod app;
pub mod dialogs;
pub mod event_handler;
pub mod formatting;
pub mod menus;
pub mod performance;
pub mod platform;
pub mod state;
pub mod testing;
pub mod theme;
pub mod widgets;

// Dioxus components and architecture
pub mod components;
pub mod context;
pub mod dioxus_app;

// Deprecated simple app - being phased out
// pub mod simple_app;

pub use app::RustIrcGui;
pub use dialogs::{DialogManager, DialogMessage, DialogType};
pub use menus::{ContextMenu, MenuBar, MenuMessage};
pub use platform::NotificationManager;

// Dioxus exports
pub use dioxus_app::launch_app;
pub use context::{IrcState, ThemeState, UiState, ContextProvider};

// pub use simple_app::{SimpleApp, run_simple_test};
