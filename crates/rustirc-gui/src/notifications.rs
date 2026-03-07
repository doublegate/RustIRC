//! Notification rules engine for RustIRC GUI
//!
//! Provides configurable notification filtering with highlight words,
//! nick mentions, channel/user filters, quiet hours, and notification history.

use chrono::{Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Quiet hours configuration to suppress notifications during specified times.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    /// Start hour (0-23) for quiet period
    pub start_hour: u8,
    /// End hour (0-23) for quiet period
    pub end_hour: u8,
    /// Whether quiet hours apply on weekends (Saturday and Sunday)
    pub weekends: bool,
    /// Whether quiet hours are currently enabled
    pub enabled: bool,
}

impl Default for QuietHours {
    fn default() -> Self {
        Self {
            start_hour: 22,
            end_hour: 8,
            weekends: false,
            enabled: false,
        }
    }
}

/// Type of notification that was triggered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    /// A configured highlight word was found in the message
    Highlight,
    /// The user's own nick was mentioned in the message
    NickMention,
    /// A private message was received
    PrivateMessage,
    /// A custom notification rule was triggered
    Custom(String),
}

/// A recorded notification entry for history tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationEntry {
    /// When the notification was generated
    pub timestamp: chrono::DateTime<Local>,
    /// Who sent the message that triggered the notification
    pub source: String,
    /// Which channel the message was in (empty for private messages)
    pub channel: String,
    /// The message content that triggered the notification
    pub message: String,
    /// What type of notification was triggered
    pub notification_type: NotificationType,
}

/// Notification rules engine that determines when and how to notify the user.
///
/// Supports highlight words, nick mention detection, per-channel and per-user
/// filtering, quiet hours, and maintains a notification history log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRules {
    /// Words that trigger highlight notifications when found in messages
    pub highlight_words: Vec<String>,
    /// Whether to notify when the user's nick is mentioned
    pub nick_mentions: bool,
    /// Per-channel notification enable/disable (channel name -> enabled)
    pub channel_filters: HashMap<String, bool>,
    /// Per-user notification enable/disable (nick -> enabled; false = blocked)
    pub user_filters: HashMap<String, bool>,
    /// Optional quiet hours configuration
    pub quiet_hours: Option<QuietHours>,
    /// History of past notifications
    #[serde(skip)]
    history: Vec<NotificationEntry>,
    /// Maximum number of history entries to retain
    pub max_history: usize,
}

impl Default for NotificationRules {
    fn default() -> Self {
        Self {
            highlight_words: Vec::new(),
            nick_mentions: true,
            channel_filters: HashMap::new(),
            user_filters: HashMap::new(),
            quiet_hours: None,
            history: Vec::new(),
            max_history: 500,
        }
    }
}

impl NotificationRules {
    /// Create a new `NotificationRules` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Determine whether a message should trigger a notification.
    ///
    /// Returns `Some(NotificationType)` if a notification should fire,
    /// or `None` if the message should be silently ignored.
    ///
    /// Checks are performed in this priority order:
    /// 1. Quiet hours suppression
    /// 2. User filter (blocked users produce no notification)
    /// 3. Channel filter (disabled channels produce no notification)
    /// 4. Private message detection (channel is empty)
    /// 5. Nick mention detection
    /// 6. Highlight word matching
    pub fn should_notify(
        &self,
        nick: &str,
        channel: &str,
        message: &str,
        own_nick: &str,
    ) -> Option<NotificationType> {
        // Suppress all notifications during quiet hours
        if self.is_quiet_hours() {
            return None;
        }

        // Check if the user is blocked
        if let Some(&enabled) = self.user_filters.get(nick) {
            if !enabled {
                return None;
            }
        }

        // Check if the channel has notifications disabled
        if !channel.is_empty() {
            if let Some(&enabled) = self.channel_filters.get(channel) {
                if !enabled {
                    return None;
                }
            }
        }

        // Private messages always notify (channel is empty)
        if channel.is_empty() {
            return Some(NotificationType::PrivateMessage);
        }

        // Check for nick mention
        if self.nick_mentions && !own_nick.is_empty() {
            let msg_lower = message.to_lowercase();
            let nick_lower = own_nick.to_lowercase();
            if msg_lower.contains(&nick_lower) {
                return Some(NotificationType::NickMention);
            }
        }

        // Check highlight words
        let msg_lower = message.to_lowercase();
        for word in &self.highlight_words {
            let word_lower = word.to_lowercase();
            if msg_lower.contains(&word_lower) {
                return Some(NotificationType::Highlight);
            }
        }

        None
    }

    /// Check whether the current time falls within configured quiet hours.
    ///
    /// Returns `false` if quiet hours are not configured or not enabled.
    /// When `weekends` is false, quiet hours do not apply on Saturday or Sunday.
    pub fn is_quiet_hours(&self) -> bool {
        let qh = match &self.quiet_hours {
            Some(qh) if qh.enabled => qh,
            _ => return false,
        };

        let now = Local::now();
        let current_hour = now.hour() as u8;
        let weekday = now.weekday();

        // Skip quiet hours on weekends if not configured for weekends
        if !qh.weekends {
            use chrono::Weekday;
            if weekday == Weekday::Sat || weekday == Weekday::Sun {
                return false;
            }
        }

        // Handle wrapping (e.g., 22:00 to 08:00)
        if qh.start_hour <= qh.end_hour {
            // Simple range: e.g., 09:00 to 17:00
            current_hour >= qh.start_hour && current_hour < qh.end_hour
        } else {
            // Wrapping range: e.g., 22:00 to 08:00
            current_hour >= qh.start_hour || current_hour < qh.end_hour
        }
    }

