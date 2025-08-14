//! Plugin manager

use anyhow::Result;

pub struct PluginManager {
    // Will be implemented in Phase 4
}

impl PluginManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn load_plugin(&mut self, _path: &str) -> Result<()> {
        // Will be implemented in Phase 4
        Ok(())
    }
}