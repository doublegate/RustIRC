//! Flood protection with token bucket algorithm
//!
//! Prevents the client from being disconnected due to excess flood by
//! rate-limiting outgoing messages using a token bucket. Messages that
//! cannot be sent immediately are queued for later delivery.

use std::collections::VecDeque;
use std::time::Instant;

use crate::config::FloodConfig;

/// A token-bucket based flood protector for outgoing IRC messages.
///
/// The token bucket starts full (at `max_tokens`) and drains by one token
/// per message sent. Tokens refill at `refill_rate` tokens per second.
/// Messages that cannot be sent immediately are placed in a bounded queue.
#[derive(Debug)]
pub struct FloodProtector {
    /// Current number of available tokens (fractional for smooth refill).
    tokens: f64,
    /// Maximum tokens (burst capacity).
    max_tokens: usize,
    /// Tokens added per second.
    refill_rate: f64,
    /// Timestamp of the last token refill calculation.
    last_refill: Instant,
    /// Queue of messages waiting to be sent when tokens become available.
    queue: VecDeque<String>,
    /// Maximum queue size; messages beyond this are dropped.
    max_queue_size: usize,
    /// Whether flood protection is enabled.
    enabled: bool,
}

impl FloodProtector {
    /// Create a new `FloodProtector` with explicit parameters.
    pub fn new(max_tokens: usize, refill_rate: f64, max_queue_size: usize) -> Self {
        Self {
            tokens: max_tokens as f64,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
            queue: VecDeque::new(),
            max_queue_size,
            enabled: true,
        }
    }

    /// Create a `FloodProtector` from a `FloodConfig`.
    pub fn from_config(config: &FloodConfig) -> Self {
        Self {
            tokens: config.burst_limit as f64,
            max_tokens: config.burst_limit,
            refill_rate: config.messages_per_second,
            last_refill: Instant::now(),
            queue: VecDeque::new(),
            max_queue_size: config.queue_size,
            enabled: config.enabled,
        }
    }

