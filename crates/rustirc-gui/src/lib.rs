//! GUI implementation for RustIRC

pub mod app;
pub mod simple_gui;
pub mod theme;
pub mod state;
pub mod widgets;
pub mod formatting;
pub mod event_handler;
pub mod menus;
pub mod dialogs;
pub mod platform;
pub mod performance;
pub mod accessibility;
pub mod testing;

// Deprecated simple app - being phased out
// pub mod simple_app;

pub use app::RustIrcGui;
pub use simple_gui::SimpleRustIrcGui;
pub use menus::{MenuBar, MenuMessage, ContextMenu};
pub use dialogs::{DialogManager, DialogMessage, DialogType};
pub use platform::NotificationManager;

// pub use simple_app::{SimpleApp, run_simple_test};