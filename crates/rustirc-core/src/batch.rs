//! IRCv3 Batch message handler
//!
//! Implements the IRCv3 BATCH specification for grouping related messages.
//! Supports nested batches, multiple batch types, and message accumulation.
//!
//! See: <https://ircv3.net/specs/extensions/batch>

use std::collections::HashMap;

use rustirc_protocol::Message;

/// The type of an IRCv3 batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchType {
    /// Network join (netsplit recovery)
    Netjoin,
    /// Network split
    Netsplit,
    /// Chat history playback
    ChatHistory,
    /// Labeled response grouping
    LabeledResponse,
    /// Custom or unknown batch type
    Custom(String),
}

impl BatchType {
    /// Parse a batch type string into a `BatchType` variant.
    pub fn parse(s: &str) -> Self {
        match s {
            "netjoin" => BatchType::Netjoin,
            "netsplit" => BatchType::Netsplit,
            "chathistory" => BatchType::ChatHistory,
            "labeled-response" => BatchType::LabeledResponse,
            other => BatchType::Custom(other.to_string()),
        }
    }

    /// Return the wire-format string for this batch type.
    pub fn as_str(&self) -> &str {
        match self {
            BatchType::Netjoin => "netjoin",
            BatchType::Netsplit => "netsplit",
            BatchType::ChatHistory => "chathistory",
            BatchType::LabeledResponse => "labeled-response",
            BatchType::Custom(s) => s,
        }
    }
}

/// A single IRCv3 batch, accumulating messages between BATCH + and BATCH -.
#[derive(Debug, Clone)]
pub struct Batch {
    /// The reference tag identifying this batch (unique per connection).
    pub ref_tag: String,
    /// The type of batch.
    pub batch_type: BatchType,
    /// Messages collected within this batch.
    pub messages: Vec<Message>,
    /// Optional parent batch reference tag (for nested batches).
    pub parent_ref: Option<String>,
    /// Extra parameters from the BATCH + command (after the batch type).
    pub params: Vec<String>,
}

impl Batch {
    /// Create a new batch with the given reference tag and type.
    pub fn new(
        ref_tag: impl Into<String>,
        batch_type: BatchType,
        parent_ref: Option<String>,
        params: Vec<String>,
    ) -> Self {
        Self {
            ref_tag: ref_tag.into(),
            batch_type,
            messages: Vec::new(),
            parent_ref,
            params,
        }
    }
}

/// Manages open IRCv3 batches on a connection.
///
/// Tracks currently-open batches, handles start/end events, and routes
/// messages to the appropriate batch based on their `batch` tag.
#[derive(Debug, Default)]
pub struct BatchManager {
    /// Currently-open batches, keyed by reference tag.
    open_batches: HashMap<String, Batch>,
    /// Completed batches awaiting consumption, keyed by reference tag.
    completed_batches: HashMap<String, Batch>,
}

impl BatchManager {
    /// Create a new, empty `BatchManager`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle a BATCH + (start) command.
    ///
    /// Parses the reference tag, batch type, and optional parameters from the
    /// BATCH message params. If the message carries a `batch` tag itself, that
    /// indicates nesting and is recorded as `parent_ref`.
    ///
    /// Returns `Ok(ref_tag)` on success, or an error string if the params are
    /// malformed or the ref_tag is already in use.
    pub fn handle_batch_start(&mut self, message: &Message) -> Result<String, String> {
        // BATCH params: ["+<ref_tag>", "<type>", ...extra_params]
        if message.params.is_empty() {
            return Err("BATCH start missing reference tag".to_string());
        }

        let ref_param = &message.params[0];
        if !ref_param.starts_with('+') || ref_param.len() < 2 {
            return Err(format!(
                "BATCH start reference tag must begin with '+': {ref_param}"
            ));
        }
        let ref_tag = ref_param[1..].to_string();

        if self.open_batches.contains_key(&ref_tag) {
            return Err(format!("Duplicate batch reference tag: {ref_tag}"));
        }

        let batch_type = if message.params.len() > 1 {
            BatchType::parse(&message.params[1])
        } else {
            return Err("BATCH start missing batch type".to_string());
        };

        let extra_params = if message.params.len() > 2 {
            message.params[2..].to_vec()
        } else {
            Vec::new()
        };

        // Detect nesting: if this BATCH message itself has a `batch` tag,
        // that is the parent reference.
        let parent_ref = message.tags.as_ref().and_then(|tags| {
            tags.iter()
                .find(|t| t.key == "batch")
                .and_then(|t| t.value.clone())
        });

        let batch = Batch::new(ref_tag.clone(), batch_type, parent_ref, extra_params);
        self.open_batches.insert(ref_tag.clone(), batch);

        Ok(ref_tag)
    }

