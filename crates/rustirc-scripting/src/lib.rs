//! Scripting engine for RustIRC
//!
//! This crate provides comprehensive scripting support for RustIRC through
//! Lua scripts. Features include:
//!
//! - Secure sandboxed Lua execution environment
//! - Comprehensive IRC API with 50+ functions
//! - Event-driven script system
//! - Custom command registration
//! - Resource management and limits
//!
//! # Examples
//!
//! ```rust
//! use rustirc_scripting::engine::ScriptEngine;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a script engine
//! let engine = ScriptEngine::new()?;
//!
//! // Load a simple script
//! engine.load_script("hello", r#"
//!     irc.print("Hello from Lua!")
//! "#).await?;
//! # Ok(())
//! # }
//! ```

pub mod api;
pub mod engine;
pub mod sandbox;
pub mod lua_api;

pub use api::ScriptApi;
pub use engine::{ScriptEngine, LoadedScript};
pub use lua_api::LuaIrcApi;
