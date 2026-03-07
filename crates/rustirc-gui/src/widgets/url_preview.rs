//! URL detection and display utilities for RustIRC GUI
//!
//! Detects URLs in IRC message text using compiled regex patterns and
//! provides functionality to open them in the user's default browser.

use regex::Regex;
use std::sync::OnceLock;

/// Get the compiled URL regex (lazily initialized)
fn url_regex() -> &'static Regex {
    static URL_REGEX: OnceLock<Regex> = OnceLock::new();
    URL_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(https?://[^\s<>\[\](){}]+|www\.[^\s<>\[\](){}]+\.[^\s<>\[\](){}]+)")
            .expect("URL regex should compile")
    })
}

/// A URL detected within a text string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedUrl {
    /// The raw URL string as found in the text
    pub url: String,
    /// Byte offset of the URL start within the source text
    pub start: usize,
    /// Byte offset of the URL end within the source text
    pub end: usize,
    /// A shortened display string for the URL (truncated if very long)
    pub display_text: String,
}

/// Maximum display length before truncation with ellipsis.
const MAX_DISPLAY_LEN: usize = 60;

/// URL detection engine with compiled regex for efficient repeated matching.
#[derive(Debug)]
pub struct UrlDetector;

impl UrlDetector {
    /// Create a new `UrlDetector`.
    pub fn new() -> Self {
        // Force lazy initialization of the regex on first use
        let _ = url_regex();
        Self
    }

    /// Detect all URLs in the given text.
    ///
    /// Returns a `Vec<DetectedUrl>` with byte offsets into the original text,
    /// the raw URL string, and a display-friendly version (truncated if needed).
    pub fn detect_urls(&self, text: &str) -> Vec<DetectedUrl> {
        url_regex()
            .find_iter(text)
            .map(|m| {
                let url = m.as_str().to_string();
                let display_text = if url.len() > MAX_DISPLAY_LEN {
                    format!("{}...", &url[..MAX_DISPLAY_LEN])
                } else {
                    url.clone()
                };

                DetectedUrl {
                    url,
                    start: m.start(),
                    end: m.end(),
                    display_text,
                }
            })
            .collect()
    }

    /// Open a URL in the user's default browser.
    ///
    /// Uses the `open` crate for cross-platform compatibility.
    /// Returns an error if the URL could not be opened.
    pub fn open_url(url: &str) -> Result<(), String> {
        // Prepend https:// for www. URLs without a scheme
        let full_url = if url.starts_with("www.") {
            format!("https://{url}")
        } else {
            url.to_string()
        };

        open::that(&full_url).map_err(|e| format!("Failed to open URL '{full_url}': {e}"))
    }
}

impl Default for UrlDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_https_url() {
        let detector = UrlDetector::new();
        let urls = detector.detect_urls("Check out https://example.com/page for info");

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "https://example.com/page");
        assert_eq!(urls[0].start, 10);
        assert_eq!(urls[0].end, 34);
    }

    #[test]
    fn test_detect_http_url() {
        let detector = UrlDetector::new();
        let urls = detector.detect_urls("Visit http://example.org today");

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "http://example.org");
    }

    #[test]
    fn test_detect_www_url() {
        let detector = UrlDetector::new();
        let urls = detector.detect_urls("Go to www.example.com for details");

        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0].url, "www.example.com");
    }

    #[test]
    fn test_detect_multiple_urls() {
        let detector = UrlDetector::new();
        let text = "See https://one.com and http://two.com and www.three.org/path";
        let urls = detector.detect_urls(text);

        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0].url, "https://one.com");
        assert_eq!(urls[1].url, "http://two.com");
        assert_eq!(urls[2].url, "www.three.org/path");
    }

    #[test]
    fn test_no_urls_in_plain_text() {
        let detector = UrlDetector::new();
        let urls = detector.detect_urls("This is just plain text with no links");

        assert!(urls.is_empty());
    }

    #[test]
    fn test_url_with_query_and_fragment() {
        let detector = UrlDetector::new();
        let urls =
            detector.detect_urls("Link: https://example.com/search?q=rust&page=2#results here");

        assert_eq!(urls.len(), 1);
        assert_eq!(
            urls[0].url,
            "https://example.com/search?q=rust&page=2#results"
        );
    }

    #[test]
    fn test_long_url_display_truncation() {
        let detector = UrlDetector::new();
        let long_url = format!("https://example.com/{}", "a".repeat(100));
        let text = format!("See {long_url} for details");
        let urls = detector.detect_urls(&text);

        assert_eq!(urls.len(), 1);
        assert!(urls[0].display_text.ends_with("..."));
        // display_text should be MAX_DISPLAY_LEN + "..."
        assert_eq!(urls[0].display_text.len(), MAX_DISPLAY_LEN + 3);
    }
}
