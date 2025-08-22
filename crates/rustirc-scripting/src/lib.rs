//! Scripting engine for RustIRC

pub mod api;
pub mod engine;
pub mod sandbox;

pub use api::ScriptApi;
pub use engine::ScriptEngine;
