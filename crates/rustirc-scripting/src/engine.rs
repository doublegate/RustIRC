//! Lua script engine implementation
//!
//! Provides a sandboxed Lua scripting engine for IRC client automation.
//! Scripts can register event handlers, create custom commands, and
//! interact with the IRC client through a safe API.

use crate::sandbox::Sandbox;
use crate::script_message::ScriptMessage;
use anyhow::Result;
use mlua::{Function, Lua};
use rustirc_core::config::ScriptingConfig;
use rustirc_core::events::EventBus;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

/// A loaded and active Lua script
struct LoadedScript {
    name: String,
    lua: Lua,
    priority: i32,
}

// Safety: Lua is Send when created with `send` feature enabled in mlua
unsafe impl Send for LoadedScript {}
unsafe impl Sync for LoadedScript {}

/// The main script engine managing all loaded Lua scripts
pub struct ScriptEngine {
    scripts: Arc<RwLock<Vec<LoadedScript>>>,
    variables: Arc<RwLock<HashMap<String, String>>>,
    #[allow(dead_code)]
    event_bus: Option<Arc<EventBus>>,
    config: ScriptingConfig,
    sandbox: Sandbox,
}

impl ScriptEngine {
    /// Create a new script engine with default configuration
    pub fn new() -> Result<Self> {
        Ok(Self {
            scripts: Arc::new(RwLock::new(Vec::new())),
            variables: Arc::new(RwLock::new(HashMap::new())),
            event_bus: None,
            config: ScriptingConfig::default(),
            sandbox: Sandbox::default(),
        })
    }

    /// Create a script engine from configuration
    pub fn from_config(config: &ScriptingConfig) -> Result<Self> {
        Ok(Self {
            scripts: Arc::new(RwLock::new(Vec::new())),
            variables: Arc::new(RwLock::new(HashMap::new())),
            event_bus: None,
            config: config.clone(),
            sandbox: Sandbox::from_config(config),
        })
    }

    /// Create a script engine with an event bus for IRC integration
    pub fn with_event_bus(config: &ScriptingConfig, event_bus: Arc<EventBus>) -> Result<Self> {
        Ok(Self {
            scripts: Arc::new(RwLock::new(Vec::new())),
            variables: Arc::new(RwLock::new(HashMap::new())),
            event_bus: Some(event_bus),
            config: config.clone(),
            sandbox: Sandbox::from_config(config),
        })
    }

    /// Load a script from source code
    pub fn load_script(&self, name: &str, code: &str, priority: i32) -> Result<()> {
        let lua = Lua::new();

        // Apply sandbox restrictions
        self.sandbox.apply(&lua)?;

        // Set up IRC API
        self.setup_api(&lua)?;

        // Load and run the script
        lua.load(code).exec()?;

        let script = LoadedScript {
            name: name.to_string(),
            lua,
            priority,
        };

        let mut scripts = self.scripts.write().map_err(|e| anyhow::anyhow!("{}", e))?;
        scripts.push(script);
        scripts.sort_by_key(|s| -s.priority);

        info!("Loaded script: {} (priority: {})", name, priority);
        Ok(())
    }

