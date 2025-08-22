//! IRC message parser

use crate::{IrcValidator, Message, Prefix, Tag, ValidationError};
use thiserror::Error;

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

pub struct Parser {
    validator: IrcValidator,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            validator: IrcValidator::new(),
        }
    }

    pub fn with_validator(validator: IrcValidator) -> Self {
        Self { validator }
    }

    /// Static convenience method for backward compatibility
    pub fn parse_message(input: &str) -> Result<Message, ParseError> {
        Self::new().parse(input)
    }

    /// Parse with validation (instance method)
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
