//! IRC protocol validation
//!
//! Provides comprehensive validation for IRC messages, parameters, and other
//! protocol elements to prevent injection attacks and ensure protocol compliance.

use thiserror::Error;

/// Validation error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ValidationError {
    #[error("Invalid nickname: {0}")]
    InvalidNickname(String),

    #[error("Invalid channel name: {0}")]
    InvalidChannelName(String),

    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Parameter too long: {0} characters (max: {1})")]
    ParameterTooLong(usize, usize),

    #[error("Invalid characters in parameter: {0}")]
    InvalidCharacters(String),

    #[error("Empty parameter not allowed")]
    EmptyParameter,

    #[error("Invalid tag key: {0}")]
    InvalidTagKey(String),

    #[error("Invalid tag value: {0}")]
    InvalidTagValue(String),

    #[error("Message too long: {0} bytes (max: 512)")]
    MessageTooLong(usize),
}

/// IRC protocol validator
pub struct IrcValidator {
    /// Maximum nickname length
    pub max_nickname_length: usize,
    /// Maximum channel name length  
    pub max_channel_length: usize,
    /// Maximum parameter length
    pub max_parameter_length: usize,
    /// Strict mode (RFC compliance)
    pub strict_mode: bool,
}

impl Default for IrcValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl IrcValidator {
    /// Create a new validator with default settings
    pub fn new() -> Self {
        Self {
            max_nickname_length: 30,   // Most servers use 30
            max_channel_length: 50,    // Reasonable default
            max_parameter_length: 400, // Leave room for protocol overhead
            strict_mode: false,        // Be lenient by default
        }
    }

    /// Create a strict RFC-compliant validator
    pub fn strict() -> Self {
        Self {
            max_nickname_length: 9, // RFC 1459 limit
            max_channel_length: 50,
            max_parameter_length: 400,
            strict_mode: true,
        }
    }

    /// Validate an IRC nickname
    pub fn validate_nickname(&self, nickname: &str) -> Result<(), ValidationError> {
        if nickname.is_empty() {
            return Err(ValidationError::EmptyParameter);
        }

        if nickname.len() > self.max_nickname_length {
            return Err(ValidationError::InvalidNickname(format!(
                "Too long: {} characters (max: {})",
                nickname.len(),
                self.max_nickname_length
            )));
        }

        // First character must be letter or special character
        let first_char = nickname.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic()
            && !matches!(
                first_char,
                '[' | ']' | '\\' | '`' | '_' | '^' | '{' | '|' | '}'
            )
        {
            return Err(ValidationError::InvalidNickname(
                "First character must be letter or special character".to_string(),
            ));
        }

        // Check for invalid characters
        for ch in nickname.chars() {
            if ch.is_ascii_control() || ch.is_ascii_whitespace() {
                return Err(ValidationError::InvalidNickname(
                    "Control characters and whitespace not allowed".to_string(),
                ));
            }

            if matches!(
                ch,
                '!' | '@' | '#' | '$' | '%' | '&' | '*' | '+' | ',' | '?' | '.'
            ) {
                return Err(ValidationError::InvalidNickname(format!(
                    "Invalid character: '{ch}'"
                )));
            }
        }

        Ok(())
    }

    /// Validate an IRC channel name
    pub fn validate_channel_name(&self, channel: &str) -> Result<(), ValidationError> {
        if channel.is_empty() {
            return Err(ValidationError::EmptyParameter);
        }

        if channel.len() > self.max_channel_length {
            return Err(ValidationError::InvalidChannelName(format!(
                "Too long: {} characters (max: {})",
                channel.len(),
                self.max_channel_length
            )));
        }

        // Must start with # or &
        if !channel.starts_with('#') && !channel.starts_with('&') {
            return Err(ValidationError::InvalidChannelName(
                "Must start with # or &".to_string(),
            ));
        }

        // Check for forbidden characters
        for ch in channel.chars() {
            if ch.is_ascii_control() || matches!(ch, ' ' | ',' | ':' | '\x07') {
                return Err(ValidationError::InvalidChannelName(format!(
                    "Invalid character: '{ch}'"
                )));
            }
        }

        Ok(())
    }

