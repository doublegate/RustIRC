//! Logger plugin - logs IRC messages to files

use crate::api::{PluginApi, PluginCapabilities, PluginContext, PluginInfo, PluginResult};
use std::path::PathBuf;

/// Built-in plugin that logs IRC messages to files
pub struct LoggerPlugin {
    log_dir: PathBuf,
    enabled: bool,
}

impl LoggerPlugin {
    pub fn new(log_dir: impl Into<String>) -> Self {
        Self {
            log_dir: PathBuf::from(log_dir.into()),
            enabled: true,
        }
    }

    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }
}

impl PluginApi for LoggerPlugin {
    fn name(&self) -> &str {
        "Logger"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Logger".to_string(),
            version: "1.0.0".to_string(),
            description: "Logs IRC messages to files".to_string(),
            authors: vec!["RustIRC Contributors".to_string()],
            capabilities: PluginCapabilities {
                handles_events: true,
                uses_storage: true,
                ..Default::default()
            },
        }
    }

    fn init(&mut self, _context: &mut PluginContext) -> PluginResult<()> {
        // Create log directory if it doesn't exist
        if !self.log_dir.exists() {
            std::fs::create_dir_all(&self.log_dir)?;
        }
        tracing::info!("Logger plugin initialized: {}", self.log_dir.display());
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("Logger plugin shutting down");
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
