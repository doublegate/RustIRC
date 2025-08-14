//! Scripting engine for RustIRC

pub mod engine;
pub mod api;
pub mod sandbox;

pub use engine::ScriptEngine;
pub use api::ScriptApi;