    /// Handle a BATCH - (end) command.
    ///
    /// Moves the batch from `open_batches` to `completed_batches`.
    /// Returns the completed `Batch` or an error if the ref_tag is unknown.
    pub fn handle_batch_end(&mut self, message: &Message) -> Result<Batch, String> {
        if message.params.is_empty() {
            return Err("BATCH end missing reference tag".to_string());
        }

        let ref_param = &message.params[0];
        if !ref_param.starts_with('-') || ref_param.len() < 2 {
            return Err(format!(
                "BATCH end reference tag must begin with '-': {ref_param}"
            ));
        }
        let ref_tag = &ref_param[1..];

        let batch = self
            .open_batches
            .remove(ref_tag)
            .ok_or_else(|| format!("Unknown batch reference tag: {ref_tag}"))?;

        self.completed_batches
            .insert(ref_tag.to_string(), batch.clone());

        Ok(batch)
    }

    /// Add a message to the appropriate open batch.
    ///
    /// The message must have an IRCv3 `batch` tag whose value matches an open
    /// batch reference tag. Returns `true` if the message was added, `false`
    /// if no matching batch was found.
    pub fn add_message(&mut self, message: &Message) -> bool {
        let batch_ref = match message.tags.as_ref().and_then(|tags| {
            tags.iter()
                .find(|t| t.key == "batch")
                .and_then(|t| t.value.clone())
        }) {
            Some(r) => r,
            None => return false,
        };

        if let Some(batch) = self.open_batches.get_mut(&batch_ref) {
            batch.messages.push(message.clone());
            true
        } else {
            false
        }
    }

    /// Retrieve a completed batch by reference tag without removing it.
    pub fn get_batch(&self, ref_tag: &str) -> Option<&Batch> {
        self.completed_batches.get(ref_tag)
    }

    /// Remove and return a completed batch by reference tag.
    pub fn take_batch(&mut self, ref_tag: &str) -> Option<Batch> {
        self.completed_batches.remove(ref_tag)
    }

    /// Check whether a reference tag corresponds to a currently-open batch.
    pub fn is_in_batch(&self, ref_tag: &str) -> bool {
        self.open_batches.contains_key(ref_tag)
    }

    /// Return the number of currently-open batches.
    pub fn open_count(&self) -> usize {
        self.open_batches.len()
    }

    /// Return the number of completed batches awaiting consumption.
    pub fn completed_count(&self) -> usize {
        self.completed_batches.len()
    }

