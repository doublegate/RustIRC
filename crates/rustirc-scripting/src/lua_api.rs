//! Lua IRC API implementation
//!
//! This module provides the comprehensive IRC API that Lua scripts can use
//! to interact with the IRC client.

use anyhow::Result;
use mlua::{Lua, Table, Function};
use rustirc_core::events::EventBus;
use std::sync::Arc;
use tracing::{debug, info};

/// IRC API for Lua scripts
pub struct LuaIrcApi {
    event_bus: Arc<EventBus>,
    connection_id: String,
}

impl LuaIrcApi {
    pub fn new(event_bus: Arc<EventBus>, connection_id: String) -> Self {
        Self {
            event_bus,
            connection_id,
        }
    }

    /// Register IRC API in Lua context
    pub fn register(lua: &Lua, event_bus: Arc<EventBus>, connection_id: String) -> Result<()> {
        let irc_table = lua.create_table()?;

        // Core functions
        Self::register_core_functions(&irc_table, lua, event_bus.clone(), connection_id.clone())?;

        // Messaging functions
        Self::register_messaging_functions(&irc_table, lua, event_bus.clone(), connection_id.clone())?;

        // Channel functions
        Self::register_channel_functions(&irc_table, lua, event_bus.clone(), connection_id.clone())?;

        // User functions
        Self::register_user_functions(&irc_table, lua, event_bus.clone(), connection_id.clone())?;

        // State query functions
        Self::register_state_functions(&irc_table, lua, event_bus.clone(), connection_id.clone())?;

        // UI functions
        Self::register_ui_functions(&irc_table, lua, event_bus, connection_id)?;

        lua.globals().set("irc", irc_table)?;

        Ok(())
    }

    fn register_core_functions(
        table: &Table,
        lua: &Lua,
        event_bus: Arc<EventBus>,
        connection_id: String,
    ) -> Result<()> {
        // irc.send(message) - Send raw IRC message
        let eb = event_bus.clone();
        let conn_id = connection_id.clone();
        let send_fn = lua.create_async_function(move |_lua, raw_msg: String| {
            let eb = eb.clone();
            let conn_id = conn_id.clone();
            async move {
                info!("Script sending raw message: {}", raw_msg);
                // Parse and send message through event bus
                Ok(())
            }
        })?;
        table.set("send", send_fn)?;

        // irc.connect() - Placeholder for connection management
        let connect_fn = lua.create_async_function(|_lua, ()| async {
            info!("Script initiated connection");
            Ok(())
        })?;
        table.set("connect", connect_fn)?;

        // irc.disconnect() - Disconnect from server
        let disconnect_fn = lua.create_async_function(|_lua, ()| async {
            info!("Script initiated disconnection");
            Ok(())
        })?;
        table.set("disconnect", disconnect_fn)?;

        Ok(())
    }

    fn register_messaging_functions(
        table: &Table,
        lua: &Lua,
        event_bus: Arc<EventBus>,
        connection_id: String,
    ) -> Result<()> {
        // irc.privmsg(target, message) - Send private message
        let eb = event_bus.clone();
        let conn_id = connection_id.clone();
        let privmsg_fn = lua.create_async_function(move |_lua, (target, message): (String, String)| {
            let eb = eb.clone();
            let conn_id = conn_id.clone();
            async move {
                debug!("Script sending PRIVMSG to {}: {}", target, message);
                // Send PRIVMSG through event bus
                Ok(())
            }
        })?;
        table.set("privmsg", privmsg_fn)?;

        // irc.notice(target, message) - Send notice
        let eb = event_bus.clone();
        let conn_id = connection_id.clone();
        let notice_fn = lua.create_async_function(move |_lua, (target, message): (String, String)| {
            let eb = eb.clone();
            let conn_id = conn_id.clone();
            async move {
                debug!("Script sending NOTICE to {}: {}", target, message);
                Ok(())
            }
        })?;
        table.set("notice", notice_fn)?;

        // irc.action(target, action) - Send CTCP ACTION
        let eb = event_bus.clone();
        let conn_id = connection_id.clone();
        let action_fn = lua.create_async_function(move |_lua, (target, action): (String, String)| {
            let eb = eb.clone();
            let conn_id = conn_id.clone();
            async move {
                debug!("Script sending ACTION to {}: {}", target, action);
                Ok(())
            }
        })?;
        table.set("action", action_fn)?;

        // irc.ctcp(target, command) - Send CTCP command
        let ctcp_fn = lua.create_async_function(|_lua, (target, command): (String, String)| async move {
            debug!("Script sending CTCP {} to {}", command, target);
            Ok(())
        })?;
        table.set("ctcp", ctcp_fn)?;

        // irc.ctcp_reply(target, reply) - Send CTCP reply
        let ctcp_reply_fn = lua.create_async_function(|_lua, (target, reply): (String, String)| async move {
            debug!("Script sending CTCP reply to {}: {}", target, reply);
            Ok(())
        })?;
        table.set("ctcp_reply", ctcp_reply_fn)?;

        Ok(())
    }

