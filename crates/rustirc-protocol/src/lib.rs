//! IRC Protocol implementation
//!
//! This crate provides a complete implementation of the IRC protocol
//! including RFC 1459, RFC 2812, and IRCv3 extensions.

pub mod command;
pub mod message;
pub mod parser;
pub mod builder;
pub mod caps;
pub mod numeric;

pub use command::Command;
pub use message::{Message, Prefix, Tag};
pub use parser::Parser;
pub use builder::MessageBuilder;
pub use caps::{Capability, CapabilitySet};
pub use numeric::Numeric;

/// IRC protocol version
pub const PROTOCOL_VERSION: &str = "RustIRC-0.1.0";

/// Maximum message length per IRC specification
pub const MAX_MESSAGE_LENGTH: usize = 512;