    /// Check if a message belongs to any open batch (has a `batch` tag
    /// matching an open ref_tag).
    pub fn message_is_batched(&self, message: &Message) -> bool {
        message
            .tags
            .as_ref()
            .and_then(|tags| {
                tags.iter()
                    .find(|t| t.key == "batch")
                    .and_then(|t| t.value.as_ref())
            })
            .map(|r| self.open_batches.contains_key(r.as_str()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustirc_protocol::message::Tag;

    fn batch_start_msg(ref_tag: &str, batch_type: &str) -> Message {
        Message::new("BATCH").with_params(vec![format!("+{ref_tag}"), batch_type.to_string()])
    }

    fn batch_end_msg(ref_tag: &str) -> Message {
        Message::new("BATCH").with_params(vec![format!("-{ref_tag}")])
    }

    fn batched_privmsg(ref_tag: &str, target: &str, text: &str) -> Message {
        Message::new("PRIVMSG")
            .with_tags(vec![Tag::from_raw("batch", Some(ref_tag))])
            .with_params(vec![target.to_string(), text.to_string()])
    }

    #[test]
    fn test_basic_batch_lifecycle() {
        let mut mgr = BatchManager::new();

        // Start a batch
        let start = batch_start_msg("abc123", "chathistory");
        let ref_tag = mgr.handle_batch_start(&start).unwrap();
        assert_eq!(ref_tag, "abc123");
        assert!(mgr.is_in_batch("abc123"));
        assert_eq!(mgr.open_count(), 1);

        // Add messages
        let msg1 = batched_privmsg("abc123", "#test", "Hello");
        let msg2 = batched_privmsg("abc123", "#test", "World");
        assert!(mgr.add_message(&msg1));
        assert!(mgr.add_message(&msg2));

        // End the batch
        let end = batch_end_msg("abc123");
        let batch = mgr.handle_batch_end(&end).unwrap();
        assert_eq!(batch.batch_type, BatchType::ChatHistory);
        assert_eq!(batch.messages.len(), 2);
        assert!(!mgr.is_in_batch("abc123"));
        assert_eq!(mgr.open_count(), 0);
        assert_eq!(mgr.completed_count(), 1);

        // Retrieve completed batch
        let completed = mgr.get_batch("abc123").unwrap();
        assert_eq!(completed.messages.len(), 2);
    }

    #[test]
    fn test_nested_batches() {
        let mut mgr = BatchManager::new();

        // Start outer batch
        let outer_start = batch_start_msg("outer", "labeled-response");
        mgr.handle_batch_start(&outer_start).unwrap();

        // Start inner batch with parent ref via batch tag
        let inner_start = Message::new("BATCH")
            .with_tags(vec![Tag::from_raw("batch", Some("outer"))])
            .with_params(vec!["+inner".to_string(), "chathistory".to_string()]);
        mgr.handle_batch_start(&inner_start).unwrap();

        assert!(mgr.is_in_batch("outer"));
        assert!(mgr.is_in_batch("inner"));
        assert_eq!(mgr.open_count(), 2);

        // The inner batch start message should also be added to the outer batch
        assert!(mgr.add_message(&inner_start));

        // Add a message to inner batch
        let msg = batched_privmsg("inner", "#ch", "nested message");
        assert!(mgr.add_message(&msg));

        // End inner batch
        let inner_end = batch_end_msg("inner");
        let inner_batch = mgr.handle_batch_end(&inner_end).unwrap();
        assert_eq!(inner_batch.parent_ref, Some("outer".to_string()));
        assert_eq!(inner_batch.messages.len(), 1);

        // End outer batch
        let outer_end = batch_end_msg("outer");
        let outer_batch = mgr.handle_batch_end(&outer_end).unwrap();
        assert_eq!(outer_batch.batch_type, BatchType::LabeledResponse);
        // Outer batch got the inner BATCH start message
        assert_eq!(outer_batch.messages.len(), 1);
    }

    #[test]
    fn test_batch_type_parsing() {
        assert_eq!(BatchType::parse("netjoin"), BatchType::Netjoin);
        assert_eq!(BatchType::parse("netsplit"), BatchType::Netsplit);
        assert_eq!(BatchType::parse("chathistory"), BatchType::ChatHistory);
        assert_eq!(
            BatchType::parse("labeled-response"),
            BatchType::LabeledResponse
        );
        assert_eq!(
            BatchType::parse("draft/multiline"),
            BatchType::Custom("draft/multiline".to_string())
        );

        // Round-trip
        assert_eq!(BatchType::Netjoin.as_str(), "netjoin");
        assert_eq!(BatchType::ChatHistory.as_str(), "chathistory");
    }

    #[test]
    fn test_error_conditions() {
        let mut mgr = BatchManager::new();

        // Missing params
        let empty = Message::new("BATCH");
        assert!(mgr.handle_batch_start(&empty).is_err());

        // Missing '+' prefix
        let bad_start = Message::new("BATCH").with_params(vec!["abc".to_string()]);
        assert!(mgr.handle_batch_start(&bad_start).is_err());

        // Missing batch type
        let no_type = Message::new("BATCH").with_params(vec!["+abc".to_string()]);
        assert!(mgr.handle_batch_start(&no_type).is_err());

        // End unknown batch
        let bad_end = batch_end_msg("nonexistent");
        assert!(mgr.handle_batch_end(&bad_end).is_err());

        // Duplicate ref_tag
        let start = batch_start_msg("dup", "netsplit");
        mgr.handle_batch_start(&start).unwrap();
        let dup = batch_start_msg("dup", "netsplit");
        assert!(mgr.handle_batch_start(&dup).is_err());

        // Message with no batch tag
        let unbatched =
            Message::new("PRIVMSG").with_params(vec!["#ch".to_string(), "hello".to_string()]);
        assert!(!mgr.add_message(&unbatched));

        // Message with unknown batch ref
        let wrong_ref = batched_privmsg("nonexistent", "#ch", "hello");
        assert!(!mgr.add_message(&wrong_ref));
    }

    #[test]
    fn test_message_is_batched() {
        let mut mgr = BatchManager::new();

        let start = batch_start_msg("ref1", "chathistory");
        mgr.handle_batch_start(&start).unwrap();

        let batched = batched_privmsg("ref1", "#ch", "in batch");
        assert!(mgr.message_is_batched(&batched));

        let not_batched = Message::new("PRIVMSG")
            .with_params(vec!["#ch".to_string(), "not in batch".to_string()]);
        assert!(!mgr.message_is_batched(&not_batched));

        let wrong_ref = batched_privmsg("unknown", "#ch", "wrong ref");
        assert!(!mgr.message_is_batched(&wrong_ref));
    }
}
