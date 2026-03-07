//! Root Dioxus application component

use crate::components::layout::MainLayout;
use crate::providers::provide_app_context;
use dioxus::prelude::*;

// Embed CSS at compile time for cargo run compatibility
const MAIN_CSS: &str = include_str!("../../../assets/main.css");
const THEME_CSS: &str = include_str!("../../../assets/themes/base.css");
const TAILWIND_CSS: &str = include_str!("../../../assets/tailwind.css");

/// Root App component. Sets up context providers and renders the main layout.
#[component]
pub fn App() -> Element {
    let (_app_state, _theme) = provide_app_context();

    rsx! {
        document::Style { {THEME_CSS} }
        document::Style { {MAIN_CSS} }
        document::Style { {TAILWIND_CSS} }
        MainLayout {}
    }
}

/// Launch the Dioxus GUI application (desktop mode by default).
pub fn run() -> anyhow::Result<()> {
    dioxus::launch(App);
    Ok(())
}