    /// Validate an IRC username
    pub fn validate_username(&self, username: &str) -> Result<(), ValidationError> {
        if username.is_empty() {
            return Err(ValidationError::EmptyParameter);
        }

        if username.len() > self.max_parameter_length {
            return Err(ValidationError::InvalidUsername(format!(
                "Too long: {} characters",
                username.len()
            )));
        }

        // Check for invalid characters
        for ch in username.chars() {
            if ch.is_ascii_control() || ch.is_ascii_whitespace() {
                return Err(ValidationError::InvalidUsername(
                    "Control characters and whitespace not allowed".to_string(),
                ));
            }

            if matches!(ch, '@' | '!') {
                return Err(ValidationError::InvalidUsername(format!(
                    "Invalid character: '{ch}'"
                )));
            }
        }

        Ok(())
    }

    /// Validate an IRC hostname
    pub fn validate_hostname(&self, hostname: &str) -> Result<(), ValidationError> {
        if hostname.is_empty() {
            return Err(ValidationError::EmptyParameter);
        }

        if hostname.len() > self.max_parameter_length {
            return Err(ValidationError::InvalidHostname(format!(
                "Too long: {} characters",
                hostname.len()
            )));
        }

        // Basic hostname validation - allow domains and IP addresses
        if hostname.contains(' ') || hostname.contains('\t') {
            return Err(ValidationError::InvalidHostname(
                "Whitespace not allowed".to_string(),
            ));
        }

        for ch in hostname.chars() {
            if ch.is_ascii_control() {
                return Err(ValidationError::InvalidHostname(
                    "Control characters not allowed".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate an IRC command
    pub fn validate_command(&self, command: &str) -> Result<(), ValidationError> {
        if command.is_empty() {
            return Err(ValidationError::EmptyParameter);
        }

        // Commands can be numeric (3 digits) or alphabetic
        if command.chars().all(|c| c.is_ascii_digit()) {
            if command.len() != 3 {
                return Err(ValidationError::InvalidCommand(
                    "Numeric commands must be exactly 3 digits".to_string(),
                ));
            }
        } else {
            // Alphabetic command
            if !command.chars().all(|c| c.is_ascii_alphabetic()) {
                return Err(ValidationError::InvalidCommand(
                    "Commands must be alphabetic or 3-digit numeric".to_string(),
                ));
            }

            if command.len() > 20 {
                return Err(ValidationError::InvalidCommand(
                    "Command too long".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate a general IRC parameter
    pub fn validate_parameter(&self, parameter: &str) -> Result<(), ValidationError> {
        if parameter.len() > self.max_parameter_length {
            return Err(ValidationError::ParameterTooLong(
                parameter.len(),
                self.max_parameter_length,
            ));
        }

        // Check for problematic characters
        for ch in parameter.chars() {
            if ch == '\0' || ch == '\r' || ch == '\n' {
                return Err(ValidationError::InvalidCharacters(
                    "Null, CR, and LF characters not allowed".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate an IRCv3 tag key
    pub fn validate_tag_key(&self, key: &str) -> Result<(), ValidationError> {
        if key.is_empty() {
            return Err(ValidationError::InvalidTagKey(
                "Tag key cannot be empty".to_string(),
            ));
        }

        if key.len() > 200 {
            return Err(ValidationError::InvalidTagKey(
                "Tag key too long".to_string(),
            ));
        }

        // Tag keys must match: [a-zA-Z0-9-._/]+ or vendor/key format
        for ch in key.chars() {
            if !ch.is_ascii_alphanumeric() && !matches!(ch, '-' | '.' | '_' | '/') {
                return Err(ValidationError::InvalidTagKey(format!(
                    "Invalid character in tag key: '{ch}'"
                )));
            }
        }

        Ok(())
    }

    /// Validate an IRCv3 tag value (after unescaping)
    pub fn validate_tag_value(&self, value: &str) -> Result<(), ValidationError> {
        if value.len() > 1000 {
            return Err(ValidationError::InvalidTagValue(
                "Tag value too long".to_string(),
            ));
        }

        // Tag values can contain any character except semicolon and space
        // (these should be escaped in the raw value)
        for ch in value.chars() {
            if ch == ';' || ch == ' ' {
                return Err(ValidationError::InvalidTagValue(
                    "Unescaped semicolon or space in tag value".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate complete IRC message length
    pub fn validate_message_length(&self, message: &str) -> Result<(), ValidationError> {
        if message.len() > 512 {
            return Err(ValidationError::MessageTooLong(message.len()));
        }
        Ok(())
    }

    /// Sanitize a string by removing/replacing problematic characters
    pub fn sanitize_parameter(&self, input: &str) -> String {
        input
            .chars()
            .filter(|&ch| ch != '\0' && ch != '\r' && ch != '\n')
            .take(self.max_parameter_length)
            .collect()
    }

    /// Sanitize a nickname to make it valid
    pub fn sanitize_nickname(&self, input: &str) -> String {
        let sanitized: String = input
            .chars()
            .filter(|&ch| {
                !ch.is_ascii_control()
                    && !ch.is_ascii_whitespace()
                    && !matches!(
                        ch,
                        '!' | '@' | '#' | '$' | '%' | '&' | '*' | '+' | ',' | '?' | '.'
                    )
            })
            .take(self.max_nickname_length)
            .collect();

        if sanitized.is_empty() {
            "Guest".to_string()
        } else {
            // Ensure first character is valid
            let first_char = sanitized.chars().next().unwrap();
            if first_char.is_ascii_alphabetic()
                || matches!(
                    first_char,
                    '[' | ']' | '\\' | '`' | '_' | '^' | '{' | '|' | '}'
                )
            {
                sanitized
            } else {
                format!("_{sanitized}")
            }
        }
    }
}

/// Quick validation functions for common use cases
pub fn is_valid_nickname(nickname: &str) -> bool {
    IrcValidator::new().validate_nickname(nickname).is_ok()
}

pub fn is_valid_channel(channel: &str) -> bool {
    IrcValidator::new().validate_channel_name(channel).is_ok()
}

pub fn is_valid_command(command: &str) -> bool {
    IrcValidator::new().validate_command(command).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nickname_validation() {
        let validator = IrcValidator::new();

        // Valid nicknames
        assert!(validator.validate_nickname("alice").is_ok());
        assert!(validator.validate_nickname("Bob123").is_ok());
        assert!(validator.validate_nickname("[guest]").is_ok());
        assert!(validator.validate_nickname("user_name").is_ok());

        // Invalid nicknames
        assert!(validator.validate_nickname("").is_err());
        assert!(validator.validate_nickname("user with spaces").is_err());
        assert!(validator.validate_nickname("user@host").is_err());
        assert!(validator.validate_nickname("123user").is_err());
        assert!(validator.validate_nickname("user!").is_err());

        // Too long nickname
        let long_nick = "a".repeat(50);
        assert!(validator.validate_nickname(&long_nick).is_err());
    }

    #[test]
    fn test_channel_validation() {
        let validator = IrcValidator::new();

        // Valid channels
        assert!(validator.validate_channel_name("#general").is_ok());
        assert!(validator.validate_channel_name("&local").is_ok());
        assert!(validator.validate_channel_name("#test_channel").is_ok());

        // Invalid channels
        assert!(validator.validate_channel_name("").is_err());
        assert!(validator.validate_channel_name("general").is_err()); // No #
        assert!(validator.validate_channel_name("#chan,nel").is_err()); // Comma
        assert!(validator.validate_channel_name("#chan nel").is_err()); // Space
        assert!(validator.validate_channel_name("#chan:nel").is_err()); // Colon
    }

    #[test]
    fn test_command_validation() {
        let validator = IrcValidator::new();

        // Valid commands
        assert!(validator.validate_command("PRIVMSG").is_ok());
        assert!(validator.validate_command("JOIN").is_ok());
        assert!(validator.validate_command("001").is_ok());
        assert!(validator.validate_command("404").is_ok());

        // Invalid commands
        assert!(validator.validate_command("").is_err());
        assert!(validator.validate_command("PRIV MSG").is_err()); // Space
        assert!(validator.validate_command("123").is_ok()); // Valid numeric command
        assert!(validator.validate_command("12").is_err()); // Too short
        assert!(validator.validate_command("PRIV123").is_err()); // Mixed
    }

    #[test]
    fn test_parameter_validation() {
        let validator = IrcValidator::new();

        // Valid parameters
        assert!(validator.validate_parameter("Hello world").is_ok());
        assert!(validator.validate_parameter("").is_ok()); // Empty is allowed for parameters
        assert!(validator.validate_parameter("user@host.com").is_ok());

        // Invalid parameters
        assert!(validator.validate_parameter("hello\0world").is_err()); // Null
        assert!(validator.validate_parameter("hello\rworld").is_err()); // CR
        assert!(validator.validate_parameter("hello\nworld").is_err()); // LF

        // Too long parameter
        let long_param = "a".repeat(500);
        assert!(validator.validate_parameter(&long_param).is_err());
    }

    #[test]
    fn test_tag_validation() {
        let validator = IrcValidator::new();

        // Valid tag keys
        assert!(validator.validate_tag_key("time").is_ok());
        assert!(validator.validate_tag_key("vendor/key").is_ok());
        assert!(validator.validate_tag_key("msg-id").is_ok());
        assert!(validator.validate_tag_key("example.com/key").is_ok());

        // Invalid tag keys
        assert!(validator.validate_tag_key("").is_err());
        assert!(validator.validate_tag_key("key with spaces").is_err());
        assert!(validator.validate_tag_key("key@invalid").is_err());

        // Valid tag values (unescaped)
        assert!(validator.validate_tag_value("simple-value").is_ok());
        assert!(validator
            .validate_tag_value("2021-01-01T00:00:00.000Z")
            .is_ok());

        // Invalid tag values (unescaped)
        assert!(validator
            .validate_tag_value("value; with semicolon")
            .is_err());
        assert!(validator.validate_tag_value("value with space").is_err());
    }

    #[test]
    fn test_sanitization() {
        let validator = IrcValidator::new();

        // Nickname sanitization
        assert_eq!(validator.sanitize_nickname("valid_nick"), "valid_nick");
        assert_eq!(
            validator.sanitize_nickname("nick with spaces"),
            "nickwithspaces"
        );
        assert_eq!(validator.sanitize_nickname("nick@host"), "nickhost");
        assert_eq!(validator.sanitize_nickname("123invalid"), "_123invalid");
        assert_eq!(validator.sanitize_nickname(""), "Guest");

        // Parameter sanitization
        assert_eq!(
            validator.sanitize_parameter("hello\0world\r\n"),
            "helloworld"
        );
        assert_eq!(validator.sanitize_parameter("normal text"), "normal text");
    }

    #[test]
    fn test_message_length_validation() {
        let validator = IrcValidator::new();

        let short_message = "PRIVMSG #channel :Hello";
        assert!(validator.validate_message_length(short_message).is_ok());

        let long_message = "PRIVMSG #channel :".to_string() + &"a".repeat(500);
        assert!(validator.validate_message_length(&long_message).is_err());
    }

    #[test]
    fn test_strict_mode() {
        let strict_validator = IrcValidator::strict();

        // 9-character nickname should be OK in strict mode
        assert!(strict_validator.validate_nickname("ninechars").is_ok());

        // 10-character nickname should fail in strict mode
        assert!(strict_validator.validate_nickname("tencharsss").is_err());
    }

    #[test]
    fn test_quick_validation_functions() {
        assert!(is_valid_nickname("alice"));
        assert!(!is_valid_nickname("alice@host"));

        assert!(is_valid_channel("#general"));
        assert!(!is_valid_channel("general"));

        assert!(is_valid_command("PRIVMSG"));
        assert!(!is_valid_command("PRIV MSG"));
    }
}
