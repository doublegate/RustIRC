//! Plugin discovery and loading

use std::path::PathBuf;

/// Plugin loader for discovering and loading plugins from the filesystem
pub struct PluginLoader {
    search_paths: Vec<PathBuf>,
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            search_paths: vec![Self::default_plugin_dir()],
        }
    }

    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        Self {
            search_paths: paths,
        }
    }

    /// Get the default plugin directory
    pub fn default_plugin_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rustirc")
            .join("plugins")
    }

    /// Discover plugin directories in search paths
    pub fn discover_plugins(&self) -> Vec<PathBuf> {
        let mut found = Vec::new();
        for search_path in &self.search_paths {
            if search_path.exists() {
                if let Ok(entries) = std::fs::read_dir(search_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            found.push(path);
                        }
                    }
                }
            }
        }
        found
    }

    /// Get the search paths
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }

    /// Add a search path
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