    /// Load a script from a file
    pub fn load_script_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let code = std::fs::read_to_string(path)?;
        self.load_script(&name, &code, 0)
    }

    /// Unload a script by name
    pub fn unload_script(&self, name: &str) -> bool {
        if let Ok(mut scripts) = self.scripts.write() {
            let len_before = scripts.len();
            scripts.retain(|s| s.name != name);
            let removed = scripts.len() < len_before;
            if removed {
                info!("Unloaded script: {}", name);
            }
            removed
        } else {
            false
        }
    }

    /// Trigger an event across all loaded scripts
    pub fn trigger_event(&self, event: &str, msg: &ScriptMessage) -> Result<()> {
        let scripts = self.scripts.read().map_err(|e| anyhow::anyhow!("{}", e))?;

        for script in scripts.iter() {
            if let Ok(handlers) = script.lua.globals().get::<mlua::Table>("_handlers") {
                if let Ok(event_handlers) = handlers.get::<mlua::Table>(event) {
                    for (_, handler) in event_handlers.pairs::<i32, Function>().flatten() {
                        if let Err(e) = handler.call::<()>(msg.clone()) {
                            error!("Script error in {} handling {}: {}", script.name, event, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a custom command defined by scripts
    ///
    /// Returns true if a script handled the command
    pub fn execute_command(&self, cmd: &str, args: Vec<String>) -> Result<bool> {
        let scripts = self.scripts.read().map_err(|e| anyhow::anyhow!("{}", e))?;

        for script in scripts.iter() {
            if let Ok(commands) = script.lua.globals().get::<mlua::Table>("_commands") {
                if let Ok(handler) = commands.get::<Function>(cmd) {
                    match handler.call::<()>(args.clone()) {
                        Ok(()) => return Ok(true),
                        Err(e) => {
                            error!("Command error in {}: {}", script.name, e);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Get the list of loaded script names
    pub fn list_scripts(&self) -> Vec<String> {
        if let Ok(scripts) = self.scripts.read() {
            scripts.iter().map(|s| s.name.clone()).collect()
        } else {
            vec![]
        }
    }

    /// Auto-load scripts from the configured scripts directory
    pub fn auto_load_scripts(&self) -> usize {
        let scripts_path = PathBuf::from(&self.config.scripts_path);
        if !scripts_path.exists() {
            return 0;
        }

        let mut count = 0;

        // Load explicitly listed auto-load scripts first
        for script_name in &self.config.auto_load {
            let path = scripts_path.join(script_name);
            if path.exists() {
                match self.load_script_file(&path) {
                    Ok(()) => count += 1,
                    Err(e) => warn!("Failed to auto-load script {}: {}", script_name, e),
                }
            }
        }

        // Then load all .lua files in the directory
        if let Ok(entries) = std::fs::read_dir(&scripts_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "lua") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");

                    // Skip if already loaded via auto_load list
                    if self.config.auto_load.iter().any(|n| n.contains(name)) {
                        continue;
                    }

                    match self.load_script_file(&path) {
                        Ok(()) => count += 1,
                        Err(e) => warn!("Failed to auto-load script {}: {}", path.display(), e),
                    }
                }
            }
        }

        count
    }

    /// Set up the IRC API table available to scripts
    fn setup_api(&self, lua: &Lua) -> Result<()> {
        let irc = lua.create_table()?;

        // irc.print - log a message
        let print_fn = lua.create_function(|_, msg: String| {
            info!("[Script] {}", msg);
            Ok(())
        })?;
        irc.set("print", print_fn)?;

        // irc.send_message - send PRIVMSG
        let send_fn = lua.create_function(|_, (target, text): (String, String)| {
            info!("[Script] send_message({}, {})", target, text);
            Ok(())
        })?;
        irc.set("send_message", send_fn)?;

        // irc.join - join a channel
        let join_fn = lua.create_function(|_, channel: String| {
            info!("[Script] join({})", channel);
            Ok(())
        })?;
        irc.set("join", join_fn)?;

        // irc.part - leave a channel
        let part_fn = lua.create_function(|_, (channel, reason): (String, Option<String>)| {
            info!(
                "[Script] part({}, {:?})",
                channel,
                reason.as_deref().unwrap_or("")
            );
            Ok(())
        })?;
        irc.set("part", part_fn)?;

        // irc.get_var / irc.set_var - persistent variable storage
        let vars = self.variables.clone();
        let get_var = lua.create_function(move |_, key: String| {
            let result = if let Ok(vars) = vars.read() {
                vars.get(&key).cloned()
            } else {
                None
            };
            Ok(result)
        })?;
        irc.set("get_var", get_var)?;

        let vars = self.variables.clone();
        let set_var = lua.create_function(move |_, (key, value): (String, String)| {
            if let Ok(mut vars) = vars.write() {
                vars.insert(key, value);
            }
            Ok(())
        })?;
        irc.set("set_var", set_var)?;

        // irc.register_handler - register an event handler
        let register = lua.create_function(|lua, (event, handler): (String, Function)| {
            let handlers: mlua::Table = lua.globals().get("_handlers").unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                lua.globals().set("_handlers", t.clone()).unwrap();
                t
            });

            let event_handlers: mlua::Table = handlers.get(event.clone()).unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                handlers.set(event.clone(), t.clone()).unwrap();
                t
            });

            let count: i32 = event_handlers.len().unwrap_or(0) as i32;
            event_handlers.set(count + 1, handler)?;

            info!("[Script] Registered handler for: {}", event);
            Ok(())
        })?;
        irc.set("register_handler", register)?;

        // irc.command - register a custom command
        let command = lua.create_function(|lua, (cmd, handler): (String, Function)| {
            let commands: mlua::Table = lua.globals().get("_commands").unwrap_or_else(|_| {
                let t = lua.create_table().unwrap();
                lua.globals().set("_commands", t.clone()).unwrap();
                t
            });

            commands.set(cmd.clone(), handler)?;
            info!("[Script] Registered command: /{}", cmd);
            Ok(())
        })?;
        irc.set("command", command)?;

        lua.globals().set("irc", irc)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_engine_creation() {
        let engine = ScriptEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_from_config() {
        let config = ScriptingConfig::default();
        let engine = ScriptEngine::from_config(&config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_load_valid_script() {
        let engine = ScriptEngine::new().unwrap();
        let result = engine.load_script("test", "irc.print('Hello from test!')", 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_invalid_script() {
        let engine = ScriptEngine::new().unwrap();
        let result = engine.load_script("bad", "this is not valid lua }{}{", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_handlers() {
        let engine = ScriptEngine::new().unwrap();
        engine
            .load_script(
                "handler_test",
                r#"
                irc.register_handler("PRIVMSG", function(msg)
                    irc.print("Got message from: " .. (msg:get_nick() or "unknown"))
                end)
                "#,
                0,
            )
            .unwrap();

        let msg = ScriptMessage::new(
            Some("alice!user@host".to_string()),
            "PRIVMSG".to_string(),
            vec!["#test".to_string(), "Hello!".to_string()],
        );

        let result = engine.trigger_event("PRIVMSG", &msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_commands() {
        let engine = ScriptEngine::new().unwrap();
        engine
            .load_script(
                "cmd_test",
                r#"
                irc.command("hello", function(args)
                    irc.print("Hello command!")
                end)
                "#,
                0,
            )
            .unwrap();

        let handled = engine.execute_command("hello", vec![]).unwrap();
        assert!(handled);

        let handled = engine.execute_command("nonexistent", vec![]).unwrap();
        assert!(!handled);
    }

    #[test]
    fn test_sandbox_applied() {
        let engine = ScriptEngine::new().unwrap();
        let result = engine.load_script("sandbox_test", "io.open('/etc/passwd')", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_variables() {
        let engine = ScriptEngine::new().unwrap();
        engine
            .load_script(
                "var_test",
                r#"
                irc.set_var("test_key", "test_value")
                local val = irc.get_var("test_key")
                if val ~= "test_value" then
                    error("Variable mismatch: " .. tostring(val))
                end
                "#,
                0,
            )
            .unwrap();
    }

    #[test]
    fn test_priority_ordering() {
        let engine = ScriptEngine::new().unwrap();
        engine.load_script("low", "irc.print('low')", 10).unwrap();
        engine
            .load_script("high", "irc.print('high')", 100)
            .unwrap();
        engine.load_script("mid", "irc.print('mid')", 50).unwrap();

        let names = engine.list_scripts();
        assert_eq!(names, vec!["high", "mid", "low"]);
    }

    #[test]
    fn test_unload_script() {
        let engine = ScriptEngine::new().unwrap();
        engine
            .load_script("to_remove", "irc.print('hello')", 0)
            .unwrap();

        assert!(engine.unload_script("to_remove"));
        assert!(!engine.unload_script("nonexistent"));

        let scripts = engine.list_scripts();
        assert!(scripts.is_empty());
    }

    #[test]
    fn test_multiple_scripts() {
        let engine = ScriptEngine::new().unwrap();
        engine.load_script("s1", "irc.print('s1')", 0).unwrap();
        engine.load_script("s2", "irc.print('s2')", 0).unwrap();

        let scripts = engine.list_scripts();
        assert_eq!(scripts.len(), 2);
    }
}
