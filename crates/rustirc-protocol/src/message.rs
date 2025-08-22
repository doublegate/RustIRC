//! IRC message types

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub tags: Option<Vec<Tag>>,
    pub prefix: Option<Prefix>,
    pub command: String,
    pub params: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub key: String,
    pub value: Option<String>,
}

impl Tag {
    /// Create a new tag with escaped value
    pub fn new(key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        Self {
            key: key.into(),
            value: value.map(|v| escape_tag_value(&v.into())),
        }
    }

    /// Create a tag from raw (potentially escaped) key-value pair
    pub fn from_raw(key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        Self {
            key: key.into(),
            value: value.map(|v| v.into()),
        }
    }

    /// Get the unescaped value
    pub fn unescaped_value(&self) -> Option<String> {
        self.value.as_ref().map(|v| unescape_tag_value(v))
    }

    /// Get the raw (escaped) value
    pub fn raw_value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

/// Escape tag value according to IRCv3 spec
/// https://ircv3.net/specs/extensions/message-tags.html#escaping-values
pub fn escape_tag_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for ch in value.chars() {
        match ch {
            ';' => escaped.push_str("\\:"),
            ' ' => escaped.push_str("\\s"),
            '\\' => escaped.push_str("\\\\"),
            '\r' => escaped.push_str("\\r"),
            '\n' => escaped.push_str("\\n"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

/// Unescape tag value according to IRCv3 spec
/// https://ircv3.net/specs/extensions/message-tags.html#escaping-values
pub fn unescape_tag_value(value: &str) -> String {
    let mut unescaped = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some(':') => unescaped.push(';'),
                Some('s') => unescaped.push(' '),
                Some('\\') => unescaped.push('\\'),
                Some('r') => unescaped.push('\r'),
                Some('n') => unescaped.push('\n'),
                Some(other) => {
                    // Unknown escape sequence, preserve as-is
                    unescaped.push('\\');
                    unescaped.push(other);
                }
                None => {
                    // Trailing backslash, preserve as-is
                    unescaped.push('\\');
                }
            }
        } else {
            unescaped.push(ch);
        }
    }

    unescaped
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Prefix {
    Server(String),
    User {
        nick: String,
        user: Option<String>,
        host: Option<String>,
    },
}

impl Message {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            tags: None,
            prefix: None,
            command: command.into(),
            params: Vec::new(),
        }
    }

    pub fn with_prefix(mut self, prefix: Prefix) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn with_params(mut self, params: Vec<String>) -> Self {
        self.params = params;
        self
    }

    pub fn add_param(mut self, param: impl Into<String>) -> Self {
        self.params.push(param.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<Tag>) -> Self {
        self.tags = Some(tags);
        self
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Tags
        if let Some(tags) = &self.tags {
            write!(f, "@")?;
            for (i, tag) in tags.iter().enumerate() {
                if i > 0 {
                    write!(f, ";")?;
                }
                write!(f, "{}", tag.key)?;
                if let Some(value) = &tag.value {
                    write!(f, "={value}")?;
                }
            }
            write!(f, " ")?;
        }

        // Prefix
        if let Some(prefix) = &self.prefix {
            write!(f, ":{prefix} ")?;
        }

        // Command
        write!(f, "{}", self.command)?;

        // Parameters
        for (i, param) in self.params.iter().enumerate() {
            write!(f, " ")?;
            if i == self.params.len() - 1 && (param.contains(' ') || param.starts_with(':')) {
                write!(f, ":{param}")?;
            } else {
                write!(f, "{param}")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Prefix::Server(server) => write!(f, "{server}"),
            Prefix::User { nick, user, host } => {
                write!(f, "{nick}")?;
                if let Some(user) = user {
                    write!(f, "!{user}")?;
                }
                if let Some(host) = host {
                    write!(f, "@{host}")?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_escaping() {
        // Test basic escaping
        assert_eq!(escape_tag_value("hello;world"), "hello\\:world");
        assert_eq!(escape_tag_value("hello world"), "hello\\sworld");
        assert_eq!(escape_tag_value("hello\\world"), "hello\\\\world");
        assert_eq!(escape_tag_value("hello\rworld"), "hello\\rworld");
        assert_eq!(escape_tag_value("hello\nworld"), "hello\\nworld");

        // Test complex escaping
        assert_eq!(
            escape_tag_value("hello; world\r\n\\test"),
            "hello\\:\\sworld\\r\\n\\\\test"
        );
    }

    #[test]
    fn test_tag_unescaping() {
        // Test basic unescaping
        assert_eq!(unescape_tag_value("hello\\:world"), "hello;world");
        assert_eq!(unescape_tag_value("hello\\sworld"), "hello world");
        assert_eq!(unescape_tag_value("hello\\\\world"), "hello\\world");
        assert_eq!(unescape_tag_value("hello\\rworld"), "hello\rworld");
        assert_eq!(unescape_tag_value("hello\\nworld"), "hello\nworld");

        // Test complex unescaping
        assert_eq!(
            unescape_tag_value("hello\\:\\sworld\\r\\n\\\\test"),
            "hello; world\r\n\\test"
        );

        // Test unknown escape sequences (should be preserved)
        assert_eq!(unescape_tag_value("hello\\xworld"), "hello\\xworld");

        // Test trailing backslash
        assert_eq!(unescape_tag_value("hello\\"), "hello\\");
    }

    #[test]
    fn test_tag_round_trip() {
        let original = "hello; world\r\n\\test";
        let escaped = escape_tag_value(original);
        let unescaped = unescape_tag_value(&escaped);
        assert_eq!(original, unescaped);
    }

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new("key", Some("hello; world"));
        assert_eq!(tag.key, "key");
        assert_eq!(tag.raw_value(), Some("hello\\:\\sworld"));
        assert_eq!(tag.unescaped_value(), Some("hello; world".to_string()));
    }

    #[test]
    fn test_tag_from_raw() {
        let tag = Tag::from_raw("key", Some("hello\\:\\sworld"));
        assert_eq!(tag.key, "key");
        assert_eq!(tag.raw_value(), Some("hello\\:\\sworld"));
        assert_eq!(tag.unescaped_value(), Some("hello; world".to_string()));
    }
}
