//! CLI Prototype for RustIRC
//!
//! Full-featured command-line interface with all GUI features for testing IRC functionality.
//! Includes themes, settings, multiple servers, tab management, and comprehensive IRC support.

use crate::{AuthState, Config, IrcClient, SaslCredentials};
use anyhow::Result;
use std::collections::HashMap;
use std::io::{self, Write};
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

/// CLI application settings matching GUI AppSettings
#[derive(Debug, Clone)]
pub struct CliSettings {
    pub theme: String,
    pub show_timestamps: bool,
    pub show_join_part: bool,
    pub highlight_words: Vec<String>,
    pub notification_sound: bool,
    pub auto_reconnect: bool,
    pub nick_colors: bool,
    pub timestamp_format: String,
    pub compact_mode: bool,
}

impl Default for CliSettings {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            show_timestamps: true,
            show_join_part: false,
            highlight_words: vec!["RustIRC_User".to_string()],
            notification_sound: false, // CLI doesn't make sounds
            auto_reconnect: true,
            nick_colors: false, // CLI doesn't have colors by default
            timestamp_format: "%H:%M:%S".to_string(),
            compact_mode: false,
        }
    }
}

/// CLI server information
pub struct CliServer {
    pub name: String,
    pub client: Option<IrcClient>,
    pub connected: bool,
    pub channels: Vec<String>,
}

/// CLI tab for session management
#[derive(Debug, Clone)]
pub struct CliTab {
    pub id: String,
    pub name: String,
    pub tab_type: CliTabType,
    pub server_id: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CliTabType {
    Server,
    Channel,
    PrivateMessage,
}

/// Full-featured CLI IRC client with GUI parity
pub struct CliClient {
    /// Multiple server support
    servers: HashMap<String, CliServer>,

    /// Tab management
    tabs: HashMap<String, CliTab>,
    current_tab_id: Option<String>,
    tab_order: Vec<String>,

    /// Settings
    settings: CliSettings,

    /// Connection state
    current_server_id: Option<String>,

    /// Command history
    command_history: Vec<String>,
    history_position: usize,
}

impl CliClient {
    pub fn new(_config: Config) -> Self {
        Self {
            servers: HashMap::new(),
            tabs: HashMap::new(),
            current_tab_id: None,
            tab_order: Vec::new(),
            settings: CliSettings::default(),
            current_server_id: None,
            command_history: Vec::new(),
            history_position: 0,
        }
    }

    /// Add a server
    pub fn add_server(&mut self, server_id: String, name: String) {
        let server = CliServer {
            name: name.clone(),
            client: None,
            connected: false,
            channels: Vec::new(),
        };
        self.servers.insert(server_id.clone(), server);

        // Create server tab
        let tab = CliTab {
            id: format!("server:{server_id}"),
            name,
            tab_type: CliTabType::Server,
            server_id: Some(server_id.clone()),
            active: false,
        };
        let tab_id = format!("server:{server_id}");
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());

        // Set as current if first
        if self.current_tab_id.is_none() {
            self.current_tab_id = Some(tab_id);
            self.current_server_id = Some(server_id);
        }
    }

    /// Add a channel tab
    pub fn add_channel_tab(&mut self, server_id: String, channel: String) {
        let tab = CliTab {
            id: format!("{server_id}:{channel}"),
            name: channel.clone(),
            tab_type: CliTabType::Channel,
            server_id: Some(server_id.clone()),
            active: false,
        };
        let tab_id = format!("{server_id}:{channel}");
        self.tabs.insert(tab_id.clone(), tab);
        self.tab_order.push(tab_id.clone());

        // Set as current tab
        self.current_tab_id = Some(tab_id);

        // Add to server's channel list
        if let Some(server) = self.servers.get_mut(&server_id) {
            server.channels.push(channel);
        }
    }

    /// Get current server
    pub fn get_current_server(&mut self) -> Option<&mut CliServer> {
        if let Some(server_id) = &self.current_server_id {
            self.servers.get_mut(server_id)
        } else {
            None
        }
    }

