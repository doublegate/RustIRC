//! GUI implementation for RustIRC using Dioxus

pub mod app;
pub mod components;
pub mod formatting;
pub mod hooks;
pub mod providers;
pub mod state;

pub use app::run as run_gui;