    /// Refill tokens based on elapsed time since the last refill.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens as f64);
            self.last_refill = now;
        }
    }

    /// Refill tokens using an externally-provided "now" instant (for testing).
    #[cfg(test)]
    fn refill_at(&mut self, now: Instant) {
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens as f64);
            self.last_refill = now;
        }
    }

    /// Check whether a message can be sent immediately.
    ///
    /// If flood protection is disabled, always returns `true`.
    /// Otherwise, refills tokens and checks if at least one is available.
    /// Consumes a token if the send is permitted.
    pub fn try_send(&mut self) -> bool {
        if !self.enabled {
            return true;
        }

        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Enqueue a message for later sending.
    ///
    /// Returns `true` if the message was enqueued, `false` if the queue
    /// is full and the message was dropped.
    pub fn enqueue(&mut self, message: String) -> bool {
        if self.queue.len() >= self.max_queue_size {
            return false;
        }
        self.queue.push_back(message);
        true
    }

    /// Calculate the next time a send will be possible.
    ///
    /// Returns `None` if a send is possible right now or flood protection
    /// is disabled. Otherwise returns the `Instant` when the next token
    /// will be available.
    pub fn next_send_time(&mut self) -> Option<Instant> {
        if !self.enabled {
            return None;
        }

        self.refill();

        if self.tokens >= 1.0 {
            None
        } else {
            let deficit = 1.0 - self.tokens;
            let wait_secs = deficit / self.refill_rate;
            Some(self.last_refill + std::time::Duration::from_secs_f64(wait_secs))
        }
    }

    /// Drain all messages from the queue that can currently be sent.
    ///
    /// Returns a `Vec` of messages that had tokens available, consuming
    /// one token per message. Remaining messages stay in the queue.
    pub fn drain_ready(&mut self) -> Vec<String> {
        if !self.enabled {
            return self.queue.drain(..).collect();
        }

        self.refill();

        let mut ready = Vec::new();
        while self.tokens >= 1.0 {
            if let Some(msg) = self.queue.pop_front() {
                self.tokens -= 1.0;
                ready.push(msg);
            } else {
                break;
            }
        }
        ready
    }

    /// Return the number of messages currently in the queue.
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Return the current (approximate) number of available tokens.
    pub fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }

    /// Check whether flood protection is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable flood protection.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_burst_sending() {
        // Allow burst of 3, refill at 1/sec
        let mut fp = FloodProtector::new(3, 1.0, 10);

        // Should allow 3 immediate sends (burst)
        assert!(fp.try_send());
        assert!(fp.try_send());
        assert!(fp.try_send());

        // Fourth should fail (no tokens left)
        assert!(!fp.try_send());
    }

    #[test]
    fn test_token_refill() {
        let mut fp = FloodProtector::new(3, 2.0, 10);

        // Drain all tokens
        assert!(fp.try_send());
        assert!(fp.try_send());
        assert!(fp.try_send());
        assert!(!fp.try_send());

        // Simulate 1 second passing (should refill 2 tokens at rate=2.0)
        let future = fp.last_refill + Duration::from_secs(1);
        fp.refill_at(future);

        // Should have ~2 tokens now
        assert!(fp.try_send());
        assert!(fp.try_send());
        assert!(!fp.try_send());
    }

    #[test]
    fn test_tokens_cap_at_max() {
        let mut fp = FloodProtector::new(3, 10.0, 10);

        // Even after a long time, tokens should not exceed max_tokens
        let future = fp.last_refill + Duration::from_secs(100);
        fp.refill_at(future);

        assert!(fp.tokens <= 3.0);
        assert!(fp.tokens >= 2.99); // Allow floating point
    }

    #[test]
    fn test_queue_management() {
        let mut fp = FloodProtector::new(2, 1.0, 3);

        // Fill the queue
        assert!(fp.enqueue("msg1".to_string()));
        assert!(fp.enqueue("msg2".to_string()));
        assert!(fp.enqueue("msg3".to_string()));

        // Queue is full (max_queue_size = 3)
        assert!(!fp.enqueue("msg4".to_string()));
        assert_eq!(fp.queue_len(), 3);

        // Drain what we can (2 tokens available)
        let ready = fp.drain_ready();
        assert_eq!(ready.len(), 2);
        assert_eq!(ready[0], "msg1");
        assert_eq!(ready[1], "msg2");
        assert_eq!(fp.queue_len(), 1);
    }

    #[test]
    fn test_drain_with_refill() {
        let mut fp = FloodProtector::new(2, 2.0, 10);

        // Drain initial tokens with try_send
        fp.try_send();
        fp.try_send();

        // Queue 5 messages
        for i in 0..5 {
            fp.enqueue(format!("msg{i}"));
        }

        // No tokens, drain should return empty
        let ready = fp.drain_ready();
        assert_eq!(ready.len(), 0);
        assert_eq!(fp.queue_len(), 5);

        // Simulate 1.5 seconds (refill 3 tokens, but capped at max_tokens=2)
        let future = fp.last_refill + Duration::from_millis(1500);
        fp.refill_at(future);

        // Now drain - should get 2 (capped at max_tokens)
        let ready = fp.drain_ready();
        assert_eq!(ready.len(), 2);
        assert_eq!(ready[0], "msg0");
        assert_eq!(ready[1], "msg1");
        assert_eq!(fp.queue_len(), 3);
    }

    #[test]
    fn test_disabled_flood_protection() {
        let mut fp = FloodProtector::new(1, 0.1, 5);
        fp.set_enabled(false);

        // Should always allow sending when disabled
        for _ in 0..100 {
            assert!(fp.try_send());
        }

        // next_send_time should return None when disabled
        assert!(fp.next_send_time().is_none());

        // drain_ready should return everything when disabled
        fp.enqueue("a".to_string());
        fp.enqueue("b".to_string());
        let ready = fp.drain_ready();
        assert_eq!(ready.len(), 2);
    }

    #[test]
    fn test_from_config() {
        let config = FloodConfig {
            enabled: true,
            messages_per_second: 3.0,
            burst_limit: 7,
            queue_size: 50,
        };

        let mut fp = FloodProtector::from_config(&config);
        assert!(fp.is_enabled());
        assert_eq!(fp.max_tokens, 7);
        assert_eq!(fp.max_queue_size, 50);

        // Should allow burst of 7
        for _ in 0..7 {
            assert!(fp.try_send());
        }
        assert!(!fp.try_send());
    }

    #[test]
    fn test_next_send_time() {
        let mut fp = FloodProtector::new(1, 2.0, 10);

        // With a full bucket, next_send_time should be None
        assert!(fp.next_send_time().is_none());

        // Consume the token
        fp.try_send();

        // Now should return a future time
        let next = fp.next_send_time();
        assert!(next.is_some());

        // The wait should be approximately 0.5s (1 token / 2 tokens_per_sec)
        let wait = next.unwrap().duration_since(fp.last_refill);
        let diff = (wait.as_secs_f64() - 0.5).abs();
        assert!(
            diff < 0.01,
            "Expected ~0.5s wait, got {:.4}s",
            wait.as_secs_f64()
        );
    }
}
