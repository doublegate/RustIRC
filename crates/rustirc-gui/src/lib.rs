//! GUI implementation for RustIRC

pub mod accessibility;
pub mod app;
pub mod components;
pub mod dialogs;
pub mod event_handler;
pub mod formatting;
pub mod material_demo;
pub mod menus;
pub mod performance;
pub mod platform;
pub mod state;
pub mod testing;
pub mod theme;
pub mod themes;
pub mod widgets;

// Deprecated simple app - being phased out
// pub mod simple_app;

pub use app::RustIrcGui;
pub use dialogs::{DialogManager, DialogMessage, DialogType};
pub use menus::{ContextMenu, MenuBar, MenuMessage};
pub use platform::NotificationManager;

// pub use simple_app::{SimpleApp, run_simple_test};
