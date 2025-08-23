//! IRC message parser with comprehensive protocol compliance
//!
//! This module provides a robust parser for IRC messages that supports both
//! traditional IRC (RFC 1459/2812) and modern IRCv3 extensions including message tags.
//! The parser includes validation, error handling, and proper handling of edge cases.
//!
//! # Examples
//!
//! ```rust
//! use rustirc_protocol::parser::Parser;
//!
//! // Parse a simple command
//! let message = Parser::parse_message("PING :irc.example.com").unwrap();
//! assert_eq!(message.command, "PING");
//! assert_eq!(message.params, vec!["irc.example.com"]);
//!
//! // Parse a message with prefix
//! let message = Parser::parse_message(
//!     ":nick!user@host PRIVMSG #channel :Hello, world!"
//! ).unwrap();
//! assert_eq!(message.command, "PRIVMSG");
//! assert_eq!(message.params, vec!["#channel", "Hello, world!"]);
//!
//! // Parse IRCv3 message with tags
//! let message = Parser::parse_message(
//!     "@time=2021-01-01T00:00:00.000Z :server NOTICE #channel :Test"
//! ).unwrap();
//! assert!(message.tags.is_some());
//! ```

use crate::{IrcValidator, Message, Prefix, Tag, ValidationError};
use thiserror::Error;

/// Errors that can occur during IRC message parsing
///
/// These errors represent various failure modes when parsing IRC protocol messages,
/// from basic format issues to protocol compliance violations.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Empty message")]
    EmptyMessage,

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Message too long: {0} bytes (max: 512)")]
    MessageTooLong(usize),

    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
}

/// IRC message parser with validation support
///
/// The Parser handles the complete IRC message parsing process including:
/// - IRCv3 message tags parsing
/// - Prefix extraction (server or user)
/// - Command parsing (numeric or alphabetic)
/// - Parameter parsing (regular and trailing)
/// - Protocol validation and security checks
///
/// # Examples
///
/// ```rust
/// use rustirc_protocol::{parser::Parser, validation::IrcValidator};
///
/// // Create parser with default validator
/// let parser = Parser::new();
///
/// // Create parser with custom validator
/// let validator = IrcValidator::strict();
/// let parser = Parser::with_validator(validator);
///
/// // Parse a message
/// let message = parser.parse("JOIN #channel").unwrap();
/// assert_eq!(message.command, "JOIN");
/// ```
pub struct Parser {
    validator: IrcValidator,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    /// Create a new parser with default validation settings
    ///
    /// Uses a standard `IrcValidator` that balances compatibility with security.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_protocol::parser::Parser;
    ///
    /// let parser = Parser::new();
    /// let message = parser.parse("PING :server").unwrap();
    /// ```
    pub fn new() -> Self {
        Self {
            validator: IrcValidator::new(),
        }
    }

    /// Create a parser with a custom validator
    ///
    /// Allows fine-tuning of validation rules for specific use cases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_protocol::{parser::Parser, validation::IrcValidator};
    ///
    /// let strict_validator = IrcValidator::strict();
    /// let parser = Parser::with_validator(strict_validator);
    /// ```
    pub fn with_validator(validator: IrcValidator) -> Self {
        Self { validator }
    }

    /// Static convenience method for parsing messages
    ///
    /// Creates a temporary parser with default settings and parses the input.
    /// For repeated parsing, create a `Parser` instance to avoid overhead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_protocol::parser::Parser;
    ///
    /// let message = Parser::parse_message("QUIT :Goodbye").unwrap();
    /// assert_eq!(message.command, "QUIT");
    /// assert_eq!(message.params, vec!["Goodbye"]);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ParseError` if the message is invalid or doesn't conform to IRC protocol.
    pub fn parse_message(input: &str) -> Result<Message, ParseError> {
        Self::new().parse(input)
    }

