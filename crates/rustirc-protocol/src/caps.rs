//! IRCv3 Capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // Core capabilities
    AccountNotify,
    AccountTag,
    AwayNotify,
    Batch,
    CapNotify,
    ChgHost,
    EchoMessage,
    ExtendedJoin,
    InviteNotify,
    LabeledResponse,
    MessageTags,
    Monitor,
    MultiPrefix,
    Sasl,
    ServerTime,
    SetName,
    UserhostInNames,

    // Draft capabilities
    Multiline,
    NoImplicitNames,
    StandardReplies,

    // Custom capability
    Custom(String),
}

impl Capability {
    pub fn as_str(&self) -> &str {
        match self {
            Capability::AccountNotify => "account-notify",
            Capability::AccountTag => "account-tag",
            Capability::AwayNotify => "away-notify",
            Capability::Batch => "batch",
            Capability::CapNotify => "cap-notify",
            Capability::ChgHost => "chghost",
            Capability::EchoMessage => "echo-message",
            Capability::ExtendedJoin => "extended-join",
            Capability::InviteNotify => "invite-notify",
            Capability::LabeledResponse => "labeled-response",
            Capability::MessageTags => "message-tags",
            Capability::Monitor => "monitor",
            Capability::MultiPrefix => "multi-prefix",
            Capability::Sasl => "sasl",
            Capability::ServerTime => "server-time",
            Capability::SetName => "setname",
            Capability::UserhostInNames => "userhost-in-names",
            Capability::Multiline => "draft/multiline",
            Capability::NoImplicitNames => "draft/no-implicit-names",
            Capability::StandardReplies => "draft/standard-replies",
            Capability::Custom(s) => s,
        }
    }
}

impl FromStr for Capability {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "account-notify" => Capability::AccountNotify,
            "account-tag" => Capability::AccountTag,
            "away-notify" => Capability::AwayNotify,
            "batch" => Capability::Batch,
            "cap-notify" => Capability::CapNotify,
            "chghost" => Capability::ChgHost,
            "echo-message" => Capability::EchoMessage,
            "extended-join" => Capability::ExtendedJoin,
            "invite-notify" => Capability::InviteNotify,
            "labeled-response" => Capability::LabeledResponse,
            "message-tags" => Capability::MessageTags,
            "monitor" => Capability::Monitor,
            "multi-prefix" => Capability::MultiPrefix,
            "sasl" => Capability::Sasl,
            "server-time" => Capability::ServerTime,
            "setname" => Capability::SetName,
            "userhost-in-names" => Capability::UserhostInNames,
            "draft/multiline" => Capability::Multiline,
            "draft/no-implicit-names" => Capability::NoImplicitNames,
            "draft/standard-replies" => Capability::StandardReplies,
            other => Capability::Custom(other.to_string()),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, cap: Capability) {
        self.capabilities.insert(cap);
    }

    pub fn remove(&mut self, cap: &Capability) -> bool {
        self.capabilities.remove(cap)
    }

    pub fn contains(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }

    pub fn to_string_list(&self) -> Vec<String> {
        self.capabilities
            .iter()
            .map(|cap| cap.as_str().to_string())
            .collect()
    }
}
