//! Built-in plugins that ship with RustIRC

pub mod highlight;
pub mod logger;

pub use highlight::HighlightPlugin;
pub use logger::LoggerPlugin;
