//! IRC message formatting for TUI
//!
//! Provides IRC text formatting support for the terminal interface including:
//! - mIRC color codes parsing
//! - Text formatting (bold, italic, underline)
//! - URL detection
//! - Conversion to ratatui Spans

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
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

/// IRC color palette (mIRC colors 0-15)
pub const IRC_COLORS: [Color; 16] = [
    Color::White,            // 0: white
    Color::Black,            // 1: black
    Color::Blue,             // 2: blue
    Color::Green,            // 3: green
    Color::Red,              // 4: red
    Color::Rgb(139, 69, 19), // 5: brown
    Color::Magenta,          // 6: purple/magenta
    Color::Rgb(255, 140, 0), // 7: orange
    Color::Yellow,           // 8: yellow
    Color::LightGreen,       // 9: light green
    Color::Cyan,             // 10: cyan
    Color::LightCyan,        // 11: light cyan
    Color::LightBlue,        // 12: light blue
    Color::LightMagenta,     // 13: pink
    Color::Gray,             // 14: grey
    Color::DarkGray,         // 15: light grey
];

/// Formatted text span with styling information
#[derive(Debug, Clone, Default)]
pub struct FormattedSpan {
    pub text: String,
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub monospace: bool,
    pub reverse: bool,
    pub is_url: bool,
}

/// URL detection regex
static URL_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_url_regex() -> &'static Regex {
    URL_REGEX
        .get_or_init(|| Regex::new(r#"(?i)\b(?:https?://|www\.)[^\s<>"]*[^\s<>".,;:!?]"#).unwrap())
}

/// Parse IRC formatted text into styled spans
pub fn parse_irc_text(text: &str) -> Vec<FormattedSpan> {
    let mut spans = Vec::new();
    let mut current_span = FormattedSpan::default();
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

                // Parse color codes
                let (fg, bg) = parse_color_codes(&mut chars);
                if let Some(fg_color) = fg {
                    current_span.foreground =
                        Some(IRC_COLORS.get(fg_color).copied().unwrap_or(Color::White));
                }
                if let Some(bg_color) = bg {
                    current_span.background =
                        Some(IRC_COLORS.get(bg_color).copied().unwrap_or(Color::Black));
                }
            }
            codes::RESET => {
                if !current_span.text.is_empty() {
                    spans.push(current_span.clone());
                }
                current_span = FormattedSpan::default();
            }
            _ => {
                current_span.text.push(ch);
            }
        }
    }

    if !current_span.text.is_empty() {
        spans.push(current_span);
    }

    // Post-process for URL detection
    detect_urls(&mut spans);

    spans
}

/// Parse color codes following the IRC color format
fn parse_color_codes(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> (Option<usize>, Option<usize>) {
    let mut fg_str = String::new();
    let mut bg_str = String::new();

    // Parse foreground color (up to 2 digits)
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

    // Check for comma (background color separator)
    if let Some(&',') = chars.peek() {
        chars.next(); // consume comma

        // Parse background color (up to 2 digits)
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

    let fg = if fg_str.is_empty() {
        None
    } else {
        fg_str.parse().ok().filter(|&n| n < 16)
    };

    let bg = if bg_str.is_empty() {
        None
    } else {
        bg_str.parse().ok().filter(|&n| n < 16)
    };

    (fg, bg)
}

/// Detect URLs in text spans and mark them
fn detect_urls(spans: &mut Vec<FormattedSpan>) {
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

            // Add text before URL
            if start > last_end {
                let mut before_span = span.clone();
                before_span.text = text[last_end..start].to_string();
                new_spans.push(before_span);
            }

            // Add URL span
            let mut url_span = span.clone();
            url_span.text = url.to_string();
            url_span.is_url = true;
            url_span.foreground = Some(Color::Cyan); // Blue for links
            url_span.underline = true;
            new_spans.push(url_span);

            last_end = end;
        }

        // Add remaining text after last URL
        if last_end < text.len() {
            let mut after_span = span.clone();
            after_span.text = text[last_end..].to_string();
            new_spans.push(after_span);
        } else if regex.find(text).is_none() {
            // No URLs found, keep original span
            new_spans.push(span);
        }
    }

    *spans = new_spans;
}

/// Convert formatted spans to ratatui Line
pub fn spans_to_line(spans: &[FormattedSpan]) -> Line {
    let ratatui_spans: Vec<Span> = spans
        .iter()
        .map(|span| {
            let mut style = Style::default();

            // Apply colors
            if let Some(fg) = span.foreground {
                style = style.fg(fg);
            }
            if let Some(bg) = span.background {
                style = style.bg(bg);
            }

            // Apply reverse video by swapping foreground and background
            if span.reverse {
                let fg = style.fg.unwrap_or(Color::White);
                let bg = style.bg.unwrap_or(Color::Black);
                style = style.fg(bg).bg(fg);
            }

            // Apply text formatting
            if span.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if span.italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            if span.underline || span.is_url {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            if span.strikethrough {
                // Note: Strikethrough is not supported in all terminals
                style = style.add_modifier(Modifier::CROSSED_OUT);
            }

            Span::styled(span.text.clone(), style)
        })
        .collect();

    Line::from(ratatui_spans)
}

/// Strip all IRC formatting from text
pub fn strip_formatting(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            codes::COLOR => {
                // Skip color codes
                parse_color_codes(&mut chars);
            }
            codes::BOLD
            | codes::ITALIC
            | codes::UNDERLINE
            | codes::STRIKETHROUGH
            | codes::MONOSPACE
            | codes::REVERSE
            | codes::RESET => {
                // Skip formatting codes
            }
            _ => {
                result.push(ch);
            }
        }
    }

    result
}

/// Get plain text from formatted spans
pub fn spans_to_plain_text(spans: &[FormattedSpan]) -> String {
    spans
        .iter()
        .map(|span| span.text.as_str())
        .collect::<Vec<_>>()
        .join("")
}

/// Emoji replacement map for common emoticons
pub fn replace_emoticons(text: &str) -> String {
    text.replace(":)", "😊")
        .replace(":(", "😢")
        .replace(":D", "😃")
        .replace(":P", "😛")
        .replace(";)", "😉")
        .replace(":o", "😮")
        .replace(":|", "😐")
        .replace("<3", "❤️")
        .replace("</3", "💔")
        .replace(":thumbsup:", "👍")
        .replace(":thumbsdown:", "👎")
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
        assert_eq!(spans[1].foreground, Some(IRC_COLORS[4])); // Red
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
        assert_eq!(strip_formatting(text), "Hello bold world");
    }
}