    /// Add a highlight word to the notification rules.
    ///
    /// Duplicate words (case-insensitive) are not added.
    pub fn add_highlight_word(&mut self, word: String) {
        let word_lower = word.to_lowercase();
        let already_exists = self
            .highlight_words
            .iter()
            .any(|w| w.to_lowercase() == word_lower);
        if !already_exists {
            self.highlight_words.push(word);
        }
    }

    /// Remove a highlight word from the notification rules.
    ///
    /// Matching is case-insensitive. Returns `true` if a word was removed.
    pub fn remove_highlight_word(&mut self, word: &str) -> bool {
        let word_lower = word.to_lowercase();
        let before = self.highlight_words.len();
        self.highlight_words
            .retain(|w| w.to_lowercase() != word_lower);
        self.highlight_words.len() < before
    }

    /// Record a notification in the history log.
    ///
    /// Automatically trims old entries if the history exceeds `max_history`.
    pub fn log_notification(
        &mut self,
        source: String,
        channel: String,
        message: String,
        notification_type: NotificationType,
    ) {
        let entry = NotificationEntry {
            timestamp: Local::now(),
            source,
            channel,
            message,
            notification_type,
        };
        self.history.push(entry);

        // Trim history if it exceeds the maximum
        if self.history.len() > self.max_history {
            let excess = self.history.len() - self.max_history;
            self.history.drain(..excess);
        }
    }

    /// Get the notification history log.
    pub fn get_history(&self) -> &[NotificationEntry] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_word_notification() {
        let mut rules = NotificationRules::new();
        rules.add_highlight_word("urgent".to_string());

        let result = rules.should_notify("alice", "#general", "This is urgent!", "bob");
        assert_eq!(result, Some(NotificationType::Highlight));
    }

    #[test]
    fn test_nick_mention_notification() {
        let rules = NotificationRules::new();

        let result = rules.should_notify("alice", "#general", "hey Bob, check this out", "Bob");
        assert_eq!(result, Some(NotificationType::NickMention));
    }

    #[test]
    fn test_private_message_notification() {
        let rules = NotificationRules::new();

        let result = rules.should_notify("alice", "", "hello there", "bob");
        assert_eq!(result, Some(NotificationType::PrivateMessage));
    }

    #[test]
    fn test_blocked_user_suppressed() {
        let mut rules = NotificationRules::new();
        rules.user_filters.insert("spammer".to_string(), false);

        let result = rules.should_notify("spammer", "", "buy my stuff", "bob");
        assert_eq!(result, None);
    }

    #[test]
    fn test_disabled_channel_suppressed() {
        let mut rules = NotificationRules::new();
        rules.add_highlight_word("important".to_string());
        rules.channel_filters.insert("#noisy".to_string(), false);

        let result = rules.should_notify("alice", "#noisy", "something important", "bob");
        assert_eq!(result, None);
    }

    #[test]
    fn test_no_match_returns_none() {
        let rules = NotificationRules::new();

        let result = rules.should_notify("alice", "#general", "just chatting", "bob");
        assert_eq!(result, None);
    }

    #[test]
    fn test_add_and_remove_highlight_words() {
        let mut rules = NotificationRules::new();
        rules.add_highlight_word("alert".to_string());
        rules.add_highlight_word("ALERT".to_string()); // duplicate, should not be added
        assert_eq!(rules.highlight_words.len(), 1);

        let removed = rules.remove_highlight_word("Alert");
        assert!(removed);
        assert!(rules.highlight_words.is_empty());

        let not_removed = rules.remove_highlight_word("nonexistent");
        assert!(!not_removed);
    }

    #[test]
    fn test_notification_history_logging() {
        let mut rules = NotificationRules::new();
        rules.max_history = 3;

        for i in 0..5 {
            rules.log_notification(
                format!("user{i}"),
                "#test".to_string(),
                format!("message {i}"),
                NotificationType::Highlight,
            );
        }

        let history = rules.get_history();
        assert_eq!(history.len(), 3);
        // Oldest entries should have been trimmed
        assert_eq!(history[0].source, "user2");
        assert_eq!(history[2].source, "user4");
    }

    #[test]
    fn test_quiet_hours_disabled_by_default() {
        let rules = NotificationRules::new();
        assert!(!rules.is_quiet_hours());
    }

    #[test]
    fn test_case_insensitive_highlight() {
        let mut rules = NotificationRules::new();
        rules.add_highlight_word("ERROR".to_string());

        let result = rules.should_notify("alice", "#dev", "there was an error in the build", "bob");
        assert_eq!(result, Some(NotificationType::Highlight));
    }
}
