//! Plugin system for RustIRC

pub mod manager;
pub mod api;
pub mod loader;

pub use manager::PluginManager;
pub use api::PluginApi;