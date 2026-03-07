//! Integration tests for configuration loading and persistence

use rustirc_core::config::{ChannelConfig, Config, ServerConfig};
use std::path::PathBuf;

fn test_dir(subdir: &str) -> PathBuf {
    std::env::temp_dir()
        .join("rustirc_integration_tests")
        .join("config")
        .join(subdir)
}

fn cleanup(subdir: &str) {
    let _ = std::fs::remove_dir_all(test_dir(subdir));
}

#[test]
fn test_config_save_and_load_roundtrip() {
    let name = "roundtrip";
    cleanup(name);
    let path = test_dir(name).join("roundtrip.toml");

    let mut config = Config::default();
    config.user.nickname = "IntegrationTest".to_string();
    config.servers.push(ServerConfig {
        name: "TestServer".to_string(),
        address: "irc.test.com".to_string(),
        port: 6697,
        use_tls: true,
        channels: vec![ChannelConfig {
            name: "#test".to_string(),
            key: None,
            auto_join: true,
        }],
        ..Default::default()
    });

    config.save(&path).unwrap();
    assert!(path.exists());

    let loaded = Config::from_file(&path).unwrap();
    assert_eq!(loaded.user.nickname, "IntegrationTest");
    assert_eq!(loaded.servers.len(), 1);
    assert_eq!(loaded.servers[0].address, "irc.test.com");
    assert_eq!(loaded.servers[0].channels.len(), 1);
    assert_eq!(loaded.servers[0].channels[0].name, "#test");

    cleanup(name);
}

#[test]
fn test_config_creates_parent_directories() {
    let name = "parent_dirs";
    cleanup(name);
    let path = test_dir(name)
        .join("deep")
        .join("nested")
        .join("config.toml");

    let config = Config::default();
    config.save(&path).unwrap();
    assert!(path.exists());

    cleanup(name);
}

#[test]
fn test_config_forward_compatibility() {
    let name = "forward_compat";
    cleanup(name);
    let dir = test_dir(name);
    let path = dir.join("minimal.toml");
    std::fs::create_dir_all(&dir).unwrap();

    // Write a minimal config that doesn't include new fields
    std::fs::write(
        &path,
        r#"
[user]
nickname = "MinimalUser"

[ui]
theme = "light"
"#,
    )
    .unwrap();

    let config = Config::from_file(&path).unwrap();
    assert_eq!(config.user.nickname, "MinimalUser");
    assert_eq!(config.ui.theme, "light");
    // New fields should have defaults
    assert!(config.flood.enabled);
    assert!(config.dcc.enabled);
    assert!(config.notifications.enabled);

    cleanup(name);
}

#[test]
fn test_config_default_path_is_valid() {
    let path = Config::default_path();
    assert!(path.to_string_lossy().contains("rustirc"));
    assert!(path.to_string_lossy().ends_with("config.toml"));
}

#[test]
fn test_generate_default_config() {
    let name = "generate";
    cleanup(name);
    let path = test_dir(name).join("generated.toml");

    Config::generate_default_config(&path).unwrap();
    assert!(path.exists());

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("RustIRC Configuration File"));
    assert!(content.contains("irc.libera.chat"));

    // Verify the generated config can be loaded
    let config = Config::from_file(&path).unwrap();
    assert_eq!(config.servers.len(), 1);
    assert_eq!(config.servers[0].name, "Libera Chat");

    cleanup(name);
}

#[test]
fn test_config_with_all_sections() {
    let name = "all_sections";
    cleanup(name);
    let path = test_dir(name).join("full.toml");

    let mut config = Config::default();
    config.flood.messages_per_second = 3.0;
    config.dcc.download_dir = "/tmp/downloads".to_string();
    config.notifications.highlight_words = vec!["alert".to_string()];

    config.save(&path).unwrap();
    let loaded = Config::from_file(&path).unwrap();

    assert_eq!(loaded.flood.messages_per_second, 3.0);
    assert_eq!(loaded.dcc.download_dir, "/tmp/downloads");
    assert_eq!(loaded.notifications.highlight_words, vec!["alert"]);

    cleanup(name);
}
