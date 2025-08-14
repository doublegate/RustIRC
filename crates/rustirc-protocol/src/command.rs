//! IRC Commands

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    // Connection commands
    Nick { nickname: String },
    User { username: String, mode: String, realname: String },
    Pass { password: String },
    Quit { message: Option<String> },
    
    // Channel commands
    Join { channels: Vec<String>, keys: Vec<String> },
    Part { channels: Vec<String>, message: Option<String> },
    Topic { channel: String, topic: Option<String> },
    Names { channels: Vec<String> },
    List { channels: Option<Vec<String>> },
    
    // Message commands
    PrivMsg { target: String, text: String },
    Notice { target: String, text: String },
    
    // User commands
    Who { mask: String },
    Whois { targets: Vec<String> },
    Whowas { nicknames: Vec<String>, count: Option<u32> },
    
    // Server commands
    Ping { server1: String, server2: Option<String> },
    Pong { server1: String, server2: Option<String> },
    
    // Capability negotiation
    Cap { subcommand: CapSubcommand },
    
    // SASL
    Authenticate { data: String },
    
    // Mode commands
    Mode { target: String, modes: Option<String>, params: Vec<String> },
    
    // Other
    Raw { command: String, params: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapSubcommand {
    Ls { version: Option<String> },
    List,
    Req { capabilities: Vec<String> },
    Ack { capabilities: Vec<String> },
    Nak { capabilities: Vec<String> },
    End,
}

impl Command {
    pub fn to_message(&self) -> crate::Message {
        match self {
            Command::Nick { nickname } => {
                crate::Message::new("NICK").add_param(nickname.clone())
            }
            Command::User { username, mode, realname } => {
                crate::Message::new("USER")
                    .add_param(username.clone())
                    .add_param(mode.clone())
                    .add_param("*")
                    .add_param(realname.clone())
            }
            Command::Join { channels, keys } => {
                let mut msg = crate::Message::new("JOIN")
                    .add_param(channels.join(","));
                if !keys.is_empty() {
                    msg = msg.add_param(keys.join(","));
                }
                msg
            }
            Command::PrivMsg { target, text } => {
                crate::Message::new("PRIVMSG")
                    .add_param(target.clone())
                    .add_param(text.clone())
            }
            Command::Ping { server1, server2 } => {
                let mut msg = crate::Message::new("PING").add_param(server1.clone());
                if let Some(server2) = server2 {
                    msg = msg.add_param(server2.clone());
                }
                msg
            }
            _ => {
                // Add more conversions as needed
                crate::Message::new("UNKNOWN")
            }
        }
    }
}