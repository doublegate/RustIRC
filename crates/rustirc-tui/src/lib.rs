//! TUI implementation for RustIRC

pub mod app;
pub mod ui;
pub mod input;
pub mod state;
pub mod formatting;
pub mod themes;
pub mod event_handler;

pub use app::TuiApp;