    /// Parse an IRC message string into structured components
    ///
    /// This is the main parsing method that handles the complete IRC message format:
    /// `[@tags] [:prefix] command [params...] [:trailing]`
    ///
    /// # Arguments
    ///
    /// * `input` - Raw IRC message string (typically without trailing CRLF)
    ///
    /// # Returns
    ///
    /// Returns a `Message` struct containing parsed components, or `ParseError` on failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_protocol::parser::Parser;
    ///
    /// let parser = Parser::new();
    ///
    /// // Basic command
    /// let msg = parser.parse("PING :irc.example.com").unwrap();
    /// assert_eq!(msg.command, "PING");
    ///
    /// // With prefix and multiple parameters
    /// let msg = parser.parse(":nick!user@host PRIVMSG #chan :Hello").unwrap();
    /// assert_eq!(msg.command, "PRIVMSG");
    /// assert_eq!(msg.params.len(), 2);
    ///
    /// // IRCv3 with tags
    /// let msg = parser.parse("@time=2021-01-01T00:00:00Z PING :server").unwrap();
    /// assert!(msg.tags.is_some());
    /// ```
    ///
    /// # Errors
    ///
    /// - `EmptyMessage` - Input string is empty
    /// - `MessageTooLong` - Message exceeds 512 byte limit
    /// - `InvalidFormat` - Malformed message structure
    /// - `ValidationError` - Protocol compliance violation
    pub fn parse(&self, input: &str) -> Result<Message, ParseError> {
        if input.is_empty() {
            return Err(ParseError::EmptyMessage);
        }

        // Validate message length first
        self.validator.validate_message_length(input)?;

        if input.len() > crate::MAX_MESSAGE_LENGTH {
            return Err(ParseError::MessageTooLong(input.len()));
        }

        let mut chars = input.chars().peekable();
        let mut tags = None;
        let mut prefix = None;
        let mut command = String::new();
        let mut params = Vec::new();

        // Parse tags (IRCv3)
        if chars.peek() == Some(&'@') {
            chars.next(); // Skip '@'
            tags = Some(self.parse_tags(&mut chars)?);
            Self::skip_whitespace(&mut chars);
        }

        // Parse prefix
        if chars.peek() == Some(&':') {
            chars.next(); // Skip ':'
            prefix = Some(Self::parse_prefix(&mut chars)?);
            Self::skip_whitespace(&mut chars);
        }

        // Parse command
        while let Some(ch) = chars.peek() {
            if ch.is_whitespace() {
                break;
            }
            if let Some(c) = chars.next() {
                command.push(c);
            } else {
                return Err(ParseError::InvalidFormat(
                    "Unexpected end of input while parsing command".to_string(),
                ));
            }
        }

        if command.is_empty() {
            return Err(ParseError::InvalidFormat("Missing command".to_string()));
        }

        // Validate command
        self.validator.validate_command(&command)?;

        Self::skip_whitespace(&mut chars);

        // Parse parameters
        while chars.peek().is_some() {
            if chars.peek() == Some(&':') {
                // Trailing parameter
                chars.next(); // Skip ':'
                params.push(chars.collect());
                break;
            } else {
                // Regular parameter
                let mut param = String::new();
                while let Some(ch) = chars.peek() {
                    if ch.is_whitespace() {
                        break;
                    }
                    if let Some(c) = chars.next() {
                        param.push(c);
                    } else {
                        return Err(ParseError::InvalidFormat(
                            "Unexpected end of input while parsing parameter".to_string(),
                        ));
                    }
                }
                if !param.is_empty() {
                    params.push(param);
                }
                Self::skip_whitespace(&mut chars);
            }
        }

        // Validate parameters
        for param in &params {
            self.validator.validate_parameter(param)?;
        }

        Ok(Message {
            tags,
            prefix,
            command,
            params,
        })
    }

