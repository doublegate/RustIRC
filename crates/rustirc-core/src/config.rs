//! Configuration management for RustIRC
//!
//! This module provides configuration structures for all aspects of the IRC client,
//! including user settings, server connections, UI preferences, and scripting options.
//!
//! # Examples
//!
//! ```
//! use rustirc_core::config::{Config, UserConfig};
//!
//! let config = Config::default();
//! assert_eq!(config.user.nickname, "RustIRC");
//!
//! // Or create a custom configuration
//! let mut custom_config = Config::default();
//! custom_config.user.nickname = "MyBot".to_string();
//! assert_eq!(custom_config.user.nickname, "MyBot");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main configuration structure for the IRC client
///
/// # Examples
///
/// ```
/// use rustirc_core::config::Config;
///
/// let config = Config::default();
/// assert_eq!(config.user.nickname, "RustIRC");
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub user: UserConfig,
    pub servers: Vec<ServerConfig>,
    pub ui: UiConfig,
    pub logging: LoggingConfig,
    pub scripting: ScriptingConfig,
    pub dcc: DccConfig,
    pub flood: FloodConfig,
    pub proxy: Option<ProxyConfig>,
    pub notifications: NotificationConfig,
    pub custom_settings: HashMap<String, String>,
}

/// User configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub nickname: String,
    pub alternative_nicknames: Vec<String>,
    pub username: String,
    pub realname: String,
    pub quit_message: String,
}

/// IRC server connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub use_tls: bool,
    pub password: Option<String>,
    pub auto_connect: bool,
    pub channels: Vec<ChannelConfig>,
    pub sasl: Option<SaslConfig>,
    pub proxy: Option<ProxyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
#[serde(default)]
pub struct UiConfig {
    pub theme: String,
    pub timestamp_format: String,
    pub show_join_part: bool,
    pub buffer_size: usize,
    pub nicklist_width: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
#[serde(default)]
pub struct ScriptingConfig {
    pub enable: bool,
    pub scripts_path: String,
    pub auto_load: Vec<String>,
    pub sandbox_memory_limit: usize,
    pub sandbox_timeout_ms: u64,
}

/// DCC (Direct Client-to-Client) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DccConfig {
    pub enabled: bool,
    pub download_dir: String,
    pub auto_accept: bool,
    pub max_file_size: u64,
    pub port_range_start: u16,
    pub port_range_end: u16,
    pub auto_resume: bool,
}

/// Flood protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FloodConfig {
    pub enabled: bool,
    pub messages_per_second: f64,
    pub burst_limit: usize,
    pub queue_size: usize,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub address: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    Socks5,
    HttpConnect,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub highlight_words: Vec<String>,
    pub nick_mentions: bool,
    pub private_messages: bool,
    pub sound: bool,
    pub quiet_hours: Option<QuietHours>,
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub enabled: bool,
    pub start_hour: u8,
    pub end_hour: u8,
    pub weekends: bool,
}

// === Default implementations ===

// Config derives Default since all fields implement Default

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            nickname: "RustIRC".to_string(),
            alternative_nicknames: vec!["RustIRC_".to_string()],
            username: "rustirc".to_string(),
            realname: "RustIRC User".to_string(),
            quit_message: "RustIRC - https://github.com/doublegate/RustIRC".to_string(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            address: String::new(),
            port: 6697,
            use_tls: true,
            password: None,
            auto_connect: false,
            channels: vec![],
            sasl: None,
            proxy: None,
        }
    }
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            key: None,
            auto_join: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            timestamp_format: "%H:%M:%S".to_string(),
            show_join_part: true,
            buffer_size: 10000,
            nicklist_width: 20,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enable: true,
            path: "logs".to_string(),
            format: LogFormat::Plain,
            rotation: LogRotation::Daily,
        }
    }
}

