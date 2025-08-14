//! State management

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientState {
    pub servers: HashMap<String, ServerState>,
    pub current_server: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub address: String,
    pub port: u16,
    pub connected: bool,
    pub nickname: String,
    pub channels: HashMap<String, ChannelState>,
    pub capabilities: Vec<String>,
    pub isupport: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    pub name: String,
    pub topic: Option<String>,
    pub users: Vec<User>,
    pub modes: String,
    pub joined: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub nickname: String,
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub realname: Option<String>,
    pub modes: String,
    pub away: bool,
}