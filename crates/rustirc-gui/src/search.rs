//! Full-text message search engine for RustIRC GUI
//!
//! Provides search functionality across IRC message history with filtering
//! by channel, user, date range, and case sensitivity options.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// A query describing what to search for and how to filter results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    /// The text to search for in message content
    pub text: String,
    /// Optional channel name to restrict search to
    pub channel_filter: Option<String>,
    /// Optional user nick to restrict search to
    pub user_filter: Option<String>,
    /// Optional start date for the search range
    pub date_from: Option<DateTime<Local>>,
    /// Optional end date for the search range
    pub date_to: Option<DateTime<Local>>,
    /// Whether the search should be case-sensitive
    pub case_sensitive: bool,
}

/// A single search result matching the query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The full text of the matching message
    pub message_text: String,
    /// Who sent the message
    pub sender: String,
    /// Which channel the message was in (empty for private messages)
    pub channel: String,
    /// When the message was sent
    pub timestamp: DateTime<Local>,
    /// The line number or index of the message in its context
    pub line_number: usize,
}

/// GUI-facing search state for tracking active search sessions.
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// The current search query
    pub query: SearchQuery,
    /// Results from the most recent search
    pub results: Vec<SearchResult>,
    /// Index of the currently selected/highlighted result
    pub selected_index: Option<usize>,
    /// Whether a search is currently in progress
    pub is_searching: bool,
}

/// A stored message entry that the search engine operates over.
#[derive(Debug, Clone)]
pub struct MessageRecord {
    /// The message text content
    pub text: String,
    /// Who sent the message
    pub sender: String,
    /// Which channel the message belongs to
    pub channel: String,
    /// When the message was received
    pub timestamp: DateTime<Local>,
}

/// Full-text search engine that operates over a collection of message records.
///
/// Maintains an internal store of messages and supports filtered search
/// with navigation through results.
#[derive(Debug, Clone, Default)]
pub struct SearchEngine {
    /// All messages available for searching
    messages: Vec<MessageRecord>,
}

impl SearchEngine {
    /// Create a new empty search engine.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Add a message record to the searchable store.
    pub fn add_message(&mut self, record: MessageRecord) {
        self.messages.push(record);
    }

    /// Execute a search query against the stored messages.
    ///
    /// Returns a `SearchState` populated with matching results.
    /// Filters are applied in order: text match, channel, user, date range.
    pub fn search(&self, query: &SearchQuery) -> SearchState {
        if query.text.is_empty() {
            return SearchState {
                query: query.clone(),
                results: Vec::new(),
                selected_index: None,
                is_searching: false,
            };
        }

        let search_text = if query.case_sensitive {
            query.text.clone()
        } else {
            query.text.to_lowercase()
        };

        let mut results = Vec::new();

        for (idx, msg) in self.messages.iter().enumerate() {
            let msg_text = if query.case_sensitive {
                msg.text.clone()
            } else {
                msg.text.to_lowercase()
            };

            // Text match
            if !msg_text.contains(&search_text) {
                continue;
            }

            // Channel filter
            if let Some(ref ch) = query.channel_filter {
                if !ch.is_empty() && msg.channel != *ch {
                    continue;
                }
            }

            // User filter
            if let Some(ref user) = query.user_filter {
                if !user.is_empty() && msg.sender != *user {
                    continue;
                }
            }

            // Date range filter
            if let Some(ref from) = query.date_from {
                if msg.timestamp < *from {
                    continue;
                }
            }
            if let Some(ref to) = query.date_to {
                if msg.timestamp > *to {
                    continue;
                }
            }

            results.push(SearchResult {
                message_text: msg.text.clone(),
                sender: msg.sender.clone(),
                channel: msg.channel.clone(),
                timestamp: msg.timestamp,
                line_number: idx + 1,
            });
        }

        let selected = if results.is_empty() { None } else { Some(0) };

        SearchState {
            query: query.clone(),
            results,
            selected_index: selected,
            is_searching: false,
        }
    }
}

