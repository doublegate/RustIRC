//! GUI implementation for RustIRC

pub mod app;
pub mod simple_gui;
pub mod theme;
pub mod state;
pub mod widgets;
pub mod formatting;
pub mod event_handler;

// Still in development - disabled for now
// pub mod simple_app;
// pub mod menus;
// pub mod dialogs;
// pub mod platform;
// pub mod performance;
// pub mod accessibility;
// pub mod testing;

pub use app::RustIrcGui;
pub use simple_gui::SimpleRustIrcGui;
// pub use simple_app::{SimpleApp, run_simple_test};