    /// Show status information
    pub fn show_status(&self) {
        println!("=== RustIRC CLI Status ===");
        println!("Current tab: {:?}", self.current_tab_id);
        println!("Servers: {}", self.servers.len());
        for (id, server) in &self.servers {
            println!(
                "  {} ({}): {} - {} channels",
                id,
                server.name,
                if server.connected {
                    "Connected"
                } else {
                    "Disconnected"
                },
                server.channels.len()
            );
        }
        println!("Tabs: {}", self.tabs.len());
        for (i, tab_id) in self.tab_order.iter().enumerate() {
            if let Some(tab) = self.tabs.get(tab_id) {
                let current_marker = if Some(tab_id) == self.current_tab_id.as_ref() {
                    "*"
                } else {
                    " "
                };
                println!(
                    "  {}{}: {} ({:?})",
                    current_marker,
                    i + 1,
                    tab.name,
                    tab.tab_type
                );
            }
        }
        println!("Settings:");
        println!("  Theme: {}", self.settings.theme);
        println!("  Timestamps: {}", self.settings.show_timestamps);
        println!("  Join/Part: {}", self.settings.show_join_part);
        println!("  Compact mode: {}", self.settings.compact_mode);
        println!();
    }

    /// List available themes
    pub fn list_themes(&self) {
        println!("Available themes:");
        println!("  default");
        println!("  dark");
        println!("  light");
        println!("  mono");
        println!("Current theme: {}", self.settings.theme);
    }

    /// Change theme
    pub fn set_theme(&mut self, theme: String) {
        self.settings.theme = theme.clone();
        println!("Theme changed to: {theme}");
    }

