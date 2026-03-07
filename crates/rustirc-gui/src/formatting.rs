//! IRC message formatting and rendering
//!
//! Provides comprehensive support for IRC text formatting including:
//! - mIRC color codes (^C)
//! - Text formatting (bold, italic, underline, strikethrough)
//! - URL detection and linking
//!
//! Output: HTML-compatible styled spans for Dioxus RSX rendering.

use regex::Regex;
use std::sync::OnceLock;

/// IRC formatting codes
pub mod codes {
    pub const BOLD: char = '\x02';
    pub const ITALIC: char = '\x1D';
    pub const UNDERLINE: char = '\x1F';
    pub const STRIKETHROUGH: char = '\x1E';
    pub const MONOSPACE: char = '\x11';
    pub const REVERSE: char = '\x16';
    pub const COLOR: char = '\x03';
    pub const RESET: char = '\x0F';
}

/// IRC color palette (mIRC colors 0-15) as CSS color strings
pub const IRC_COLORS: [&str; 16] = [
    "#ffffff", // 0: white
    "#000000", // 1: black
    "#00007f", // 2: blue
    "#009300", // 3: green
    "#ff0000", // 4: red
    "#7f0000", // 5: brown
    "#9c009c", // 6: purple
    "#fc7f00", // 7: orange
    "#ffff00", // 8: yellow
    "#00fc00", // 9: light green
    "#009393", // 10: cyan
    "#00ffff", // 11: light cyan
    "#0000fc", // 12: light blue
    "#ff00ff", // 13: pink
    "#7f7f7f", // 14: grey
    "#d2d2d2", // 15: light grey
];

/// Formatted text span with styling information
#[derive(Debug, Clone, Default)]
pub struct TextSpan {
    pub text: String,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub monospace: bool,
    pub reverse: bool,
    pub is_url: bool,
    pub url_target: Option<String>,
}

impl TextSpan {
    /// Generate inline CSS style string for this span
    pub fn to_css_style(&self) -> String {
        let mut styles = Vec::new();

        if let Some(ref fg) = self.foreground {
            styles.push(format!("color:{fg}"));
        }
        if let Some(ref bg) = self.background {
            styles.push(format!("background-color:{bg}"));
        }
        if self.bold {
            styles.push("font-weight:bold".to_string());
        }
        if self.italic {
            styles.push("font-style:italic".to_string());
        }

        let mut decorations = Vec::new();
        if self.underline {
            decorations.push("underline");
        }
        if self.strikethrough {
            decorations.push("line-through");
        }
        if !decorations.is_empty() {
            styles.push(format!("text-decoration:{}", decorations.join(" ")));
        }

        if self.monospace {
            styles.push("font-family:monospace".to_string());
        }

        styles.join(";")
    }

    /// Generate CSS class names for this span
    pub fn to_css_classes(&self) -> String {
        let mut classes = Vec::new();
        if self.bold {
            classes.push("font-bold");
        }
        if self.italic {
            classes.push("italic");
        }
        if self.underline {
            classes.push("underline");
        }
        if self.strikethrough {
            classes.push("line-through");
        }
        if self.monospace {
            classes.push("font-mono");
        }
        if self.is_url {
            classes.push("cursor-pointer hover:opacity-80");
        }
        classes.join(" ")
    }
}

/// URL detection regex
static URL_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_url_regex() -> &'static Regex {
    URL_REGEX
        .get_or_init(|| Regex::new(r#"(?i)\b(?:https?://|www\.)[^\s<>"]*[^\s<>".,;:!?]"#).unwrap())
}

/// Parse IRC formatted text into styled spans
pub fn parse_irc_text(text: &str) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    let mut current_span = TextSpan::default();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            codes::BOLD => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                    current_span.text.clear();
                }
                current_span.bold = !current_span.bold;
            }
            codes::ITALIC => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                    current_span.text.clear();
                }
                current_span.italic = !current_span.italic;
            }
            codes::UNDERLINE => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                    current_span.text.clear();
                }
                current_span.underline = !current_span.underline;
            }
            codes::STRIKETHROUGH => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                    current_span.text.clear();
                }
                current_span.strikethrough = !current_span.strikethrough;
            }
            codes::MONOSPACE => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                    current_span.text.clear();
                }
                current_span.monospace = !current_span.monospace;
            }
            codes::REVERSE => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                    current_span.text.clear();
                }
                current_span.reverse = !current_span.reverse;
            }
            codes::COLOR => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                    current_span.text.clear();
                }

                let (fg, bg) = parse_color_codes(&mut chars);
                if let Some(fg_idx) = fg {
                    current_span.foreground = IRC_COLORS.get(fg_idx).map(|c| c.to_string());
                }
                if let Some(bg_idx) = bg {
                    current_span.background = IRC_COLORS.get(bg_idx).map(|c| c.to_string());
                }
            }
            codes::RESET => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                }
                current_span = TextSpan::default();
            }
            _ => {
                current_span.text.push(ch);
            }
        }
    }

    if !current_span.text.is_empty() {
        spans.push(current_span);
    }

    detect_urls(&mut spans);

    spans
}

