//! IRC Commands

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    // Connection commands
    Nick {
        nickname: String,
    },
    User {
        username: String,
        mode: String,
        realname: String,
    },
    Pass {
        password: String,
    },
    Quit {
        message: Option<String>,
    },

    // Channel commands
    Join {
        channels: Vec<String>,
        keys: Vec<String>,
    },
    Part {
        channels: Vec<String>,
        message: Option<String>,
    },
    Topic {
        channel: String,
        topic: Option<String>,
    },
    Names {
        channels: Vec<String>,
    },
    List {
        channels: Option<Vec<String>>,
    },

    // Message commands
    PrivMsg {
        target: String,
        text: String,
    },
    Notice {
        target: String,
        text: String,
    },

    // User commands
    Who {
        mask: String,
    },
    Whois {
        targets: Vec<String>,
    },
    Whowas {
        nicknames: Vec<String>,
        count: Option<u32>,
    },

    // Channel management
    Kick {
        channel: String,
        nick: String,
        comment: Option<String>,
    },

    // Server commands
    Ping {
        server1: String,
        server2: Option<String>,
    },
    Pong {
        server1: String,
        server2: Option<String>,
    },

    // Capability negotiation
    Cap {
        subcommand: CapSubcommand,
    },

    // SASL
    Authenticate {
        data: String,
    },

    // Mode commands
    Mode {
        target: String,
        modes: Option<String>,
        params: Vec<String>,
    },

    // Other
    Raw {
        command: String,
        params: Vec<String>,
    },
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
            Command::Nick { nickname } => crate::Message::new("NICK").add_param(nickname.clone()),
            Command::User {
                username,
                mode,
                realname,
            } => crate::Message::new("USER")
                .add_param(username.clone())
                .add_param(mode.clone())
                .add_param("*")
                .add_param(realname.clone()),
            Command::Join { channels, keys } => {
                let mut msg = crate::Message::new("JOIN").add_param(channels.join(","));
                if !keys.is_empty() {
                    msg = msg.add_param(keys.join(","));
                }
                msg
            }
            Command::PrivMsg { target, text } => crate::Message::new("PRIVMSG")
                .add_param(target.clone())
                .add_param(text.clone()),
            Command::Ping { server1, server2 } => {
                let mut msg = crate::Message::new("PING").add_param(server1.clone());
                if let Some(server2) = server2 {
                    msg = msg.add_param(server2.clone());
                }
                msg
            }
            Command::Pass { password } => crate::Message::new("PASS").add_param(password.clone()),
            Command::Quit { message } => {
                let mut msg = crate::Message::new("QUIT");
                if let Some(message) = message {
                    msg = msg.add_param(message.clone());
                }
                msg
            }
            Command::Part { channels, message } => {
                let mut msg = crate::Message::new("PART").add_param(channels.join(","));
                if let Some(message) = message {
                    msg = msg.add_param(message.clone());
                }
                msg
            }
            Command::Topic { channel, topic } => {
                let mut msg = crate::Message::new("TOPIC").add_param(channel.clone());
                if let Some(topic) = topic {
                    msg = msg.add_param(topic.clone());
                }
                msg
            }
            Command::Names { channels } => {
                crate::Message::new("NAMES").add_param(channels.join(","))
            }
            Command::List { channels } => {
                let mut msg = crate::Message::new("LIST");
                if let Some(channels) = channels {
                    msg = msg.add_param(channels.join(","));
                }
                msg
            }
            Command::Notice { target, text } => crate::Message::new("NOTICE")
                .add_param(target.clone())
                .add_param(text.clone()),
            Command::Who { mask } => crate::Message::new("WHO").add_param(mask.clone()),
            Command::Whois { targets } => crate::Message::new("WHOIS").add_param(targets.join(",")),
            Command::Whowas { nicknames, count } => {
                let mut msg = crate::Message::new("WHOWAS").add_param(nicknames.join(","));
                if let Some(count) = count {
                    msg = msg.add_param(count.to_string());
                }
                msg
            }
            Command::Kick {
                channel,
                nick,
                comment,
            } => {
                let mut msg = crate::Message::new("KICK")
                    .add_param(channel.clone())
                    .add_param(nick.clone());
                if let Some(comment) = comment {
                    msg = msg.add_param(comment.clone());
                }
                msg
            }
            Command::Pong { server1, server2 } => {
                let mut msg = crate::Message::new("PONG").add_param(server1.clone());
                if let Some(server2) = server2 {
                    msg = msg.add_param(server2.clone());
                }
                msg
            }
            Command::Cap { subcommand } => match subcommand {
                CapSubcommand::Ls { version } => {
                    let mut msg = crate::Message::new("CAP").add_param("LS");
                    if let Some(version) = version {
                        msg = msg.add_param(version.clone());
                    }
                    msg
                }
                CapSubcommand::List => crate::Message::new("CAP").add_param("LIST"),
                CapSubcommand::Req { capabilities } => crate::Message::new("CAP")
                    .add_param("REQ")
                    .add_param(capabilities.join(" ")),
                CapSubcommand::Ack { capabilities } => crate::Message::new("CAP")
                    .add_param("ACK")
                    .add_param(capabilities.join(" ")),
                CapSubcommand::Nak { capabilities } => crate::Message::new("CAP")
                    .add_param("NAK")
                    .add_param(capabilities.join(" ")),
                CapSubcommand::End => crate::Message::new("CAP").add_param("END"),
            },
            Command::Authenticate { data } => {
                crate::Message::new("AUTHENTICATE").add_param(data.clone())
            }
            Command::Mode {
                target,
                modes,
                params,
            } => {
                let mut msg = crate::Message::new("MODE").add_param(target.clone());
                if let Some(modes) = modes {
                    msg = msg.add_param(modes.clone());
                }
                for param in params {
                    msg = msg.add_param(param.clone());
                }
                msg
            }
            Command::Raw { command, params } => {
                let mut msg = crate::Message::new(command.clone());
                for param in params {
                    msg = msg.add_param(param.clone());
                }
                msg
            }
        }
    }
}
