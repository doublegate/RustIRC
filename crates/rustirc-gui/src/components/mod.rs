// Modern GUI components following atomic design principles
// Atomic design: atoms → molecules → organisms → templates → pages

pub mod atoms;
pub mod canvas;
pub mod molecules; 
pub mod organisms;

// Dioxus components (new architecture)
pub mod sidebar;
pub mod tab_bar;
pub mod message_view;
pub mod input_area;
pub mod user_list;
pub mod status_bar;
pub mod dialogs;
pub mod context_menu;

// Re-export commonly used components for convenience
pub use atoms::{
    button::{MaterialButton, ButtonVariant, FloatingActionButton, FabSize},
    typography::{MaterialText, TextVariant, RichText, TextSpan},
};

pub use molecules::{
    message_bubble::{MessageBubble, ChatMessage, MessageType, UserBadge, Reaction},
};

pub use canvas::{
    animated_spinner::{AnimatedSpinner, SpinnerMessage, SpinnerType},
};

pub use organisms::{
    sidebar::{ModernSidebar, ServerInfo, ChannelInfo, ConnectionStatus, ChannelType, ActivityLevel, SidebarMessage},
    rich_text_editor::{RichTextEditor, RichTextMessage, FormatType},
    responsive_layout::{ResponsiveLayout, LayoutMessage, Breakpoint, LayoutMode},
};