impl SearchState {
    /// Move to the next search result, wrapping around at the end.
    pub fn next_result(&mut self) {
        if self.results.is_empty() {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            Some(idx) => (idx + 1) % self.results.len(),
            None => 0,
        });
    }

    /// Move to the previous search result, wrapping around at the beginning.
    pub fn prev_result(&mut self) {
        if self.results.is_empty() {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            Some(0) => self.results.len() - 1,
            Some(idx) => idx - 1,
            None => self.results.len() - 1,
        });
    }

    /// Clear the search state, removing all results and the query.
    pub fn clear(&mut self) {
        self.query = SearchQuery::default();
        self.results.clear();
        self.selected_index = None;
        self.is_searching = false;
    }

    /// Get the currently selected search result, if any.
    pub fn current_result(&self) -> Option<&SearchResult> {
        self.selected_index.and_then(|idx| self.results.get(idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine_with_messages() -> SearchEngine {
        let mut engine = SearchEngine::new();
        let base_time = Local::now();

        let messages = vec![
            ("Hello everyone!", "alice", "#general"),
            ("HELLO WORLD", "bob", "#general"),
            ("Does anyone know about Rust?", "charlie", "#dev"),
            ("hello from private", "dave", ""),
            ("Goodbye world", "alice", "#general"),
        ];

        for (i, (text, sender, channel)) in messages.into_iter().enumerate() {
            engine.add_message(MessageRecord {
                text: text.to_string(),
                sender: sender.to_string(),
                channel: channel.to_string(),
                timestamp: base_time + chrono::Duration::seconds(i as i64),
            });
        }

        engine
    }

    #[test]
    fn test_basic_search_case_insensitive() {
        let engine = make_engine_with_messages();
        let query = SearchQuery {
            text: "hello".to_string(),
            ..Default::default()
        };

        let state = engine.search(&query);
        assert_eq!(state.results.len(), 3);
        assert_eq!(state.selected_index, Some(0));
    }

    #[test]
    fn test_search_with_channel_filter() {
        let engine = make_engine_with_messages();
        let query = SearchQuery {
            text: "hello".to_string(),
            channel_filter: Some("#general".to_string()),
            ..Default::default()
        };

        let state = engine.search(&query);
        assert_eq!(state.results.len(), 2);
        assert!(state.results.iter().all(|r| r.channel == "#general"));
    }

    #[test]
    fn test_search_with_user_filter() {
        let engine = make_engine_with_messages();
        let query = SearchQuery {
            text: "hello".to_string(),
            user_filter: Some("alice".to_string()),
            ..Default::default()
        };

        let state = engine.search(&query);
        assert_eq!(state.results.len(), 1);
        assert_eq!(state.results[0].sender, "alice");
    }

    #[test]
    fn test_case_sensitive_search() {
        let engine = make_engine_with_messages();
        let query = SearchQuery {
            text: "HELLO".to_string(),
            case_sensitive: true,
            ..Default::default()
        };

        let state = engine.search(&query);
        assert_eq!(state.results.len(), 1);
        assert_eq!(state.results[0].sender, "bob");
    }

    #[test]
    fn test_empty_query_returns_no_results() {
        let engine = make_engine_with_messages();
        let query = SearchQuery::default();

        let state = engine.search(&query);
        assert!(state.results.is_empty());
        assert_eq!(state.selected_index, None);
    }

    #[test]
    fn test_navigation_next_prev() {
        let engine = make_engine_with_messages();
        let query = SearchQuery {
            text: "hello".to_string(),
            ..Default::default()
        };

        let mut state = engine.search(&query);
        assert_eq!(state.selected_index, Some(0));

        state.next_result();
        assert_eq!(state.selected_index, Some(1));

        state.next_result();
        assert_eq!(state.selected_index, Some(2));

        // Wrap around
        state.next_result();
        assert_eq!(state.selected_index, Some(0));

        // Previous from 0 wraps to end
        state.prev_result();
        assert_eq!(state.selected_index, Some(2));
    }

    #[test]
    fn test_clear_resets_state() {
        let engine = make_engine_with_messages();
        let query = SearchQuery {
            text: "hello".to_string(),
            ..Default::default()
        };

        let mut state = engine.search(&query);
        assert!(!state.results.is_empty());

        state.clear();
        assert!(state.results.is_empty());
        assert_eq!(state.selected_index, None);
        assert!(state.query.text.is_empty());
    }
}
