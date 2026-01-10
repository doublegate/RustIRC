//! Plugin API for extending IRC client functionality
//!
//! This module provides the plugin system that allows extending the IRC client
//! with custom functionality through Rust plugins. Plugins can handle events,
//! register commands, provide UI components, and integrate with external services.
//!
//! # Examples
//!
//! ```rust
//! use rustirc_plugins::api::{PluginApi, PluginContext, PluginResult};
//!
//! struct EchoPlugin;
//!
//! impl PluginApi for EchoPlugin {
//!     fn name(&self) -> &str {
//!         "Echo Plugin"
//!     }
//!     
//!     fn version(&self) -> &str {
//!         "1.0.0"
//!     }
//!     
//!     fn init(&mut self, _context: &mut PluginContext) -> PluginResult<()> {
//!         println!("Echo plugin initialized!");
//!         Ok(())
//!     }
//!     
//!     fn shutdown(&mut self) -> PluginResult<()> {
//!         println!("Echo plugin shutting down!");
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Plugin Capabilities
//!
//! When fully implemented, plugins will be able to:
//! - Register custom IRC commands
//! - Handle IRC events and messages
//! - Provide GUI components and dialogs
//! - Store and retrieve persistent data
//! - Communicate with other plugins
//! - Access network resources
//! - Integrate with external APIs

use std::collections::HashMap;
use std::error::Error;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Plugin capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PluginCapabilities {
    /// Can handle IRC messages and events
    pub handles_events: bool,
    /// Can register custom commands
    pub provides_commands: bool,
    /// Can provide GUI components
    pub provides_gui: bool,
    /// Can store persistent data
    pub uses_storage: bool,
    /// Can access network resources
    pub network_access: bool,
}

/// Plugin metadata and information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author(s)
    pub authors: Vec<String>,
    /// Plugin capabilities
    pub capabilities: PluginCapabilities,
}

/// Context provided to plugins during initialization
///
/// Contains references to client systems that plugins can interact with.
/// This is a placeholder for Phase 4 implementation.
pub struct PluginContext {
    /// Plugin configuration data
    pub config: HashMap<String, String>,
    /// Plugin data storage path
    pub data_path: Option<std::path::PathBuf>,
}

impl PluginContext {
    /// Create a new plugin context
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::PluginContext;
    ///
    /// let context = PluginContext::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
            data_path: None,
        }
    }

    /// Get a configuration value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::PluginContext;
    ///
    /// let context = PluginContext::new();
    /// let value = context.get_config("api_key");
    /// ```
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    /// Set a configuration value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::PluginContext;
    ///
    /// let mut context = PluginContext::new();
    /// context.set_config("api_key".to_string(), "secret123".to_string());
    /// ```
    pub fn set_config(&mut self, key: String, value: String) {
        self.config.insert(key, value);
    }
}

impl Default for PluginContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait that all IRC client plugins must implement
///
/// Provides the basic interface for plugin lifecycle management and
/// metadata. Plugins register with the client through this interface.
///
/// # Examples
///
/// ```rust
/// use rustirc_plugins::api::{PluginApi, PluginContext, PluginResult, PluginInfo, PluginCapabilities};
///
/// struct MyPlugin {
///     enabled: bool,
/// }
///
/// impl MyPlugin {
///     pub fn new() -> Self {
///         Self { enabled: false }
///     }
/// }
///
/// impl PluginApi for MyPlugin {
///     fn name(&self) -> &str {
///         "My Custom Plugin"
///     }
///     
///     fn version(&self) -> &str {
///         "1.0.0"
///     }
///     
///     fn info(&self) -> PluginInfo {
///         PluginInfo {
///             name: self.name().to_string(),
///             version: self.version().to_string(),
///             description: "A custom plugin for IRC client".to_string(),
///             authors: vec!["Plugin Author".to_string()],
///             capabilities: PluginCapabilities {
///                 handles_events: true,
///                 provides_commands: true,
///                 ..Default::default()
///             },
///         }
///     }
///     
///     fn init(&mut self, _context: &mut PluginContext) -> PluginResult<()> {
///         self.enabled = true;
///         println!("Plugin {} initialized", self.name());
///         Ok(())
///     }
///     
///     fn shutdown(&mut self) -> PluginResult<()> {
///         self.enabled = false;
///         println!("Plugin {} shutting down", self.name());
///         Ok(())
///     }
/// }
/// ```
pub trait PluginApi: Send + Sync {
    /// Get the plugin name
    ///
    /// Should return a unique, human-readable name for the plugin.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::PluginApi;
    ///
    /// struct MyPlugin;
    ///
    /// impl PluginApi for MyPlugin {
    ///     fn name(&self) -> &str {
    ///         "IRC Logger"
    ///     }
    ///     # fn version(&self) -> &str { "1.0.0" }
    ///     # fn init(&mut self, _: &mut rustirc_plugins::api::PluginContext) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    ///     # fn shutdown(&mut self) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    /// }
    /// ```
    fn name(&self) -> &str;

    /// Get the plugin version
    ///
    /// Should return a semantic version string (e.g., "1.2.3").
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::PluginApi;
    ///
    /// struct MyPlugin;
    ///
    /// impl PluginApi for MyPlugin {
    ///     fn version(&self) -> &str {
    ///         "2.1.0"
    ///     }
    ///     # fn name(&self) -> &str { "Test Plugin" }
    ///     # fn init(&mut self, _: &mut rustirc_plugins::api::PluginContext) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    ///     # fn shutdown(&mut self) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    /// }
    /// ```
    fn version(&self) -> &str;

