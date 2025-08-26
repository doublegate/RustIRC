//! IRC connection and state management provider

use crate::context::{ConnectionInfo, IrcState, MessageType};
use dioxus::prelude::*;
use rustirc_core::{ClientManager, Config, IrcClient};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// IRC provider for managing connections and client state
#[component]
pub fn IrcProvider(children: Element) -> Element {
    let client_manager = use_signal(|| Arc::new(ClientManager::new()));

    // Initialize IRC state context
    use_context_provider(|| IrcState::default());

    let irc_state = use_context::<IrcState>();

    // Set up IRC event handling
    use_effect(move || {
        let client_manager = client_manager.clone();
        let irc_state = irc_state.clone();

        spawn(async move {
            // Set up IRC event loop
            setup_irc_event_loop(client_manager(), irc_state).await;
        });
    });

    rsx! { {children} }
}

/// Set up the main IRC event processing loop
async fn setup_irc_event_loop(client_manager: Arc<ClientManager>, irc_state: IrcState) {
    // This would typically set up event listeners for IRC events
    // and update the Dioxus state accordingly

    // TODO: Implement actual IRC client integration
    // For now, this is a placeholder for the event loop structure

    tracing::info!("IRC event loop initialized");
}

/// Connect to an IRC server
pub async fn connect_to_server(
    server: String,
    port: u16,
    nickname: String,
    password: Option<String>,
    use_tls: bool,
    irc_state: &IrcState,
    client_manager: Arc<ClientManager>,
) -> Result<String, String> {
    let connection_id = Uuid::new_v4().to_string();

    // Create IRC client configuration
    let mut config = Config::new();
    config.server = Some(server.clone());
    config.port = Some(port);
    config.nick = Some(nickname.clone());
    config.password = password;
    config.use_tls = Some(use_tls);

    // Create and connect the IRC client
    match client_manager
        .create_client(connection_id.clone(), config)
        .await
    {
        Ok(client) => {
            // Update Dioxus state
            irc_state.connect_server(server, port, nickname);

            // TODO: Start connection process in background
            spawn_irc_connection_task(client, connection_id.clone(), irc_state.clone());

            Ok(connection_id)
        }
        Err(e) => Err(format!("Failed to create IRC client: {}", e)),
    }
}

/// Background task for handling IRC connection
fn spawn_irc_connection_task(client: Arc<IrcClient>, connection_id: String, irc_state: IrcState) {
    spawn(async move {
        // Simulate connection process
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Update connection state
        if let Some(mut connection) = irc_state.connections.write().get_mut(&connection_id) {
            connection.state = rustirc_core::ConnectionState::Connected;
            connection.client = Some(client.as_ref().clone());
        }

        // Add system message about successful connection
        irc_state.add_message(
            connection_id.clone(),
            connection_id.clone(), // Server-level message
            None,
            "Connected successfully!".to_string(),
            MessageType::System,
        );

        // TODO: Set up message event listener
        // This would listen for actual IRC messages and update the UI
    });
}

/// Join an IRC channel
pub async fn join_channel(
    server_id: String,
    channel: String,
    irc_state: &IrcState,
    client_manager: Arc<ClientManager>,
) -> Result<(), String> {
    if let Some(client) = client_manager.get_client(&server_id).await {
        // TODO: Send JOIN command to IRC server
        // For now, just update local state
        irc_state.join_channel(channel.clone());

        // Add system message
        irc_state.add_message(
            server_id.clone(),
            channel.clone(),
            None,
            format!("Joining {}...", channel),
            MessageType::System,
        );

        // Simulate join process
        spawn({
            let irc_state = irc_state.clone();
            let server_id = server_id.clone();
            let channel = channel.clone();

            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                // Mark channel as joined
                if let Some(connection) = irc_state.connections.write().get_mut(&server_id) {
                    if let Some(ch) = connection.channels.get_mut(&channel) {
                        ch.joined = true;
                    }
                }

                // Add join confirmation message
                irc_state.add_message(
                    server_id,
                    channel.clone(),
                    None,
                    format!("You have joined {}", channel),
                    MessageType::Join,
                );
            }
        });

        Ok(())
    } else {
        Err("IRC client not found".to_string())
    }
}

/// Send a message to a channel or user
pub async fn send_message(
    server_id: String,
    target: String,
    message: String,
    irc_state: &IrcState,
    client_manager: Arc<ClientManager>,
) -> Result<(), String> {
    if let Some(_client) = client_manager.get_client(&server_id).await {
        // TODO: Send PRIVMSG command to IRC server

        // For now, just add to local state (would normally be echoed back from server)
        irc_state.add_message(
            server_id,
            target,
            Some("YourNick".to_string()), // TODO: Get actual nickname from client
            message,
            MessageType::Normal,
        );

        Ok(())
    } else {
        Err("IRC client not found".to_string())
    }
}

/// Disconnect from an IRC server
pub async fn disconnect_from_server(
    server_id: String,
    irc_state: &IrcState,
    client_manager: Arc<ClientManager>,
) -> Result<(), String> {
    if let Some(_client) = client_manager.remove_client(&server_id).await {
        // Update connection state
        if let Some(mut connection) = irc_state.connections.write().get_mut(&server_id) {
            connection.state = rustirc_core::ConnectionState::Disconnected;
        }

        // Add system message
        irc_state.add_message(
            server_id.clone(),
            server_id.clone(),
            None,
            "Disconnected from server".to_string(),
            MessageType::System,
        );

        Ok(())
    } else {
        Err("IRC client not found".to_string())
    }
}