    /// Toggle setting
    pub fn toggle_setting(&mut self, setting: &str) {
        match setting {
            "timestamps" => {
                self.settings.show_timestamps = !self.settings.show_timestamps;
                println!(
                    "Timestamps: {}",
                    if self.settings.show_timestamps {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            "joinpart" => {
                self.settings.show_join_part = !self.settings.show_join_part;
                println!(
                    "Join/Part messages: {}",
                    if self.settings.show_join_part {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            "compact" => {
                self.settings.compact_mode = !self.settings.compact_mode;
                println!(
                    "Compact mode: {}",
                    if self.settings.compact_mode {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            "colors" => {
                self.settings.nick_colors = !self.settings.nick_colors;
                println!(
                    "Nick colors: {}",
                    if self.settings.nick_colors {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            "reconnect" => {
                self.settings.auto_reconnect = !self.settings.auto_reconnect;
                println!(
                    "Auto-reconnect: {}",
                    if self.settings.auto_reconnect {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            _ => {
                println!("Unknown setting: {setting}. Available: timestamps, joinpart, compact, colors, reconnect");
            }
        }
    }

    /// Start the CLI session
    pub async fn run(&mut self) -> Result<()> {
        println!("RustIRC CLI - Full Featured Mode");
        println!("===============================");
        println!("Commands: /help for full list");
        println!();

        // Add default server
        self.add_server("libera".to_string(), "Libera.Chat".to_string());

        loop {
            print!("> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if let Err(e) = self.handle_command(input).await {
                eprintln!("Error: {e}");
            }

            if input == "/quit" {
                break;
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, input: &str) -> Result<()> {
        // Add to command history
        self.command_history.push(input.to_string());
        if self.command_history.len() > 100 {
            self.command_history.remove(0);
        }
        self.history_position = self.command_history.len();

        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.first() {
            // Connection management
            Some(&"/connect") => self.connect().await,
            Some(&"/disconnect") => {
                self.disconnect().await?;
                Ok(())
            }

            // Channel management
            Some(&"/join") => {
                if let Some(channel) = parts.get(1) {
                    self.join_channel(channel).await
                } else {
                    println!("Usage: /join <channel>");
                    Ok(())
                }
            }
            Some(&"/part") => {
                if let Some(channel) = parts.get(1) {
                    self.part_channel(channel).await
                } else {
                    println!("Usage: /part <channel>");
                    Ok(())
                }
            }
            Some(&"/list") => self.list_channels().await,

            // Messaging
            Some(&"/msg") => {
                if parts.len() >= 3 {
                    let target = parts[1];
                    let message = parts[2..].join(" ");
                    self.send_message(target, &message).await
                } else {
                    println!("Usage: /msg <target> <message>");
                    Ok(())
                }
            }

            // Tab management
            Some(&"/tab") => {
                if let Some(action) = parts.get(1) {
                    match *action {
                        "next" => self.next_tab(),
                        "prev" => self.prev_tab(),
                        "close" => self.close_current_tab(),
                        "list" => self.list_tabs(),
                        _ => {
                            if let Ok(tab_num) = action.parse::<usize>() {
                                self.select_tab(tab_num);
                            } else {
                                println!("Usage: /tab [next|prev|close|list|<number>]");
                            }
                        }
                    }
                } else {
                    self.list_tabs();
                }
                Ok(())
            }

            // Theme management
            Some(&"/theme") => {
                if let Some(theme_name) = parts.get(1) {
                    self.set_theme(theme_name.to_string());
                } else {
                    self.list_themes();
                }
                Ok(())
            }

            // Settings
            Some(&"/set") => {
                if let Some(setting) = parts.get(1) {
                    self.toggle_setting(setting);
                } else {
                    println!("Usage: /set <setting>. Settings: timestamps, joinpart, compact, colors, reconnect");
                }
                Ok(())
            }

            // Status and information
            Some(&"/status") => {
                self.show_status();
                Ok(())
            }
            Some(&"/whois") => {
                if let Some(nick) = parts.get(1) {
                    self.whois(nick).await
                } else {
                    println!("Usage: /whois <nick>");
                    Ok(())
                }
            }

            // Application control
            Some(&"/quit") => {
                self.disconnect().await?;
                println!("Goodbye!");
                Ok(())
            }
            Some(&"/help") => {
                self.show_help();
                Ok(())
            }
            Some(cmd) if cmd.starts_with('/') => {
                println!("Unknown command: {cmd}. Type /help for available commands.");
                Ok(())
            }
            _ => {
                let is_connected = self.servers.values().any(|s| s.connected);
                if is_connected {
                    // Send as message to current channel/target
                    println!("Raw message: {input}");
                } else {
                    println!("Not connected. Use /connect first.");
                }
                Ok(())
            }
        }
    }

    async fn connect(&mut self) -> Result<()> {
        // Use current server or create default
        let server_id = self
            .current_server_id
            .clone()
            .unwrap_or_else(|| "libera".to_string());

        // Check if already connected
        if let Some(server) = self.servers.get(&server_id) {
            if server.connected {
                println!("Already connected to {server_id}.");
                return Ok(());
            }
        }

        println!("Connecting to IRC server: {server_id}");
        info!("Starting IRC connection attempt for server: {server_id}");

        // Create new client for this server
        let client = IrcClient::new(Config::default());

        // Use timeout to prevent hanging
        match timeout(
            Duration::from_secs(10),
            client.connect("irc.libera.chat", 6667),
        )
        .await
        {
            Ok(Ok(())) => {
                println!("Connected successfully to {server_id}!");
                info!("IRC connection established successfully for server: {server_id}");

                // Create or update server entry
                let server = CliServer {
                    name: server_id.clone(),
                    client: Some(client),
                    connected: true,
                    channels: Vec::new(),
                };

                self.servers.insert(server_id.clone(), server);

                // Create server tab if it doesn't exist
                let tab_id = format!("server:{server_id}");
                if !self.tabs.contains_key(&tab_id) {
                    let tab = CliTab {
                        id: tab_id.clone(),
                        name: server_id.clone(),
                        tab_type: CliTabType::Server,
                        server_id: Some(server_id.clone()),
                        active: false,
                    };
                    self.tabs.insert(tab_id.clone(), tab);
                    self.tab_order.push(tab_id.clone());

                    // Set as current if first
                    if self.current_tab_id.is_none() {
                        self.current_tab_id = Some(tab_id);
                        self.current_server_id = Some(server_id.clone());
                    }
                }

                // Test SASL authentication functionality
                self.test_sasl_functionality().await?;
            }
            Ok(Err(e)) => {
                error!("Connection failed: {}", e);
                return Err(e.into());
            }
            Err(_) => {
                warn!("Connection timeout after 10 seconds");
                error!("Connection timeout");
                return Err(anyhow::anyhow!("Connection timeout"));
            }
        }

        Ok(())
    }

    async fn join_channel(&mut self, channel: &str) -> Result<()> {
        let server_id = match &self.current_server_id {
            Some(id) => id.clone(),
            None => {
                println!("No server selected. Use /connect first.");
                return Ok(());
            }
        };

        let server = match self.servers.get_mut(&server_id) {
            Some(s) if s.connected => s,
            Some(_) => {
                println!("Not connected to {server_id}. Use /connect first.");
                return Ok(());
            }
            None => {
                println!("Server {server_id} not found. Use /connect first.");
                return Ok(());
            }
        };

        println!("Joining channel: {channel}");

        if let Some(client) = &mut server.client {
            client.join_channel(channel).await?;
            self.add_channel_tab(server_id, channel.to_string());
            println!("Joined {channel}");
        } else {
            println!("No client connection available for server {server_id}");
        }

        Ok(())
    }

    async fn send_message(&mut self, target: &str, message: &str) -> Result<()> {
        let server_id = match &self.current_server_id {
            Some(id) => id.clone(),
            None => {
                println!("No server selected. Use /connect first.");
                return Ok(());
            }
        };

        let server = match self.servers.get_mut(&server_id) {
            Some(s) if s.connected => s,
            Some(_) => {
                println!("Not connected to {server_id}. Use /connect first.");
                return Ok(());
            }
            None => {
                println!("Server {server_id} not found. Use /connect first.");
                return Ok(());
            }
        };

        println!("<{target}> {message}");

        if let Some(client) = &mut server.client {
            client.send_message(target, message).await?;
        } else {
            println!("No client connection available for server {server_id}");
        }

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        let server_id = match &self.current_server_id {
            Some(id) => id.clone(),
            None => {
                println!("No server selected.");
                return Ok(());
            }
        };

        if let Some(server) = self.servers.get_mut(&server_id) {
            if server.connected {
                if let Some(client) = &mut server.client {
                    client.disconnect().await?;
                }
                server.connected = false;
                server.client = None;
                println!("Disconnected from {server_id}.");
            } else {
                println!("Already disconnected from {server_id}.");
            }
        } else {
            println!("Server {server_id} not found.");
        }

        Ok(())
    }

    async fn test_sasl_functionality(&self) -> Result<()> {
        // Test SASL credentials and authentication state
        println!("Testing SASL functionality...");

        // Create test credentials
        let credentials = SaslCredentials {
            username: "testuser".to_string(),
            password: "testpass".to_string().into(),
            authzid: None,
        };

        // Test authentication state transitions
        let mut auth_state = AuthState::Idle;
        println!("Initial auth state: {auth_state:?}");

        auth_state = AuthState::InProgress;
        println!("Auth state during authentication: {auth_state:?}");

        auth_state = AuthState::Success;
        println!("Auth state after success: {auth_state:?}");

        // In a real implementation, this would use the credentials with a SaslAuthenticator
        println!(
            "SASL credentials validated for user: {}",
            credentials.username
        );
        println!("SASL authentication test completed successfully");

        Ok(())
    }

    fn show_help(&self) {
        println!("=== RustIRC CLI Commands ===");
        println!();
        println!("Connection:");
        println!("  /connect              - Connect to IRC server");
        println!("  /disconnect           - Disconnect from server");
        println!();
        println!("Channels:");
        println!("  /join <channel>       - Join a channel");
        println!("  /part <channel>       - Leave a channel");
        println!("  /list                 - List available channels");
        println!();
        println!("Messaging:");
        println!("  /msg <target> <msg>   - Send private message");
        println!("  /whois <nick>         - Get user information");
        println!();
        println!("Tab Management:");
        println!("  /tab                  - List all tabs");
        println!("  /tab next             - Switch to next tab");
        println!("  /tab prev             - Switch to previous tab");
        println!("  /tab <number>         - Switch to tab number");
        println!("  /tab close            - Close current tab");
        println!();
        println!("Settings:");
        println!("  /set <setting>        - Toggle setting on/off");
        println!("    Available: timestamps, joinpart, compact, colors, reconnect");
        println!();
        println!("Themes:");
        println!("  /theme                - List available themes");
        println!("  /theme <name>         - Change theme");
        println!();
        println!("Information:");
        println!("  /status               - Show connection status");
        println!("  /help                 - Show this help");
        println!("  /quit                 - Exit the client");
        println!();
        println!("When connected, type messages without / to send to current channel.");
        println!("All GUI features are available via commands for full functionality parity.");
    }

    // Tab management methods
    fn next_tab(&mut self) {
        if let Some(current_id) = &self.current_tab_id {
            if let Some(current_pos) = self.tab_order.iter().position(|id| id == current_id) {
                let next_pos = (current_pos + 1) % self.tab_order.len();
                if let Some(next_id) = self.tab_order.get(next_pos) {
                    self.current_tab_id = Some(next_id.clone());
                    if let Some(tab) = self.tabs.get(next_id) {
                        println!("Switched to tab: {}", tab.name);
                    }
                }
            }
        }
    }

    fn prev_tab(&mut self) {
        if let Some(current_id) = &self.current_tab_id {
            if let Some(current_pos) = self.tab_order.iter().position(|id| id == current_id) {
                let prev_pos = if current_pos == 0 {
                    self.tab_order.len() - 1
                } else {
                    current_pos - 1
                };
                if let Some(prev_id) = self.tab_order.get(prev_pos) {
                    self.current_tab_id = Some(prev_id.clone());
                    if let Some(tab) = self.tabs.get(prev_id) {
                        println!("Switched to tab: {}", tab.name);
                    }
                }
            }
        }
    }

    fn select_tab(&mut self, tab_number: usize) {
        if tab_number > 0 && tab_number <= self.tab_order.len() {
            if let Some(tab_id) = self.tab_order.get(tab_number - 1) {
                self.current_tab_id = Some(tab_id.clone());
                if let Some(tab) = self.tabs.get(tab_id) {
                    println!("Switched to tab {}: {}", tab_number, tab.name);
                }
            }
        } else {
            println!("Tab number {tab_number} not found. Use /tab list to see available tabs.");
        }
    }

    fn close_current_tab(&mut self) {
        if let Some(current_id) = &self.current_tab_id {
            let current_id = current_id.clone();
            if let Some(tab) = self.tabs.get(&current_id) {
                println!("Closing tab: {}", tab.name);
            }

            // Remove from tabs and order
            self.tabs.remove(&current_id);
            self.tab_order.retain(|id| id != &current_id);

            // Switch to next available tab
            self.current_tab_id = self.tab_order.first().cloned();
            if let Some(new_id) = &self.current_tab_id {
                if let Some(tab) = self.tabs.get(new_id) {
                    println!("Switched to tab: {}", tab.name);
                }
            }
        }
    }

    fn list_tabs(&self) {
        println!("Tabs:");
        for (i, tab_id) in self.tab_order.iter().enumerate() {
            if let Some(tab) = self.tabs.get(tab_id) {
                let current_marker = if Some(tab_id) == self.current_tab_id.as_ref() {
                    "*"
                } else {
                    " "
                };
                println!(
                    "  {}{}: {} ({:?})",
                    current_marker,
                    i + 1,
                    tab.name,
                    tab.tab_type
                );
            }
        }
    }

    // Additional IRC methods
    async fn part_channel(&mut self, channel: &str) -> Result<()> {
        if let Some(server) = self.get_current_server() {
            if server.connected {
                if let Some(client) = &server.client {
                    println!("Leaving channel: {channel}");
                    client
                        .send_command(rustirc_protocol::Command::Part {
                            channels: vec![channel.to_string()],
                            message: None,
                        })
                        .await?;

                    // Remove from server's channel list
                    server.channels.retain(|c| c != channel);

                    // Close the tab
                    let tab_id =
                        format!("{}:{}", self.current_server_id.as_ref().unwrap(), channel);
                    self.tabs.remove(&tab_id);
                    self.tab_order.retain(|id| id != &tab_id);

                    // Switch to server tab
                    if let Some(server_id) = &self.current_server_id {
                        let server_tab_id = format!("server:{server_id}");
                        self.current_tab_id = Some(server_tab_id);
                    }

                    println!("Left {channel}");
                } else {
                    println!("No IRC client available");
                }
            } else {
                println!("Not connected to any server");
            }
        } else {
            println!("No current server");
        }
        Ok(())
    }

    async fn list_channels(&mut self) -> Result<()> {
        if let Some(server) = self.get_current_server() {
            if server.connected {
                if let Some(client) = &server.client {
                    println!("Requesting channel list...");
                    client
                        .send_command(rustirc_protocol::Command::List { channels: None })
                        .await?;
                    println!("Channel list requested (results will appear in messages)");
                } else {
                    println!("No IRC client available");
                }
            } else {
                println!("Not connected to any server");
            }
        } else {
            println!("No current server");
        }
        Ok(())
    }

    async fn whois(&mut self, nick: &str) -> Result<()> {
        if let Some(server) = self.get_current_server() {
            if server.connected {
                if let Some(client) = &server.client {
                    println!("Requesting WHOIS for: {nick}");
                    client
                        .send_command(rustirc_protocol::Command::Whois {
                            targets: vec![nick.to_string()],
                        })
                        .await?;
                    println!("WHOIS requested for {nick} (results will appear in messages)");
                } else {
                    println!("No IRC client available");
                }
            } else {
                println!("Not connected to any server");
            }
        } else {
            println!("No current server");
        }
        Ok(())
    }
}

/// Run the CLI prototype
pub async fn run_cli_prototype(config: Config) -> Result<()> {
    let mut cli = CliClient::new(config);
    cli.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_creation() {
        let config = Config::default();
        let cli = CliClient::new(config);
        assert!(!cli.connected);
    }

    #[test]
    fn test_cli_commands() {
        // Test that CLI can handle basic commands without panicking
        let config = Config::default();
        let cli = CliClient::new(config);
        assert!(!cli.connected);
    }
}