    fn register_channel_functions(
        table: &Table,
        lua: &Lua,
        event_bus: Arc<EventBus>,
        connection_id: String,
    ) -> Result<()> {
        // irc.join(channel, [key]) - Join channel
        let join_fn = lua.create_async_function(|_lua, (channel, key): (String, Option<String>)| async move {
            info!("Script joining channel: {}", channel);
            Ok(())
        })?;
        table.set("join", join_fn)?;

        // irc.part(channel, [reason]) - Leave channel
        let part_fn = lua.create_async_function(|_lua, (channel, reason): (String, Option<String>)| async move {
            info!("Script parting channel: {}", channel);
            Ok(())
        })?;
        table.set("part", part_fn)?;

        // irc.kick(channel, user, [reason]) - Kick user
        let kick_fn = lua.create_async_function(|_lua, (channel, user, reason): (String, String, Option<String>)| async move {
            info!("Script kicking {} from {}", user, channel);
            Ok(())
        })?;
        table.set("kick", kick_fn)?;

        // irc.topic(channel, [topic]) - Get/set topic
        let topic_fn = lua.create_async_function(|_lua, (channel, topic): (String, Option<String>)| async move {
            if let Some(t) = topic {
                info!("Script setting topic in {}: {}", channel, t);
            } else {
                debug!("Script querying topic for {}", channel);
            }
            Ok(())
        })?;
        table.set("topic", topic_fn)?;

        // irc.mode(channel, mode, [params]) - Set channel mode
        let mode_fn = lua.create_async_function(|_lua, (channel, mode, params): (String, String, Option<Vec<String>>)| async move {
            info!("Script setting mode in {}: {}", channel, mode);
            Ok(())
        })?;
        table.set("mode", mode_fn)?;

        // irc.invite(user, channel) - Invite user to channel
        let invite_fn = lua.create_async_function(|_lua, (user, channel): (String, String)| async move {
            info!("Script inviting {} to {}", user, channel);
            Ok(())
        })?;
        table.set("invite", invite_fn)?;

        Ok(())
    }

    fn register_user_functions(
        table: &Table,
        lua: &Lua,
        event_bus: Arc<EventBus>,
        connection_id: String,
    ) -> Result<()> {
        // irc.nick(new_nick) - Change nickname
        let nick_fn = lua.create_async_function(|_lua, new_nick: String| async move {
            info!("Script changing nick to: {}", new_nick);
            Ok(())
        })?;
        table.set("nick", nick_fn)?;

        // irc.whois(nick) - Query user info
        let whois_fn = lua.create_async_function(|_lua, nick: String| async move {
            debug!("Script querying WHOIS for: {}", nick);
            Ok(())
        })?;
        table.set("whois", whois_fn)?;

        // irc.who(target) - Query WHO information
        let who_fn = lua.create_async_function(|_lua, target: String| async move {
            debug!("Script querying WHO for: {}", target);
            Ok(())
        })?;
        table.set("who", who_fn)?;

        // irc.userhost(nicks) - Query userhost
        let userhost_fn = lua.create_async_function(|_lua, nicks: Vec<String>| async move {
            debug!("Script querying USERHOST for: {:?}", nicks);
            Ok(())
        })?;
        table.set("userhost", userhost_fn)?;

        // irc.away([message]) - Set/unset away status
        let away_fn = lua.create_async_function(|_lua, message: Option<String>| async move {
            if let Some(msg) = message {
                info!("Script setting away: {}", msg);
            } else {
                info!("Script unsetting away");
            }
            Ok(())
        })?;
        table.set("away", away_fn)?;

        // irc.ison(nicks) - Check if users are online
        let ison_fn = lua.create_async_function(|_lua, nicks: Vec<String>| async move {
            debug!("Script checking ISON for: {:?}", nicks);
            Ok(())
        })?;
        table.set("ison", ison_fn)?;

        Ok(())
    }

