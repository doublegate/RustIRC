//! Plugin lifecycle management

use crate::api::{PluginApi, PluginContext, PluginInfo, PluginResult};
use std::collections::HashMap;
use tracing::{error, info};

/// A loaded plugin instance with metadata
struct LoadedPlugin {
    instance: Box<dyn PluginApi>,
    info: PluginInfo,
    enabled: bool,
}

/// Manages the lifecycle of all registered plugins
pub struct PluginManager {
    plugins: HashMap<String, LoadedPlugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a new plugin instance
    pub fn register_plugin(&mut self, mut plugin: Box<dyn PluginApi>) -> PluginResult<()> {
        let info = plugin.info();
        let name = info.name.clone();

        let mut context = PluginContext::new();
        plugin.init(&mut context)?;

        info!("Registered plugin: {} v{}", name, info.version);

        self.plugins.insert(
            name,
            LoadedPlugin {
                instance: plugin,
                info,
                enabled: true,
            },
        );

        Ok(())
    }

    /// Unload a plugin by name
    pub fn unload_plugin(&mut self, name: &str) -> PluginResult<bool> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            plugin.instance.shutdown()?;
            info!("Unloaded plugin: {}", name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get plugin info by name
    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins.get(name).map(|p| &p.info)
    }

    /// List all registered plugin names
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, name: &str) -> PluginResult<bool> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = true;
            plugin.instance.set_enabled(true)?;
            info!("Enabled plugin: {}", name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) -> PluginResult<bool> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.enabled = false;
            plugin.instance.set_enabled(false)?;
            info!("Disabled plugin: {}", name);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if a plugin is enabled
    pub fn is_plugin_enabled(&self, name: &str) -> bool {
        self.plugins.get(name).map(|p| p.enabled).unwrap_or(false)
    }

    /// Shutdown all plugins
    pub fn shutdown_all(&mut self) {
        for (name, plugin) in self.plugins.iter_mut() {
            if let Err(e) = plugin.instance.shutdown() {
                error!("Error shutting down plugin {}: {}", name, e);
            }
        }
        self.plugins.clear();
        info!("All plugins shut down");
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::PluginCapabilities;

    struct TestPlugin {
        name: String,
        enabled: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                enabled: false,
            }
        }
    }

    impl PluginApi for TestPlugin {
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
                description: "Test plugin".to_string(),
                authors: vec![],
                capabilities: PluginCapabilities::default(),
            }
        }
        fn init(&mut self, _context: &mut PluginContext) -> PluginResult<()> {
            self.enabled = true;
            Ok(())
        }
        fn shutdown(&mut self) -> PluginResult<()> {
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
    fn test_register_plugin() {
        let mut manager = PluginManager::new();
        let result = manager.register_plugin(Box::new(TestPlugin::new("test")));
        assert!(result.is_ok());
        assert_eq!(manager.list_plugins().len(), 1);
    }

    #[test]
    fn test_plugin_lifecycle() {
        let mut manager = PluginManager::new();
        manager
            .register_plugin(Box::new(TestPlugin::new("lifecycle")))
            .unwrap();

        assert!(manager.is_plugin_enabled("lifecycle"));

        manager.disable_plugin("lifecycle").unwrap();
        assert!(!manager.is_plugin_enabled("lifecycle"));

        manager.enable_plugin("lifecycle").unwrap();
        assert!(manager.is_plugin_enabled("lifecycle"));
    }

    #[test]
    fn test_unload_plugin() {
        let mut manager = PluginManager::new();
        manager
            .register_plugin(Box::new(TestPlugin::new("unload")))
            .unwrap();

        let removed = manager.unload_plugin("unload").unwrap();
        assert!(removed);
        assert!(manager.list_plugins().is_empty());

        let removed = manager.unload_plugin("nonexistent").unwrap();
        assert!(!removed);
    }

    #[test]
    fn test_multiple_plugins() {
        let mut manager = PluginManager::new();
        manager
            .register_plugin(Box::new(TestPlugin::new("p1")))
            .unwrap();
        manager
            .register_plugin(Box::new(TestPlugin::new("p2")))
            .unwrap();
        assert_eq!(manager.list_plugins().len(), 2);
    }

    #[test]
    fn test_get_plugin_info() {
        let mut manager = PluginManager::new();
        manager
            .register_plugin(Box::new(TestPlugin::new("info_test")))
            .unwrap();

        let info = manager.get_plugin_info("info_test");
        assert!(info.is_some());
        assert_eq!(info.unwrap().version, "1.0.0");

        assert!(manager.get_plugin_info("nonexistent").is_none());
    }

    #[test]
    fn test_shutdown_all() {
        let mut manager = PluginManager::new();
        manager
            .register_plugin(Box::new(TestPlugin::new("s1")))
            .unwrap();
        manager
            .register_plugin(Box::new(TestPlugin::new("s2")))
            .unwrap();

        manager.shutdown_all();
        assert!(manager.list_plugins().is_empty());
    }

    #[test]
    fn test_enable_disable() {
        let mut manager = PluginManager::new();
        manager
            .register_plugin(Box::new(TestPlugin::new("toggle")))
            .unwrap();

        assert!(manager.disable_plugin("toggle").unwrap());
        assert!(!manager.is_plugin_enabled("toggle"));

        assert!(manager.enable_plugin("toggle").unwrap());
        assert!(manager.is_plugin_enabled("toggle"));

        assert!(!manager.enable_plugin("nonexistent").unwrap());
    }
}
