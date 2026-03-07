//! IRC action hooks for sending commands to the server
//!
//! Provides action functions that components call to interact with IRC.
//! Network operations use the global IrcClient via spawn() for async execution.

use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_core::spawn_forever;
use rustirc_core::connection::ConnectionConfig;
use tracing::info;

/// IRC action methods available to components.
/// Signal is Copy, keeping this struct Copy for easy use in closures.
#[derive(Clone, Copy)]
pub struct IrcActions {
    app_state: Signal<AppState>,
}

impl IrcActions {
    pub fn new(app_state: Signal<AppState>) -> Self {
        Self { app_state }
    }

    pub fn connect(
        &self,
        server_id: &str,
        address: &str,
        port: u16,
        nickname: &str,
        use_tls: bool,
        auto_join_channels: Vec<String>,
    ) {
        info!(
            "Connecting to {}:{} (tls={}, nick={})",
            address, port, use_tls, nickname
        );

        // Update local state immediately
        let mut state = self.app_state;
        {
            let mut s = state.write();
            s.add_server(server_id.to_string(), address.to_string());
            // Store channels to auto-join once connected
            s.pending_auto_joins
                .insert(server_id.to_string(), auto_join_channels);
        }

        // Build connection config with user-specified settings
        let connection_config = ConnectionConfig {
            server: address.to_string(),
            port,
            use_tls,
            verify_tls: true,
            nickname: nickname.to_string(),
            username: nickname.to_lowercase(),
            realname: format!("{nickname} - RustIRC Client"),
            ..Default::default()
        };

        // Use spawn_forever so the task survives component unmounting
        // (e.g., ConnectDialog closing immediately after clicking Connect)
        let client = crate::irc_client();
        spawn_forever(async move {
            match client.connect_with_config(connection_config).await {
                Ok(conn_id) => {
                    info!("Connection initiated: {}", conn_id);
                }
                Err(e) => {
                    tracing::error!("Connection failed: {}", e);
                }
            }
        });
    }

    pub fn disconnect(&self, server_id: &str) {
        info!("Disconnecting from {}", server_id);
        let mut state = self.app_state;
        state.write().remove_server(server_id);

        let client = crate::irc_client();
        spawn_forever(async move {
            if let Err(e) = client.disconnect().await {
                tracing::error!("Disconnect error: {}", e);
            }
        });
    }

    pub fn send_message(&self, _server_id: &str, target: &str, message: &str) {
        let target = target.to_string();
        let message = message.to_string();

        let client = crate::irc_client();
        spawn_forever(async move {
            if let Err(e) = client.send_message(&target, &message).await {
                tracing::error!("Send message error: {}", e);
            }
        });
    }

    pub fn join_channel(&self, server_id: &str, channel: &str) {
        info!("Joining {} on {}", channel, server_id);

        let channel = channel.to_string();
        let client = crate::irc_client();
        spawn_forever(async move {
            if let Err(e) = client.join_channel(&channel).await {
                tracing::error!("Join channel error: {}", e);
            }
        });
    }

    pub fn leave_channel(&self, server_id: &str, channel: &str) {
        info!("Leaving {} on {}", channel, server_id);
        let mut state = self.app_state;
        let tab_id = format!("{server_id}:{channel}");
        state.write().close_tab(&tab_id);

        let channel = channel.to_string();
        let client = crate::irc_client();
        spawn_forever(async move {
            if let Err(e) = client
                .send_command(rustirc_protocol::Command::Part {
                    channels: vec![channel],
                    message: None,
                })
                .await
            {
                tracing::error!("Leave channel error: {}", e);
            }
        });
    }

    pub fn switch_tab(&self, tab_id: &str) {
        let mut state = self.app_state;
        let mut s = state.write();
        s.switch_to_tab(tab_id);
        if let Some(tab) = s.tabs.get_mut(tab_id) {
            tab.mark_as_read();
        }
    }

    pub fn close_tab(&self, tab_id: &str) {
        let mut state = self.app_state;
        state.write().close_tab(tab_id);
    }
}
