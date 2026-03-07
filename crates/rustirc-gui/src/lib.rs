//! RustIRC GUI - Dioxus-based graphical interface

pub mod app;
pub mod bridge;
pub mod components;
pub mod formatting;
pub mod hooks;
pub mod providers;
pub mod state;

use rustirc_core::client::IrcClient;
use std::sync::{Arc, OnceLock};

/// Global IrcClient instance shared across the GUI.
static IRC_CLIENT: OnceLock<Arc<IrcClient>> = OnceLock::new();

/// Get the global IrcClient instance.
pub(crate) fn irc_client() -> Arc<IrcClient> {
    IRC_CLIENT
        .get()
        .cloned()
        .expect("IRC client not initialized")
}

/// Launch the Dioxus GUI application with config.
pub fn run_gui_with_config(config: rustirc_core::Config) -> anyhow::Result<()> {
    // Create the IrcClient before launching Dioxus
    let client = Arc::new(IrcClient::new(config));
    IRC_CLIENT.set(client).ok();

    // Install rustls crypto provider - required for TLS connections.
    // Both ring and aws-lc-rs are in the dependency tree, so rustls can't
    // auto-detect which to use. We explicitly install ring.
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Create a multi-threaded tokio runtime so that tokio::spawn() works
    // within Dioxus spawn tasks. The core IrcClient uses tokio::spawn internally
    // for connection management, and Dioxus's own async executor doesn't provide
    // a tokio runtime context.
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    dioxus::launch(app::App);
    Ok(())
}

/// Launch the Dioxus GUI application with default config (fallback).
pub fn run_gui() -> anyhow::Result<()> {
    run_gui_with_config(rustirc_core::Config::load_or_default())
}
