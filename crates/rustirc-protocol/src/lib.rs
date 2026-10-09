//! IRC Protocol implementation
//!
//! This crate provides a complete implementation of the IRC protocol
//! including RFC 1459, RFC 2812, and IRCv3 extensions.

pub mod builder;
pub mod caps;
pub mod command;
pub mod ctcp;
pub mod message;
pub mod numeric;
pub mod parser;
pub mod validation;

pub use builder::MessageBuilder;
pub use caps::{Capability, CapabilitySet};
pub use command::Command;
pub use ctcp::{escape_ctcp, unescape_ctcp, CtcpHandler, CtcpMessage};
pub use message::{
    escape_tag_value, unescape_tag_value, Message, MessageRef, Prefix, PrefixRef, Tag, TagRef,
};
pub use numeric::Numeric;
pub use parser::Parser;
pub use validation::{
    is_valid_channel, is_valid_command, is_valid_nickname, IrcValidator, ValidationError,
};

/// IRC protocol version
pub const PROTOCOL_VERSION: &str = "RustIRC-0.1.0";

/// Maximum message length per IRC specification (excluding message tags)
pub const MAX_MESSAGE_LENGTH: usize = 512;

/// Maximum tags section length per IRCv3 Message Tags specification (4094 bytes tag data + space)
pub const MAX_TAGS_LENGTH: usize = 4094;

/// Maximum total length for an IRCv3 message with tags
pub const MAX_TOTAL_MESSAGE_LENGTH: usize = MAX_MESSAGE_LENGTH + MAX_TAGS_LENGTH + 2; // 4608