    /// Get detailed plugin information
    ///
    /// Returns metadata about the plugin including description, authors,
    /// and capabilities. Default implementation creates info from name/version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::{PluginApi, PluginInfo, PluginCapabilities};
    ///
    /// struct MyPlugin;
    ///
    /// impl PluginApi for MyPlugin {
    ///     fn info(&self) -> PluginInfo {
    ///         PluginInfo {
    ///             name: "Advanced Logger".to_string(),
    ///             version: "1.0.0".to_string(),
    ///             description: "Logs IRC messages to files".to_string(),
    ///             authors: vec!["John Doe".to_string()],
    ///             capabilities: PluginCapabilities {
    ///                 handles_events: true,
    ///                 uses_storage: true,
    ///                 ..Default::default()
    ///             },
    ///         }
    ///     }
    ///     # fn name(&self) -> &str { "Advanced Logger" }
    ///     # fn version(&self) -> &str { "1.0.0" }
    ///     # fn init(&mut self, _: &mut rustirc_plugins::api::PluginContext) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    ///     # fn shutdown(&mut self) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    /// }
    /// ```
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: self.name().to_string(),
            version: self.version().to_string(),
            description: "No description provided".to_string(),
            authors: vec![],
            capabilities: PluginCapabilities::default(),
        }
    }

    /// Initialize the plugin
    ///
    /// Called when the plugin is loaded and should perform any setup
    /// required for the plugin to function. The context provides access
    /// to client systems and configuration.
    ///
    /// # Arguments
    ///
    /// * `context` - Plugin context with configuration and client access
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::{PluginApi, PluginContext, PluginResult};
    ///
    /// struct MyPlugin {
    ///     initialized: bool,
    /// }
    ///
    /// impl PluginApi for MyPlugin {
    ///     fn init(&mut self, context: &mut PluginContext) -> PluginResult<()> {
    ///         // Perform plugin initialization
    ///         self.initialized = true;
    ///         
    ///         // Check configuration
    ///         if let Some(config_value) = context.get_config("enabled") {
    ///             println!("Plugin config: {}", config_value);
    ///         }
    ///         
    ///         println!("Plugin {} v{} initialized", self.name(), self.version());
    ///         Ok(())
    ///     }
    ///     # fn name(&self) -> &str { "Test Plugin" }
    ///     # fn version(&self) -> &str { "1.0.0" }
    ///     # fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Should return an error if initialization fails. The plugin will not
    /// be loaded and other methods will not be called.
    fn init(&mut self, context: &mut PluginContext) -> PluginResult<()>;

    /// Shutdown the plugin
    ///
    /// Called when the plugin is being unloaded and should perform cleanup.
    /// This method should release any resources held by the plugin.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::{PluginApi, PluginResult};
    ///
    /// struct MyPlugin {
    ///     connections: Vec<String>,
    /// }
    ///
    /// impl PluginApi for MyPlugin {
    ///     fn shutdown(&mut self) -> PluginResult<()> {
    ///         // Clean up resources
    ///         self.connections.clear();
    ///         
    ///         // Save any persistent data
    ///         println!("Plugin {} shutting down cleanly", self.name());
    ///         Ok(())
    ///     }
    ///     # fn name(&self) -> &str { "Test Plugin" }
    ///     # fn version(&self) -> &str { "1.0.0" }
    ///     # fn init(&mut self, _: &mut rustirc_plugins::api::PluginContext) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Should return an error if shutdown encounters problems, but the
    /// plugin will still be unloaded.
    fn shutdown(&mut self) -> PluginResult<()>;

    /// Check if the plugin is currently enabled
    ///
    /// Default implementation returns true. Plugins can override this
    /// to provide dynamic enable/disable functionality.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::PluginApi;
    ///
    /// struct MyPlugin {
    ///     enabled: bool,
    /// }
    ///
    /// impl PluginApi for MyPlugin {
    ///     fn is_enabled(&self) -> bool {
    ///         self.enabled
    ///     }
    ///     # fn name(&self) -> &str { "Test Plugin" }
    ///     # fn version(&self) -> &str { "1.0.0" }
    ///     # fn init(&mut self, _: &mut rustirc_plugins::api::PluginContext) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    ///     # fn shutdown(&mut self) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    /// }
    /// ```
    fn is_enabled(&self) -> bool {
        true
    }

    /// Enable or disable the plugin
    ///
    /// Default implementation does nothing. Plugins can override this
    /// to support runtime enable/disable functionality.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether the plugin should be enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rustirc_plugins::api::{PluginApi, PluginResult};
    ///
    /// struct MyPlugin {
    ///     enabled: bool,
    /// }
    ///
    /// impl PluginApi for MyPlugin {
    ///     fn set_enabled(&mut self, enabled: bool) -> PluginResult<()> {
    ///         self.enabled = enabled;
    ///         println!("Plugin {} is now {}",
    ///                  self.name(),
    ///                  if enabled { "enabled" } else { "disabled" });
    ///         Ok(())
    ///     }
    ///     # fn name(&self) -> &str { "Test Plugin" }
    ///     # fn version(&self) -> &str { "1.0.0" }
    ///     # fn init(&mut self, _: &mut rustirc_plugins::api::PluginContext) -> rustirc_plugins::api::PluginResult<()> { Ok(()) }
    ///     # fn shutdown(&mut self) -> PluginResult<()> { Ok(()) }
    /// }
    /// ```
    fn set_enabled(&mut self, _enabled: bool) -> PluginResult<()> {
        Ok(())
    }
}