    /// Parse IRCv3 message tags from the character stream
    ///
    /// Handles the `@key=value;key2=value2` format at the start of IRCv3 messages.
    /// Tags are separated by semicolons and may have optional values.
    fn parse_tags(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<Vec<Tag>, ParseError> {
        let mut tags = Vec::new();
        let mut current_tag = String::new();

        while let Some(ch) = chars.peek() {
            if ch.is_whitespace() {
                break;
            }
            if *ch == ';' {
                chars.next();
                if !current_tag.is_empty() {
                    tags.push(self.parse_single_tag(&current_tag)?);
                    current_tag.clear();
                }
            } else if let Some(c) = chars.next() {
                current_tag.push(c);
            } else {
                return Err(ParseError::InvalidFormat(
                    "Unexpected end of input while parsing tag".to_string(),
                ));
            }
        }

        if !current_tag.is_empty() {
            tags.push(self.parse_single_tag(&current_tag)?);
        }

        Ok(tags)
    }

    /// Parse a single tag from key=value or key format
    ///
    /// Validates both the key and optional value according to IRCv3 specifications.
    fn parse_single_tag(&self, tag_str: &str) -> Result<Tag, ParseError> {
        if let Some((key, value)) = tag_str.split_once('=') {
            // Validate tag key and value
            self.validator.validate_tag_key(key)?;
            self.validator.validate_tag_value(value)?;
            Ok(Tag::from_raw(key, Some(value)))
        } else {
            // Validate tag key (no value)
            self.validator.validate_tag_key(tag_str)?;
            Ok(Tag::from_raw(tag_str, None::<String>))
        }
    }

    /// Parse IRC message prefix (server name or user!nick@host)
    ///
    /// Determines whether the prefix is a server name or user information
    /// and extracts the appropriate components.
    fn parse_prefix(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<Prefix, ParseError> {
        let mut prefix_str = String::new();

        while let Some(ch) = chars.peek() {
            if ch.is_whitespace() {
                break;
            }
            if let Some(c) = chars.next() {
                prefix_str.push(c);
            } else {
                return Err(ParseError::InvalidFormat(
                    "Unexpected end of input while parsing prefix".to_string(),
                ));
            }
        }

        if prefix_str.is_empty() {
            return Err(ParseError::InvalidFormat("Empty prefix".to_string()));
        }

        // Check if it's a user prefix (contains ! or @)
        if prefix_str.contains('!') || prefix_str.contains('@') {
            let mut parts = prefix_str.splitn(2, '!');
            let nick = parts
                .next()
                .ok_or_else(|| ParseError::InvalidFormat("Empty nick in prefix".to_string()))?
                .to_string();

            if let Some(rest) = parts.next() {
                let mut parts = rest.splitn(2, '@');
                let user = Some(
                    parts
                        .next()
                        .ok_or_else(|| {
                            ParseError::InvalidFormat("Empty user in prefix".to_string())
                        })?
                        .to_string(),
                );
                let host = parts.next().map(String::from);

                Ok(Prefix::User { nick, user, host })
            } else if let Some((nick, host)) = prefix_str.split_once('@') {
                Ok(Prefix::User {
                    nick: nick.to_string(),
                    user: None,
                    host: Some(host.to_string()),
                })
            } else {
                Ok(Prefix::User {
                    nick,
                    user: None,
                    host: None,
                })
            }
        } else {
            Ok(Prefix::Server(prefix_str))
        }
    }

    /// Skip whitespace characters in the input stream
    ///
    /// IRC messages use spaces to separate components, this helper advances
    /// past any whitespace to the next significant character.
    fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
        while chars.peek().is_some_and(|ch| ch.is_whitespace()) {
            chars.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let msg = Parser::parse_message("PING :server").unwrap();
        assert_eq!(msg.command, "PING");
        assert_eq!(msg.params, vec!["server"]);
    }

    #[test]
    fn test_parse_with_prefix() {
        let msg = Parser::parse_message(":nick!user@host PRIVMSG #channel :Hello, world!").unwrap();
        assert!(matches!(msg.prefix, Some(Prefix::User { .. })));
        assert_eq!(msg.command, "PRIVMSG");
        assert_eq!(msg.params, vec!["#channel", "Hello, world!"]);
    }

    #[test]
    fn test_parse_with_tags() {
        let msg =
            Parser::parse_message("@time=2021-01-01T00:00:00.000Z :server NOTICE #channel :Test")
                .unwrap();
        assert!(msg.tags.is_some());
        assert_eq!(msg.tags.unwrap()[0].key, "time");
    }
}
