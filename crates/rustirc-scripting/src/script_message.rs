//! ScriptMessage - IRC message type exposed to Lua scripts

use mlua::{UserData, UserDataMethods};

/// IRC message representation for Lua scripts
#[derive(Debug, Clone)]
pub struct ScriptMessage {
    pub prefix: Option<String>,
    pub command: String,
    pub params: Vec<String>,
}

impl ScriptMessage {
    pub fn new(prefix: Option<String>, command: String, params: Vec<String>) -> Self {
        Self {
            prefix,
            command,
            params,
        }
    }

    /// Create from a protocol Message
    pub fn from_protocol_message(msg: &rustirc_protocol::Message) -> Self {
        let prefix = msg.prefix.as_ref().map(|p| p.to_string());
        Self {
            prefix,
            command: msg.command.clone(),
            params: msg.params.clone(),
        }
    }
}

impl UserData for ScriptMessage {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_prefix", |_, this, ()| Ok(this.prefix.clone()));
        methods.add_method("get_command", |_, this, ()| Ok(this.command.clone()));
        methods.add_method("get_params", |_, this, ()| Ok(this.params.clone()));

        methods.add_method("get_channel", |_, this, ()| {
            Ok(this.params.first().cloned())
        });

        methods.add_method("get_text", |_, this, ()| Ok(this.params.last().cloned()));

        methods.add_method("get_nick", |_, this, ()| {
            Ok(this
                .prefix
                .as_ref()
                .and_then(|p| p.split('!').next().map(String::from)))
        });
    }
}
