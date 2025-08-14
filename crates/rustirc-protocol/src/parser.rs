//! IRC message parser

use crate::{Message, Prefix, Tag};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Empty message")]
    EmptyMessage,
    
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("Message too long: {0} bytes (max: 512)")]
    MessageTooLong(usize),
}

pub struct Parser;

impl Parser {
    pub fn parse(input: &str) -> Result<Message, ParseError> {
        if input.is_empty() {
            return Err(ParseError::EmptyMessage);
        }

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
            tags = Some(Self::parse_tags(&mut chars)?);
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
            command.push(chars.next().unwrap());
        }

        if command.is_empty() {
            return Err(ParseError::InvalidFormat("Missing command".to_string()));
        }

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
                    param.push(chars.next().unwrap());
                }
                if !param.is_empty() {
                    params.push(param);
                }
                Self::skip_whitespace(&mut chars);
            }
        }

        Ok(Message {
            tags,
            prefix,
            command,
            params,
        })
    }

    fn parse_tags(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Vec<Tag>, ParseError> {
        let mut tags = Vec::new();
        let mut current_tag = String::new();

        while let Some(ch) = chars.peek() {
            if ch.is_whitespace() {
                break;
            }
            if *ch == ';' {
                chars.next();
                if !current_tag.is_empty() {
                    tags.push(Self::parse_single_tag(&current_tag)?);
                    current_tag.clear();
                }
            } else {
                current_tag.push(chars.next().unwrap());
            }
        }

        if !current_tag.is_empty() {
            tags.push(Self::parse_single_tag(&current_tag)?);
        }

        Ok(tags)
    }

    fn parse_single_tag(tag_str: &str) -> Result<Tag, ParseError> {
        if let Some((key, value)) = tag_str.split_once('=') {
            Ok(Tag {
                key: key.to_string(),
                value: Some(value.to_string()),
            })
        } else {
            Ok(Tag {
                key: tag_str.to_string(),
                value: None,
            })
        }
    }

    fn parse_prefix(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Prefix, ParseError> {
        let mut prefix_str = String::new();
        
        while let Some(ch) = chars.peek() {
            if ch.is_whitespace() {
                break;
            }
            prefix_str.push(chars.next().unwrap());
        }

        if prefix_str.is_empty() {
            return Err(ParseError::InvalidFormat("Empty prefix".to_string()));
        }

        // Check if it's a user prefix (contains ! or @)
        if prefix_str.contains('!') || prefix_str.contains('@') {
            let mut parts = prefix_str.splitn(2, '!');
            let nick = parts.next().unwrap().to_string();
            
            if let Some(rest) = parts.next() {
                let mut parts = rest.splitn(2, '@');
                let user = Some(parts.next().unwrap().to_string());
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
        while chars.peek().map_or(false, |ch| ch.is_whitespace()) {
            chars.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_command() {
        let msg = Parser::parse("PING :server").unwrap();
        assert_eq!(msg.command, "PING");
        assert_eq!(msg.params, vec!["server"]);
    }

    #[test]
    fn test_parse_with_prefix() {
        let msg = Parser::parse(":nick!user@host PRIVMSG #channel :Hello, world!").unwrap();
        assert!(matches!(msg.prefix, Some(Prefix::User { .. })));
        assert_eq!(msg.command, "PRIVMSG");
        assert_eq!(msg.params, vec!["#channel", "Hello, world!"]);
    }

    #[test]
    fn test_parse_with_tags() {
        let msg = Parser::parse("@time=2021-01-01T00:00:00.000Z :server NOTICE #channel :Test").unwrap();
        assert!(msg.tags.is_some());
        assert_eq!(msg.tags.unwrap()[0].key, "time");
    }
}