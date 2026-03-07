//! Root Dioxus application component

use crate::components::layout::MainLayout;
use crate::providers::provide_app_context;
use dioxus::prelude::*;

/// Root App component. Sets up context providers and renders the main layout.
#[component]
pub fn App() -> Element {
    let (_app_state, _theme) = provide_app_context();

    rsx! {
        MainLayout {}
    }
}

/// Launch the Dioxus GUI application (desktop mode by default).
pub fn run() -> anyhow::Result<()> {
    dioxus::launch(App);
    Ok(())
}
