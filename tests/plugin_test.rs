//! Integration tests for plugin system

use rustirc_plugins::api::{
    PluginApi, PluginCapabilities, PluginContext, PluginInfo, PluginResult,
};
use rustirc_plugins::builtin::{HighlightPlugin, LoggerPlugin};
use rustirc_plugins::PluginManager;

struct CounterPlugin {
    name: String,
    init_count: u32,
    shutdown_count: u32,
    enabled: bool,
}

impl CounterPlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            init_count: 0,
            shutdown_count: 0,
            enabled: false,
        }
    }
}

impl PluginApi for CounterPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name.clone(),
            version: "1.0.0".to_string(),
            description: "Test counter plugin".to_string(),
            authors: vec![],
            capabilities: PluginCapabilities {
                handles_events: true,
                ..Default::default()
            },
        }
    }
    fn init(&mut self, _ctx: &mut PluginContext) -> PluginResult<()> {
        self.init_count += 1;
        self.enabled = true;
        Ok(())
    }
    fn shutdown(&mut self) -> PluginResult<()> {
        self.shutdown_count += 1;
        self.enabled = false;
        Ok(())
    }
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn set_enabled(&mut self, enabled: bool) -> PluginResult<()> {
        self.enabled = enabled;
        Ok(())
    }
}

#[test]
fn test_register_and_list_plugins() {
    let mut manager = PluginManager::new();
    manager
        .register_plugin(Box::new(CounterPlugin::new("plugin_a")))
        .unwrap();
    manager
        .register_plugin(Box::new(CounterPlugin::new("plugin_b")))
        .unwrap();

    let plugins = manager.list_plugins();
    assert_eq!(plugins.len(), 2);
    assert!(plugins.contains(&"plugin_a".to_string()));
    assert!(plugins.contains(&"plugin_b".to_string()));
}

#[test]
fn test_plugin_enable_disable() {
    let mut manager = PluginManager::new();
    manager
        .register_plugin(Box::new(CounterPlugin::new("toggle")))
        .unwrap();

    assert!(manager.is_plugin_enabled("toggle"));

    manager.disable_plugin("toggle").unwrap();
    assert!(!manager.is_plugin_enabled("toggle"));

    manager.enable_plugin("toggle").unwrap();
    assert!(manager.is_plugin_enabled("toggle"));
}

#[test]
fn test_unload_plugin() {
    let mut manager = PluginManager::new();
    manager
        .register_plugin(Box::new(CounterPlugin::new("removable")))
        .unwrap();

    assert!(manager.unload_plugin("removable").unwrap());
    assert!(manager.list_plugins().is_empty());
    assert!(!manager.unload_plugin("removable").unwrap());
}

#[test]
fn test_builtin_highlight_plugin() {
    let mut plugin = HighlightPlugin::new(vec!["rust".to_string(), "irc".to_string()]);

    let matches = plugin.check_message("I love Rust programming");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0], "rust");

    let matches = plugin.check_message("normal message");
    assert!(matches.is_empty());

    plugin.add_word("test".to_string());
    assert_eq!(plugin.words().len(), 3);

    assert!(plugin.remove_word("irc"));
    assert_eq!(plugin.words().len(), 2);
}

#[test]
fn test_builtin_logger_plugin() {
    let dir = std::env::temp_dir().join("rustirc_plugin_test_logs");
    let _ = std::fs::remove_dir_all(&dir);

    let mut plugin = LoggerPlugin::new(dir.to_string_lossy().to_string());
    let mut ctx = PluginContext::new();
    plugin.init(&mut ctx).unwrap();
    assert!(dir.exists());

    plugin.shutdown().unwrap();

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_plugin_manager_shutdown_all() {
    let mut manager = PluginManager::new();
    manager
        .register_plugin(Box::new(CounterPlugin::new("s1")))
        .unwrap();
    manager
        .register_plugin(Box::new(CounterPlugin::new("s2")))
        .unwrap();
    manager
        .register_plugin(Box::new(CounterPlugin::new("s3")))
        .unwrap();

    manager.shutdown_all();
    assert!(manager.list_plugins().is_empty());
}

#[test]
fn test_plugin_info_retrieval() {
    let mut manager = PluginManager::new();
    manager
        .register_plugin(Box::new(CounterPlugin::new("info_test")))
        .unwrap();

    let info = manager.get_plugin_info("info_test");
    assert!(info.is_some());
    let info = info.unwrap();
    assert_eq!(info.name, "info_test");
    assert_eq!(info.version, "1.0.0");
    assert!(info.capabilities.handles_events);
}