/// Parse color codes following the IRC color format
fn parse_color_codes(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> (Option<usize>, Option<usize>) {
    let mut fg_str = String::new();
    let mut bg_str = String::new();
    let mut parsing_bg = false;

    for _ in 0..2 {
        if let Some(&ch) = chars.peek() {
            if ch.is_ascii_digit() {
                fg_str.push(chars.next().unwrap());
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if let Some(&',') = chars.peek() {
        chars.next();
        parsing_bg = true;

        if parsing_bg {
            for _ in 0..2 {
                if let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        bg_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
    }

    let fg = if fg_str.is_empty() {
        None
    } else {
        fg_str.parse().ok().filter(|&n: &usize| n < 16)
    };

    let bg = if bg_str.is_empty() || !parsing_bg {
        None
    } else {
        bg_str.parse().ok().filter(|&n: &usize| n < 16)
    };

    (fg, bg)
}

/// Detect URLs in text spans and mark them
fn detect_urls(spans: &mut Vec<TextSpan>) {
    let regex = get_url_regex();
    let mut new_spans = Vec::new();

    for span in spans.drain(..) {
        if span.is_url {
            new_spans.push(span);
            continue;
        }

        let text = &span.text;
        let mut last_end = 0;

        for url_match in regex.find_iter(text) {
            let start = url_match.start();
            let end = url_match.end();
            let url = url_match.as_str();

            if start > last_end {
                let mut before_span = span.clone();
                before_span.text = text[last_end..start].to_string();
                new_spans.push(before_span);
            }

            let mut url_span = span.clone();
            url_span.text = url.to_string();
            url_span.is_url = true;
            url_span.url_target = Some(if url.starts_with("http") {
                url.to_string()
            } else {
                format!("http://{url}")
            });
            url_span.foreground = Some("#0088ff".to_string());
            url_span.underline = true;
            new_spans.push(url_span);

            last_end = end;
        }

        if last_end < text.len() {
            let mut after_span = span.clone();
            after_span.text = text[last_end..].to_string();
            new_spans.push(after_span);
        } else if regex.find(text).is_none() {
            new_spans.push(span);
        }
    }

    *spans = new_spans;
}

/// Strip all IRC formatting from text
pub fn strip_formatting(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            codes::COLOR => {
                parse_color_codes(&mut chars);
            }
            codes::BOLD
            | codes::ITALIC
            | codes::UNDERLINE
            | codes::STRIKETHROUGH
            | codes::MONOSPACE
            | codes::REVERSE
            | codes::RESET => {}
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

/// Get plain text from formatted spans
pub fn spans_to_plain_text(spans: &[TextSpan]) -> String {
    spans.iter().map(|span| span.text.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bold_text() {
        let spans = parse_irc_text("Hello \x02bold\x02 world");
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].text, "Hello ");
        assert!(!spans[0].bold);
        assert_eq!(spans[1].text, "bold");
        assert!(spans[1].bold);
        assert_eq!(spans[2].text, " world");
        assert!(!spans[2].bold);
    }

    #[test]
    fn test_parse_color_codes() {
        let spans = parse_irc_text("Hello \x0304red\x03 world");
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[1].text, "red");
        assert_eq!(spans[1].foreground, Some(IRC_COLORS[4].to_string()));
    }

    #[test]
    fn test_url_detection() {
        let spans = parse_irc_text("Visit https://example.com for more info");
        let url_spans: Vec<_> = spans.iter().filter(|s| s.is_url).collect();
        assert_eq!(url_spans.len(), 1);
        assert_eq!(url_spans[0].text, "https://example.com");
    }

    #[test]
    fn test_strip_formatting() {
        let text = "Hello \x02bold\x0304red\x03 world";
        assert_eq!(strip_formatting(text), "Hello boldred world");
    }

    #[test]
    fn test_span_css_style() {
        let span = TextSpan {
            text: "test".to_string(),
            bold: true,
            foreground: Some("#ff0000".to_string()),
            underline: true,
            ..Default::default()
        };
        let style = span.to_css_style();
        assert!(style.contains("color:#ff0000"));
        assert!(style.contains("font-weight:bold"));
        assert!(style.contains("text-decoration:underline"));
    }
}
