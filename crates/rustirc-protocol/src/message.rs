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
