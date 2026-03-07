//! IRC action hooks for sending commands to the server
//!
//! Provides action functions that components call to interact with IRC.
//! These call rustirc-core methods directly.

use crate::state::AppState;
use dioxus::prelude::*;
use tracing::info;

/// IRC action methods available to components.
/// Signal is Copy, so clone/copy is cheap - we take &mut self for write access.
#[derive(Clone, Copy)]
pub struct IrcActions {
    app_state: Signal<AppState>,
}

impl IrcActions {
    pub fn new(app_state: Signal<AppState>) -> Self {
        Self { app_state }
    }

    pub fn connect(&self, server_id: &str, address: &str) {
        info!("Connecting to {} ({})", server_id, address);
        let mut state = self.app_state;
        state
            .write()
            .add_server(server_id.to_string(), address.to_string());
    }

    pub fn disconnect(&self, server_id: &str) {
        info!("Disconnecting from {}", server_id);
        let mut state = self.app_state;
        state.write().remove_server(server_id);
    }

    pub fn send_message(&self, server_id: &str, target: &str, message: &str) {
        let mut state = self.app_state;
        state
            .write()
            .add_message(server_id, target, message, "self");
    }

    pub fn join_channel(&self, server_id: &str, channel: &str) {
        info!("Joining {} on {}", channel, server_id);
        let mut state = self.app_state;
        state
            .write()
            .add_channel_tab(server_id.to_string(), channel.to_string());
    }

    pub fn leave_channel(&self, server_id: &str, channel: &str) {
        info!("Leaving {} on {}", channel, server_id);
        let mut state = self.app_state;
        let tab_id = format!("{server_id}:{channel}");
        state.write().close_tab(&tab_id);
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