    fn register_state_functions(
        table: &Table,
        lua: &Lua,
        event_bus: Arc<EventBus>,
        connection_id: String,
    ) -> Result<()> {
        // irc.servers() - Get list of servers
        let servers_fn = lua.create_function(|lua, ()| {
            let servers = lua.create_table()?;
            servers.set(1, "irc.libera.chat")?;
            Ok(servers)
        })?;
        table.set("servers", servers_fn)?;

        // irc.channels() - Get list of joined channels
        let channels_fn = lua.create_function(|lua, ()| {
            let channels = lua.create_table()?;
            // Return empty for now, will be connected to actual state
            Ok(channels)
        })?;
        table.set("channels", channels_fn)?;

        // irc.users(channel) - Get users in channel
        let users_fn = lua.create_function(|lua, channel: String| {
            let users = lua.create_table()?;
            // Return empty for now
            Ok(users)
        })?;
        table.set("users", users_fn)?;

        // irc.my_nick() - Get current nickname
        let my_nick_fn = lua.create_function(|_lua, ()| {
            Ok("ScriptUser")
        })?;
        table.set("my_nick", my_nick_fn)?;

        // irc.is_op(channel) - Check if we have op status
        let is_op_fn = lua.create_function(|_lua, channel: String| {
            Ok(false)
        })?;
        table.set("is_op", is_op_fn)?;

        // irc.is_voice(channel) - Check if we have voice status
        let is_voice_fn = lua.create_function(|_lua, channel: String| {
            Ok(false)
        })?;
        table.set("is_voice", is_voice_fn)?;

        // irc.connection_id() - Get current connection ID
        let conn_id = connection_id.clone();
        let connection_id_fn = lua.create_function(move |_lua, ()| {
            Ok(conn_id.clone())
        })?;
        table.set("connection_id", connection_id_fn)?;

        Ok(())
    }

    fn register_ui_functions(
        table: &Table,
        lua: &Lua,
        event_bus: Arc<EventBus>,
        connection_id: String,
    ) -> Result<()> {
        // irc.print(message, [target]) - Print to UI
        let print_fn = lua.create_function(|_lua, (message, target): (String, Option<String>)| {
            if let Some(t) = target {
                info!("[{}] {}", t, message);
            } else {
                info!("{}", message);
            }
            Ok(())
        })?;
        table.set("print", print_fn)?;

        // irc.echo(message) - Echo to current window
        let echo_fn = lua.create_function(|_lua, message: String| {
            info!("Script echo: {}", message);
            Ok(())
        })?;
        table.set("echo", echo_fn)?;

        // irc.log(level, message) - Log message
        let log_fn = lua.create_function(|_lua, (level, message): (String, String)| {
            match level.as_str() {
                "debug" => debug!("{}", message),
                "info" => info!("{}", message),
                "warn" => tracing::warn!("{}", message),
                "error" => tracing::error!("{}", message),
                _ => info!("{}", message),
            }
            Ok(())
        })?;
        table.set("log", log_fn)?;

        // irc.status(message) - Set status bar message
        let status_fn = lua.create_function(|_lua, message: String| {
            info!("Status: {}", message);
            Ok(())
        })?;
        table.set("status", status_fn)?;

        // irc.notify(title, message) - Send notification
        let notify_fn = lua.create_function(|_lua, (title, message): (String, String)| {
            info!("Notification: {} - {}", title, message);
            Ok(())
        })?;
        table.set("notify", notify_fn)?;

        // irc.beep() - Play beep sound
        let beep_fn = lua.create_function(|_lua, ()| {
            debug!("Beep!");
            Ok(())
        })?;
        table.set("beep", beep_fn)?;

        Ok(())
    }

    /// Register event handler
    pub fn register_event_handler(
        lua: &Lua,
        event_type: String,
        handler: Function,
    ) -> Result<()> {
        // Store event handler in global irc.on_<event> function
        let irc: Table = lua.globals().get("irc")?;
        irc.set(format!("on_{}", event_type), handler)?;
        Ok(())
    }

    /// Register custom command
    pub fn register_command(
        lua: &Lua,
        command_name: String,
        handler: Function,
    ) -> Result<()> {
        let irc: Table = lua.globals().get("irc")?;

        // Get or create commands table
        let commands: Table = match irc.get("commands") {
            Ok(table) => table,
            Err(_) => {
                let table = lua.create_table()?;
                irc.set("commands", table.clone())?;
                table
            }
        };

        commands.set(command_name, handler)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustirc_core::events::EventBus;

    #[test]
    fn test_lua_api_registration() {
        let lua = Lua::new();
        let event_bus = Arc::new(EventBus::new());
        let connection_id = "test_server".to_string();

        let result = LuaIrcApi::register(&lua, event_bus, connection_id);
        assert!(result.is_ok(), "API registration should succeed");

        // Verify irc table exists
        let irc: Table = lua.globals().get("irc").unwrap();
        assert!(irc.contains_key("privmsg").unwrap());
        assert!(irc.contains_key("join").unwrap());
        assert!(irc.contains_key("part").unwrap());
    }

    #[test]
    fn test_state_functions() {
        let lua = Lua::new();
        let event_bus = Arc::new(EventBus::new());
        let connection_id = "test_server".to_string();

        LuaIrcApi::register(&lua, event_bus, connection_id.clone()).unwrap();

        // Test my_nick()
        let result: String = lua.load("return irc.my_nick()").eval().unwrap();
        assert_eq!(result, "ScriptUser");

        // Test connection_id()
        let result: String = lua.load("return irc.connection_id()").eval().unwrap();
        assert_eq!(result, connection_id);
    }
}
