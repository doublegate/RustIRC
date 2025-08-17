//! GUI implementation for RustIRC

pub mod app;
pub mod simple_app;
pub mod theme;
pub mod widgets;
pub mod state;
pub mod menus;
pub mod dialogs;
pub mod platform;
pub mod performance;
pub mod accessibility;
pub mod testing;
pub mod formatting;
pub mod event_handler;

pub use app::RustIrcGui;
pub use simple_app::{SimpleApp, run_simple_test};