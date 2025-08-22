//! Configuration management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user: UserConfig,
    pub servers: Vec<ServerConfig>,
    pub ui: UiConfig,
    pub logging: LoggingConfig,
    pub scripting: ScriptingConfig,
    pub custom_settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub nickname: String,
    pub alternative_nicknames: Vec<String>,
    pub username: String,
    pub realname: String,
    pub quit_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub use_tls: bool,
    pub password: Option<String>,
    pub auto_connect: bool,
    pub channels: Vec<ChannelConfig>,
    pub sasl: Option<SaslConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub name: String,
    pub key: Option<String>,
    pub auto_join: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaslConfig {
    pub mechanism: SaslMechanism,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SaslMechanism {
    Plain,
    External,
    ScramSha256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub timestamp_format: String,
    pub show_join_part: bool,
    pub buffer_size: usize,
    pub nicklist_width: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub enable: bool,
    pub path: String,
    pub format: LogFormat,
    pub rotation: LogRotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Plain,
    Json,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogRotation {
    Daily,
    Weekly,
    Monthly,
    Size(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptingConfig {
    pub enable: bool,
    pub scripts_path: String,
    pub auto_load: Vec<String>,
    pub sandbox_memory_limit: usize,
    pub sandbox_timeout_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user: UserConfig {
                nickname: "RustIRC".to_string(),
                alternative_nicknames: vec!["RustIRC_".to_string()],
                username: "rustirc".to_string(),
                realname: "RustIRC User".to_string(),
                quit_message: "RustIRC - https://github.com/doublegate/RustIRC".to_string(),
            },
            servers: vec![],
            ui: UiConfig {
                theme: "dark".to_string(),
                timestamp_format: "%H:%M:%S".to_string(),
                show_join_part: true,
                buffer_size: 10000,
                nicklist_width: 20,
            },
            logging: LoggingConfig {
                enable: true,
                path: "logs".to_string(),
                format: LogFormat::Plain,
                rotation: LogRotation::Daily,
            },
            scripting: ScriptingConfig {
                enable: true,
                scripts_path: "scripts".to_string(),
                auto_load: vec![],
                sandbox_memory_limit: 100 * 1024 * 1024, // 100MB
                sandbox_timeout_ms: 5000,
            },
            custom_settings: HashMap::new(),
        }
    }
}
