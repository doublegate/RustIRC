//! Message builder

use crate::{Message, Prefix, Tag};

pub struct MessageBuilder {
    message: Message,
}

impl MessageBuilder {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            message: Message::new(command),
        }
    }

    pub fn prefix(mut self, prefix: Prefix) -> Self {
        self.message.prefix = Some(prefix);
        self
    }

    pub fn server_prefix(self, server: impl Into<String>) -> Self {
        self.prefix(Prefix::Server(server.into()))
    }

    pub fn user_prefix(self, nick: impl Into<String>) -> Self {
        self.prefix(Prefix::User {
            nick: nick.into(),
            user: None,
            host: None,
        })
    }

    pub fn full_user_prefix(
        self,
        nick: impl Into<String>,
        user: impl Into<String>,
        host: impl Into<String>,
    ) -> Self {
        self.prefix(Prefix::User {
            nick: nick.into(),
            user: Some(user.into()),
            host: Some(host.into()),
        })
    }

    pub fn param(mut self, param: impl Into<String>) -> Self {
        self.message.params.push(param.into());
        self
    }

    pub fn params(mut self, params: Vec<String>) -> Self {
        self.message.params.extend(params);
        self
    }

    pub fn trailing(mut self, trailing: impl Into<String>) -> Self {
        self.message.params.push(trailing.into());
        self
    }

    pub fn tag(mut self, key: impl Into<String>, value: Option<String>) -> Self {
        let tag = Tag {
            key: key.into(),
            value,
        };

        if let Some(ref mut tags) = self.message.tags {
            tags.push(tag);
        } else {
            self.message.tags = Some(vec![tag]);
        }

        self
    }

    pub fn build(self) -> Message {
        self.message
    }
}

impl From<MessageBuilder> for Message {
    fn from(builder: MessageBuilder) -> Self {
        builder.build()
    }
}
