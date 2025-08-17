//! GUI implementation for RustIRC

pub mod simple_gui;
pub mod theme;
pub mod state;

// Temporarily disabled while updating to Iced 0.13.1
// pub mod app;
// pub mod simple_app;
// pub mod widgets;
// pub mod menus;
// pub mod dialogs;
// pub mod platform;
// pub mod performance;
// pub mod accessibility;
// pub mod testing;
// pub mod formatting;
// pub mod event_handler;

// pub use app::RustIrcGui;
// pub use simple_app::{SimpleApp, run_simple_test};
pub use simple_gui::SimpleRustIrcGui;