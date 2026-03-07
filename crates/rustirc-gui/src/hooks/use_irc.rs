//! IRC event handler hook
//!
//! Bridges the rustirc-core EventBus to Dioxus Signal<AppState>.
//! Uses use_coroutine to listen for events and update state reactively.

use crate::state::{ActivityLevel, AppState};
use dioxus::prelude::*;
use rustirc_core::connection::ConnectionState as CoreConnectionState;
use rustirc_core::events::Event;
use tracing::{debug, info, warn};

/// Spawns a coroutine that processes IRC events and updates AppState.
///
/// Call this once from the root App component. It listens on the provided
/// tokio mpsc receiver for Event variants from the core EventBus.
pub fn use_irc_event_handler(
    mut app_state: Signal<AppState>,
    mut event_rx: Signal<Option<tokio::sync::mpsc::UnboundedReceiver<Event>>>,
) {
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        // Take ownership of the receiver
        let Some(mut rx) = event_rx.write().take() else {
            warn!("No event receiver available for IRC event handler");
            return;
        };

        info!("IRC event handler coroutine started");

        while let Some(event) = rx.recv().await {
            debug!("Processing IRC event: {:?}", event);
            process_event(&mut app_state, event);
        }

        info!("IRC event handler coroutine ended (channel closed)");
    });
}

fn process_event(app_state: &mut Signal<AppState>, event: Event) {
    match event {
        Event::Connected { connection_id } => {
            let mut state = app_state.write();
            if let Some(server) = state.servers.get_mut(&connection_id) {
                server.connection_state = CoreConnectionState::Connected;
            }
            state.add_message(
                &connection_id,
                &connection_id,
                "Connected to server",
                "System",
            );
        }

        Event::Disconnected {
            connection_id,
            reason,
        } => {
            let mut state = app_state.write();
            if let Some(server) = state.servers.get_mut(&connection_id) {
                server.connection_state = CoreConnectionState::Disconnected;
            }
            state.add_message(
                &connection_id,
                &connection_id,
                &format!("Disconnected: {reason}"),
                "System",
            );
        }

        Event::StateChanged {
            connection_id,
            state: conn_state,
        } => {
            let mut state = app_state.write();
            if let Some(server) = state.servers.get_mut(&connection_id) {
                server.connection_state = conn_state;
            }
        }

        Event::MessageReceived {
            connection_id,
            message,
        } => {
            let mut state = app_state.write();
            let target = message
                .params
                .first()
                .cloned()
                .unwrap_or_else(|| connection_id.clone());
            let sender = message
                .prefix
                .as_ref()
                .map(|p| match p {
                    rustirc_protocol::Prefix::User { nick, .. } => nick.clone(),
                    rustirc_protocol::Prefix::Server(s) => s.clone(),
                })
                .unwrap_or_else(|| "Unknown".to_string());
            let content = message.params.get(1).cloned().unwrap_or_default();

            state.add_message(&connection_id, &target, &content, &sender);

            // Update activity indicator for non-current tabs
            let tab_id = if target.starts_with('#') || target.starts_with('&') {
                format!("{connection_id}:{target}")
            } else {
                format!("server:{connection_id}")
            };

            let is_current = state.current_tab_id.as_ref() == Some(&tab_id);
            if !is_current {
                if let Some(tab) = state.tabs.get_mut(&tab_id) {
                    tab.activity = ActivityLevel::Activity;
                    tab.has_activity = true;
                }
            }
        }

        Event::MessageSent {
            connection_id,
            message,
        } => {
            let mut state = app_state.write();
            let target = message
                .params
                .first()
                .cloned()
                .unwrap_or_else(|| connection_id.clone());
            let content = message.params.get(1).cloned().unwrap_or_default();

            state.add_message(&connection_id, &target, &content, "self");
        }

        Event::ChannelJoined {
            connection_id,
            channel,
        } => {
            let mut state = app_state.write();
            state.add_channel_tab(connection_id.clone(), channel.clone());
            state.add_message(
                &connection_id,
                &channel,
                &format!("Joined {channel}"),
                "System",
            );
        }

        Event::ChannelLeft {
            connection_id,
            channel,
        } => {
            let mut state = app_state.write();
            let tab_id = format!("{connection_id}:{channel}");
            state.add_message(
                &connection_id,
                &channel,
                &format!("Left {channel}"),
                "System",
            );
            state.close_tab(&tab_id);
        }

        Event::UserJoined {
            connection_id,
            channel,
            user,
        } => {
            let mut state = app_state.write();
            state.add_user_to_channel(&connection_id, &channel, &user);
            if state.settings.show_join_part {
                state.add_message(
                    &connection_id,
                    &channel,
                    &format!("{user} has joined"),
                    "System",
                );
            }
        }

        Event::UserLeft {
            connection_id,
            channel,
            user,
        } => {
            let mut state = app_state.write();
            state.remove_user_from_channel(&connection_id, &channel, &user);
            if state.settings.show_join_part {
                state.add_message(
                    &connection_id,
                    &channel,
                    &format!("{user} has left"),
                    "System",
                );
            }
        }

        Event::NickChanged {
            connection_id,
            old,
            new,
        } => {
            let mut state = app_state.write();
            // Update nick in all channels
            if let Some(server) = state.servers.get_mut(&connection_id) {
                if server.nickname == old {
                    server.nickname = new.clone();
                }
            }
        }

        Event::TopicChanged {
            connection_id,
            channel,
            topic,
        } => {
            let mut state = app_state.write();
            if let Some(server) = state.servers.get_mut(&connection_id) {
                if let Some(chan) = server.channels.get_mut(&channel) {
                    chan.topic = Some(topic.clone());
                }
            }
            state.add_message(
                &connection_id,
                &channel,
                &format!("Topic changed to: {topic}"),
                "System",
            );
        }

        Event::Error {
            connection_id,
            error,
        } => {
            let mut state = app_state.write();
            let server_id = connection_id.as_deref().unwrap_or("global");
            state.add_message(server_id, server_id, &format!("Error: {error}"), "System");
        }

        Event::PongRequired {
            connection_id,
            server: _,
        } => {
            // Handled at the protocol level, just update last ping time
            let mut state = app_state.write();
            if let Some(server) = state.servers.get_mut(&connection_id) {
                server.last_ping = Some(std::time::SystemTime::now());
            }
        }
    }
}
