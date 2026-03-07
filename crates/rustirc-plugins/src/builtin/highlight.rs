//! Highlight plugin - keyword notifications for IRC messages

use crate::api::{PluginApi, PluginCapabilities, PluginContext, PluginInfo, PluginResult};

/// Built-in plugin that triggers notifications when highlight words are mentioned
pub struct HighlightPlugin {
    words: Vec<String>,
    enabled: bool,
}

impl HighlightPlugin {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            words,
            enabled: true,
        }
    }

    /// Check if a message contains any highlight words
    pub fn check_message(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        self.words
            .iter()
            .filter(|w| text_lower.contains(&w.to_lowercase()))
            .cloned()
            .collect()
    }

    /// Add a highlight word
    pub fn add_word(&mut self, word: String) {
        if !self.words.contains(&word) {
            self.words.push(word);
        }
    }

    /// Remove a highlight word
    pub fn remove_word(&mut self, word: &str) -> bool {
        let len = self.words.len();
        self.words.retain(|w| w != word);
        self.words.len() < len
    }

    /// Get the list of highlight words
    pub fn words(&self) -> &[String] {
        &self.words
    }
}

impl PluginApi for HighlightPlugin {
    fn name(&self) -> &str {
        "Highlight"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "Highlight".to_string(),
            version: "1.0.0".to_string(),
            description: "Triggers notifications for highlight words".to_string(),
            authors: vec!["RustIRC Contributors".to_string()],
            capabilities: PluginCapabilities {
                handles_events: true,
                ..Default::default()
            },
        }
    }

    fn init(&mut self, _context: &mut PluginContext) -> PluginResult<()> {
        tracing::info!(
            "Highlight plugin initialized with {} words",
            self.words.len()
        );
        Ok(())
    }

    fn shutdown(&mut self) -> PluginResult<()> {
        tracing::info!("Highlight plugin shutting down");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_message() {
        let plugin = HighlightPlugin::new(vec!["hello".to_string(), "rust".to_string()]);
        let matches = plugin.check_message("Hello World");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], "hello");
    }

    #[test]
    fn test_no_match() {
        let plugin = HighlightPlugin::new(vec!["hello".to_string()]);
        let matches = plugin.check_message("goodbye world");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_add_remove_word() {
        let mut plugin = HighlightPlugin::new(vec![]);
        plugin.add_word("test".to_string());
        assert_eq!(plugin.words().len(), 1);

        // Don't add duplicates
        plugin.add_word("test".to_string());
        assert_eq!(plugin.words().len(), 1);

        assert!(plugin.remove_word("test"));
        assert!(plugin.words().is_empty());
        assert!(!plugin.remove_word("nonexistent"));
    }
}