impl Default for ScriptingConfig {
    fn default() -> Self {
        Self {
            enable: true,
            scripts_path: "scripts".to_string(),
            auto_load: vec![],
            sandbox_memory_limit: 100 * 1024 * 1024, // 100MB
            sandbox_timeout_ms: 5000,
        }
    }
}

impl Default for DccConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            download_dir: "downloads".to_string(),
            auto_accept: false,
            max_file_size: 100 * 1024 * 1024, // 100MB
            port_range_start: 1024,
            port_range_end: 65535,
            auto_resume: true,
        }
    }
}

impl Default for FloodConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            messages_per_second: 2.0,
            burst_limit: 5,
            queue_size: 100,
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            highlight_words: vec![],
            nick_mentions: true,
            private_messages: true,
            sound: true,
            quiet_hours: None,
        }
    }
}

// === Config I/O methods ===

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file with pretty formatting
    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the default configuration file path
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustirc")
            .join("config.toml")
    }

    /// Load config from default path, falling back to defaults
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            match Self::from_file(&path) {
                Ok(config) => {
                    tracing::info!("Loaded configuration from {}", path.display());
                    config
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load config from {}: {}, using defaults",
                        path.display(),
                        e
                    );
                    Self::default()
                }
            }
        } else {
            tracing::info!("No config file found at {}, using defaults", path.display());
            Self::default()
        }
    }

    /// Generate a default config file with comments for first-run experience
    pub fn generate_default_config(path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut config = Config::default();
        // Add example Libera Chat server
        config.servers.push(ServerConfig {
            name: "Libera Chat".to_string(),
            address: "irc.libera.chat".to_string(),
            port: 6697,
            use_tls: true,
            auto_connect: false,
            channels: vec![ChannelConfig {
                name: "#rustirc".to_string(),
                key: None,
                auto_join: true,
            }],
            ..Default::default()
        });

        let content = toml::to_string_pretty(&config)?;
        let commented = format!(
            "# RustIRC Configuration File\n\
             # https://github.com/doublegate/RustIRC\n\
             #\n\
             # Edit this file to customize your RustIRC experience.\n\
             # All settings have sensible defaults.\n\n{content}"
        );
        std::fs::write(path, commented)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_config_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.user.nickname, config.user.nickname);
        assert_eq!(loaded.ui.theme, config.ui.theme);
    }

    #[test]
    fn test_from_file_valid() {
        let dir = std::env::temp_dir().join("rustirc_test_from_file");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        let config = Config::default();
        config.save(&path).unwrap();

        let loaded = Config::from_file(&path).unwrap();
        assert_eq!(loaded.user.nickname, "RustIRC");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_from_file_invalid() {
        let dir = std::env::temp_dir().join("rustirc_test_invalid");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"this is not valid { toml [").unwrap();

        assert!(Config::from_file(&path).is_err());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_save_and_load() {
        let dir = std::env::temp_dir().join("rustirc_test_save_load");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        let mut config = Config::default();
        config.user.nickname = "TestUser".to_string();
        config.save(&path).unwrap();

        let loaded = Config::from_file(&path).unwrap();
        assert_eq!(loaded.user.nickname, "TestUser");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_default_path() {
        let path = Config::default_path();
        assert!(path.to_string_lossy().contains("rustirc"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_config_with_servers() {
        let mut config = Config::default();
        config.servers.push(ServerConfig {
            name: "Test".to_string(),
            address: "irc.test.com".to_string(),
            port: 6697,
            use_tls: true,
            ..Default::default()
        });

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.servers.len(), 1);
        assert_eq!(loaded.servers[0].address, "irc.test.com");
    }

    #[test]
    fn test_serde_default_forward_compat() {
        // Verify that missing fields use defaults (forward compatibility)
        let minimal = r#"
[user]
nickname = "TestNick"
"#;
        let config: Config = toml::from_str(minimal).unwrap();
        assert_eq!(config.user.nickname, "TestNick");
        assert_eq!(config.ui.theme, "dark"); // default
        assert!(config.flood.enabled); // default
    }
}
