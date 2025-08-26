//! Material Design 3 Components for Iced v0.13.1
//! 
//! This module provides a comprehensive set of Material Design 3 components
//! built specifically for the Iced GUI framework.

pub mod atoms;
pub mod molecules;
pub mod organisms;
pub mod canvas;

// Re-export commonly used components
pub use atoms::button::{MaterialButton, ButtonVariant};
pub use atoms::typography::{MaterialText, TypographyVariant};
pub use atoms::card::MaterialCard;
pub use atoms::chip::{MaterialChip, ChipVariant};
pub use atoms::icon::MaterialIcon;
pub use atoms::input::MaterialInput;
pub use atoms::surface::MaterialSurface;

pub use molecules::app_bar::MaterialAppBar;
pub use molecules::bottom_navigation::MaterialBottomNavigation;
pub use molecules::dialog::MaterialDialog;
pub use molecules::list_item::MaterialListItem;
pub use molecules::message_bubble::MessageBubble;
pub use molecules::search_bar::MaterialSearchBar;
// pub use molecules::snackbar::MaterialSnackbar;

// pub use organisms::chat_view::ChatView;
pub use organisms::responsive_layout::ResponsiveLayout;
pub use organisms::rich_text_editor::RichTextEditor;
pub use organisms::sidebar::ModernSidebar;

// pub use canvas::animation_canvas::AnimationCanvas;
// pub use canvas::wave_effect::WaveEffect;