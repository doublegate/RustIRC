//! Plugin system for RustIRC

pub mod api;
pub mod builtin;
pub mod loader;
pub mod manager;

pub use api::PluginApi;
pub use manager::PluginManager;
