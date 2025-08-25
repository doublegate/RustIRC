//! Provider modules for React-like context management
//!
//! This module organizes context providers for better separation of concerns:
//! - IRC connection and state management
//! - Theme management and CSS injection
//! - UI state and layout preferences
//! - Keyboard shortcut handling

pub mod irc_provider;
pub mod theme_provider;
pub mod ui_provider;
pub mod keyboard_provider;

pub use irc_provider::IrcProvider;
pub use theme_provider::ThemeProvider;
pub use ui_provider::UiProvider;
pub use keyboard_provider::KeyboardProvider;