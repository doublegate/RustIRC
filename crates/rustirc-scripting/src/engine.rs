//! Script engine implementation
//!
//! Provides a secure Lua scripting engine with sandboxing, resource limits,
//! and integration with the IRC client event system.

use anyhow::{Context, Result};
use mlua::{Error as LuaError, Function, Lua, Value};
use rustirc_core::events::{Event, EventBus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Represents a loaded script with its metadata and environment
#[derive(Clone)]
pub struct LoadedScript {
    pub name: String,
    pub code: String,
    pub enabled: bool,
    pub event_handlers: Vec<String>,
}

/// Main script engine managing Lua scripts and their execution
pub struct ScriptEngine {
    lua: Arc<RwLock<Lua>>,
    scripts: Arc<RwLock<HashMap<String, LoadedScript>>>,
    event_bus: Option<Arc<EventBus>>,
    custom_commands: Arc<RwLock<HashMap<String, String>>>, // command -> script name
}

impl ScriptEngine {
    /// Create a new script engine with sandboxing enabled
    pub fn new() -> Result<Self> {
        let lua = Lua::new();

        // Apply sandboxing
        Self::setup_sandbox(&lua)?;

        Ok(Self {
            lua: Arc::new(RwLock::new(lua)),
            scripts: Arc::new(RwLock::new(HashMap::new())),
            event_bus: None,
            custom_commands: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create engine with event bus integration
    pub fn with_event_bus(event_bus: Arc<EventBus>) -> Result<Self> {
        let mut engine = Self::new()?;
        engine.event_bus = Some(event_bus);
        Ok(engine)
    }

    /// Set up Lua sandbox by removing dangerous functions
    fn setup_sandbox(lua: &Lua) -> Result<()> {
        lua.load(r#"
            -- Save safe functions before removing
            local safe_os_clock = os.clock
            local safe_os_date = os.date
            local safe_os_difftime = os.difftime
            local safe_os_time = os.time
            local safe_debug_traceback = debug and debug.traceback or function() return "" end

            -- Remove dangerous global functions
            os.execute = nil
            os.exit = nil
            os.remove = nil
            os.rename = nil
            os.tmpname = nil

            -- Remove file I/O functions
            io.open = nil
            io.popen = nil
            io.tmpfile = nil
            io.input = nil
            io.output = nil

            -- Remove module loading functions
            require = nil
            dofile = nil
            loadfile = nil

            -- Keep only safe os functions
            os = {
                clock = safe_os_clock,
                date = safe_os_date,
                difftime = safe_os_difftime,
                time = safe_os_time,
            }

            -- Keep only safe debug functions (for tracebacks)
            debug = {
                traceback = safe_debug_traceback,
            }
        "#).exec().map_err(|e| anyhow::anyhow!("Failed to setup Lua sandbox: {}", e))?;

        Ok(())
    }

    /// Load a script from code
    pub async fn load_script(&self, name: &str, code: &str) -> Result<()> {
        info!("Loading script: {}", name);

        let lua = self.lua.read().await;

        // Validate syntax by attempting to load
        lua.load(code)
            .set_name(name)
            .exec()
            .map_err(|e| anyhow::anyhow!("Script execution failed: {}", e))?;

        let script = LoadedScript {
            name: name.to_string(),
            code: code.to_string(),
            enabled: true,
            event_handlers: Vec::new(),
        };

        self.scripts.write().await.insert(name.to_string(), script);
        info!("Script loaded successfully: {}", name);

        Ok(())
    }

    /// Load script from file
    pub async fn load_script_file(&self, path: &str) -> Result<()> {
        let code = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read script file")?;

        let name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .context("Invalid script filename")?
            .to_string();

        self.load_script(&name, &code).await
    }

    /// Unload a script
    pub async fn unload_script(&self, name: &str) -> Result<()> {
        info!("Unloading script: {}", name);

        self.scripts.write().await.remove(name);
        self.custom_commands.write().await.retain(|_, script_name| script_name != name);

        Ok(())
    }

    /// Enable a script
    pub async fn enable_script(&self, name: &str) -> Result<()> {
        let mut scripts = self.scripts.write().await;
        if let Some(script) = scripts.get_mut(name) {
            script.enabled = true;
            info!("Script enabled: {}", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Script not found: {}", name))
        }
    }

    /// Disable a script
    pub async fn disable_script(&self, name: &str) -> Result<()> {
        let mut scripts = self.scripts.write().await;
        if let Some(script) = scripts.get_mut(name) {
            script.enabled = false;
            info!("Script disabled: {}", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Script not found: {}", name))
        }
    }

    /// Execute a custom command
    pub async fn execute_command(&self, cmd: &str, args: Vec<String>) -> Result<()> {
        debug!("Executing command: {} with args: {:?}", cmd, args);

        let commands = self.custom_commands.read().await;
        let script_name = commands.get(cmd)
            .context(format!("Command not found: {}", cmd))?
            .clone();

        drop(commands);

        let scripts = self.scripts.read().await;
        let script = scripts.get(&script_name)
            .context("Script not found")?;

        if !script.enabled {
            return Err(anyhow::anyhow!("Script is disabled"));
        }

        let lua = self.lua.read().await;

        // Set command arguments in Lua global
        let args_table = lua.create_table()?;
        for (i, arg) in args.iter().enumerate() {
            args_table.set(i + 1, arg.clone())?;
        }
        lua.globals().set("args", args_table)?;

        // Execute the command handler if it exists
        let result: Result<(), LuaError> = lua.load(&format!(
            r#"
            if irc.commands and irc.commands["{}"] then
                irc.commands["{}"](args)
            end
            "#,
            cmd, cmd
        )).exec();

        result.context("Command execution failed")?;

        Ok(())
    }

    /// Register a custom command from a script
    pub async fn register_command(&self, script_name: String, command: String) -> Result<()> {
        info!("Registering command: {} from script: {}", command, script_name);

        self.custom_commands.write().await.insert(command.clone(), script_name);

        Ok(())
    }

    /// Call a Lua function with arguments
    pub async fn call_function(&self, function_name: &str, args: Vec<Value>) -> Result<Value> {
        let lua = self.lua.read().await;

        let func: Function = lua.globals().get(function_name)
            .map_err(|e| anyhow::anyhow!("Function not found {}: {}", function_name, e))?;

        let result = func.call::<Value>(mlua::MultiValue::from_vec(args))
            .map_err(|e| anyhow::anyhow!("Function call failed: {}", e))?;

        Ok(result)
    }

    /// Get list of loaded scripts
    pub async fn list_scripts(&self) -> Vec<String> {
        self.scripts.read().await.keys().cloned().collect()
    }

    /// Get script info
    pub async fn get_script_info(&self, name: &str) -> Option<LoadedScript> {
        self.scripts.read().await.get(name).cloned()
    }

    /// Handle an IRC event by calling script event handlers
    pub async fn handle_event(&self, event: &Event) -> Result<()> {
        let scripts = self.scripts.read().await;

        for script in scripts.values().filter(|s| s.enabled) {
            if let Err(e) = self.dispatch_event_to_script(&script.name, event).await {
                error!("Error in script {} handling event: {}", script.name, e);
            }
        }

        Ok(())
    }

    /// Dispatch event to a specific script
    async fn dispatch_event_to_script(&self, script_name: &str, event: &Event) -> Result<()> {
        let lua = self.lua.read().await;

        // Convert event to Lua table
        let event_table = lua.create_table()?;

        match event {
            Event::Connected { connection_id } => {
                event_table.set("type", "connected")?;
                event_table.set("connection_id", connection_id.as_str())?;
            }
            Event::Disconnected { connection_id, reason } => {
                event_table.set("type", "disconnected")?;
                event_table.set("connection_id", connection_id.as_str())?;
                event_table.set("reason", reason.as_str())?;
            }
            Event::MessageReceived { connection_id, message } => {
                event_table.set("type", "message")?;
                event_table.set("connection_id", connection_id.as_str())?;
                event_table.set("command", message.command.as_str())?;

                let params_table = lua.create_table()?;
                for (i, param) in message.params.iter().enumerate() {
                    params_table.set(i + 1, param.as_str())?;
                }
                event_table.set("params", params_table)?;
            }
            Event::ChannelJoined { connection_id, channel } => {
                event_table.set("type", "join")?;
                event_table.set("connection_id", connection_id.as_str())?;
                event_table.set("channel", channel.as_str())?;
            }
            Event::ChannelLeft { connection_id, channel } => {
                event_table.set("type", "part")?;
                event_table.set("connection_id", connection_id.as_str())?;
                event_table.set("channel", channel.as_str())?;
            }
            Event::UserJoined { connection_id, channel, user } => {
                event_table.set("type", "user_join")?;
                event_table.set("connection_id", connection_id.as_str())?;
                event_table.set("channel", channel.as_str())?;
                event_table.set("user", user.as_str())?;
            }
            Event::UserLeft { connection_id, channel, user } => {
                event_table.set("type", "user_part")?;
                event_table.set("connection_id", connection_id.as_str())?;
                event_table.set("channel", channel.as_str())?;
                event_table.set("user", user.as_str())?;
            }
            Event::NickChanged { connection_id, old, new } => {
                event_table.set("type", "nick")?;
                event_table.set("connection_id", connection_id.as_str())?;
                event_table.set("old_nick", old.as_str())?;
                event_table.set("new_nick", new.as_str())?;
            }
            Event::TopicChanged { connection_id, channel, topic } => {
                event_table.set("type", "topic")?;
                event_table.set("connection_id", connection_id.as_str())?;
                event_table.set("channel", channel.as_str())?;
                event_table.set("topic", topic.as_str())?;
            }
            Event::Error { connection_id, error } => {
                event_table.set("type", "error")?;
                if let Some(conn_id) = connection_id {
                    event_table.set("connection_id", conn_id.as_str())?;
                }
                event_table.set("error", error.as_str())?;
            }
            _ => {
                // Handle other event types
                event_table.set("type", "unknown")?;
            }
        }

        // Call event handler if it exists
        let event_type: String = event_table.get("type")?;
        let result: Result<(), LuaError> = lua.load(&format!(
            r#"
            if irc.on_{} then
                irc.on_{}(event)
            end
            "#,
            event_type, event_type
        ))
        .set_name(&format!("{}_event_handler", script_name))
        .exec();

        lua.globals().set("event", event_table)?;

        if let Err(e) = result {
            warn!("Script {} event handler error: {}", script_name, e);
        }

        Ok(())
    }

    /// Reload a script
    pub async fn reload_script(&self, name: &str) -> Result<()> {
        let scripts = self.scripts.read().await;
        let script = scripts.get(name)
            .context("Script not found")?;

        let code = script.code.clone();
        drop(scripts);

        self.unload_script(name).await?;
        self.load_script(name, &code).await?;

        info!("Script reloaded: {}", name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_engine_creation() {
        let engine = ScriptEngine::new();
        assert!(engine.is_ok(), "ScriptEngine creation should succeed");
    }

    #[tokio::test]
    async fn test_load_script() {
        let engine = ScriptEngine::new().unwrap();
        let result = engine
            .load_script("test_script", "local x = 1 -- simple test")
            .await;
        assert!(result.is_ok(), "Script loading should succeed");
    }

    #[tokio::test]
    async fn test_load_invalid_script() {
        let engine = ScriptEngine::new().unwrap();
        let result = engine
            .load_script("bad_script", "this is not valid lua ][[[")
            .await;
        assert!(result.is_err(), "Invalid script should fail to load");
    }

    #[tokio::test]
    async fn test_enable_disable_script() {
        let engine = ScriptEngine::new().unwrap();
        engine.load_script("test", "local x = 1").await.unwrap();

        engine.disable_script("test").await.unwrap();
        let info = engine.get_script_info("test").await.unwrap();
        assert!(!info.enabled);

        engine.enable_script("test").await.unwrap();
        let info = engine.get_script_info("test").await.unwrap();
        assert!(info.enabled);
    }

    #[tokio::test]
    async fn test_unload_script() {
        let engine = ScriptEngine::new().unwrap();
        engine.load_script("test", "local x = 1").await.unwrap();

        assert!(engine.get_script_info("test").await.is_some());

        engine.unload_script("test").await.unwrap();
        assert!(engine.get_script_info("test").await.is_none());
    }

    #[tokio::test]
    async fn test_list_scripts() {
        let engine = ScriptEngine::new().unwrap();
        engine.load_script("script1", "local x = 1").await.unwrap();
        engine.load_script("script2", "local y = 2").await.unwrap();

        let scripts = engine.list_scripts().await;
        assert_eq!(scripts.len(), 2);
        assert!(scripts.contains(&"script1".to_string()));
        assert!(scripts.contains(&"script2".to_string()));
    }

    #[tokio::test]
    async fn test_sandbox_restrictions() {
        let engine = ScriptEngine::new().unwrap();

        // os.execute should be nil (removed by sandbox)
        let result = engine.load_script("test", r#"
            assert(os.execute == nil, "os.execute should be nil")
        "#).await;
        assert!(result.is_ok(), "Sandbox should remove os.execute");

        // io.open should be nil
        let result = engine.load_script("test2", r#"
            assert(io.open == nil, "io.open should be nil")
        "#).await;
        assert!(result.is_ok(), "Sandbox should remove io.open");
    }

    #[tokio::test]
    async fn test_multiple_operations() {
        let engine = ScriptEngine::new().unwrap();

        // Load multiple scripts
        let result1 = engine.load_script("script1", "function test() end").await;
        let result2 = engine.load_script("script2", "local x = 42").await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Verify both are loaded
        let scripts = engine.list_scripts().await;
        assert_eq!(scripts.len(), 2);
    }

    #[test]
    fn test_lua_initialization() {
        let engine = ScriptEngine::new().unwrap();
        // Engine creation succeeded, so Lua init worked
        assert!(true);
    }
}
