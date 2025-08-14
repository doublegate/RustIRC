use anyhow::Result;
use mlua::{Function, Lua, MetaMethod, UserData, UserDataMethods, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
struct IrcMessage {
    prefix: Option<String>,
    command: String,
    params: Vec<String>,
}

impl UserData for IrcMessage {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get_prefix", |_, this, ()| Ok(this.prefix.clone()));
        methods.add_method("get_command", |_, this, ()| Ok(this.command.clone()));
        methods.add_method("get_params", |_, this, ()| Ok(this.params.clone()));
        methods.add_method("get_channel", |_, this, ()| {
            // First param is often the channel for PRIVMSG
            Ok(this.params.first().cloned())
        });
        methods.add_method("get_text", |_, this, ()| {
            // Last param is often the message text
            Ok(this.params.last().cloned())
        });
        methods.add_method("get_nick", |_, this, ()| {
            // Extract nick from prefix (nick!user@host)
            Ok(this.prefix.as_ref().and_then(|p| {
                p.split('!').next().map(String::from)
            }))
        });
    }
}

#[derive(Clone)]
struct IrcClient {
    scripts: Arc<RwLock<Vec<Script>>>,
    variables: Arc<RwLock<HashMap<String, String>>>,
}

struct Script {
    name: String,
    lua: Lua,
    priority: i32,
}

impl IrcClient {
    fn new() -> Self {
        Self {
            scripts: Arc::new(RwLock::new(Vec::new())),
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn load_script(&self, name: &str, code: &str, priority: i32) -> Result<()> {
        let lua = Lua::new();
        
        // Set up IRC API
        self.setup_api(&lua).await?;
        
        // Load the script
        lua.load(code).exec()?;
        
        let script = Script {
            name: name.to_string(),
            lua,
            priority,
        };
        
        let mut scripts = self.scripts.write().await;
        scripts.push(script);
        scripts.sort_by_key(|s| -s.priority); // Higher priority first
        
        info!("Loaded script: {} (priority: {})", name, priority);
        Ok(())
    }

    async fn setup_api(&self, lua: &Lua) -> Result<()> {
        let irc = lua.create_table()?;
        
        // irc.print function
        let print_fn = lua.create_function(|_, msg: String| {
            info!("[Script] {}", msg);
            Ok(())
        })?;
        irc.set("print", print_fn)?;
        
        // irc.send_message function
        let send_fn = lua.create_function(|_, (target, text): (String, String)| {
            info!("[Script] Would send to {}: {}", target, text);
            Ok(())
        })?;
        irc.set("send_message", send_fn)?;
        
        // irc.get_var and irc.set_var for persistent storage
        let vars = self.variables.clone();
        let get_var = lua.create_async_function(move |_, key: String| {
            let vars = vars.clone();
            async move {
                let vars = vars.read().await;
                Ok(vars.get(&key).cloned())
            }
        })?;
        irc.set("get_var", get_var)?;
        
        let vars = self.variables.clone();
        let set_var = lua.create_async_function(move |_, (key, value): (String, String)| {
            let vars = vars.clone();
            async move {
                let mut vars = vars.write().await;
                vars.insert(key, value);
                Ok(())
            }
        })?;
        irc.set("set_var", set_var)?;
        
        // irc.register_handler function
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
        
        // irc.command function for creating aliases
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

    async fn trigger_event(&self, event: &str, msg: &IrcMessage) -> Result<()> {
        let scripts = self.scripts.read().await;
        
        for script in scripts.iter() {
            if let Ok(handlers) = script.lua.globals().get::<mlua::Table>("_handlers") {
                if let Ok(event_handlers) = handlers.get::<mlua::Table>(event) {
                    for pair in event_handlers.pairs::<i32, Function>() {
                        if let Ok((_, handler)) = pair {
                            match handler.call::<IrcMessage>(msg.clone()) {
                                Ok(()) => {}
                                Err(e) => {
                                    error!("Script error in {}: {}", script.name, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn execute_command(&self, cmd: &str, args: Vec<String>) -> Result<()> {
        let scripts = self.scripts.read().await;
        
        for script in scripts.iter() {
            if let Ok(commands) = script.lua.globals().get::<mlua::Table>("_commands") {
                if let Ok(handler) = commands.get::<Function>(cmd) {
                    match handler.call::<Vec<String>>(args.clone()) {
                        Ok(()) => return Ok(()),
                        Err(e) => {
                            error!("Command error in {}: {}", script.name, e);
                        }
                    }
                }
            }
        }
        
        warn!("Unknown command: /{}", cmd);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("rustirc_scripting_prototype=info")
        .init();

    info!("RustIRC Scripting Prototype");
    info!("===========================");
    
    let client = IrcClient::new();
    
    // Load a sample script with event handlers
    let script1 = r#"
        -- Auto-op script
        irc.register_handler("JOIN", function(msg)
            local nick = msg:get_nick()
            local channel = msg:get_channel()
            
            if nick == "friend" then
                irc.print("Auto-opping " .. nick .. " in " .. channel)
                irc.send_message("ChanServ", "OP " .. channel .. " " .. nick)
            end
        end)
        
        -- Greeting script
        irc.register_handler("JOIN", function(msg)
            local nick = msg:get_nick()
            local channel = msg:get_channel()
            
            if nick ~= "RustIRC" then  -- Don't greet ourselves
                irc.send_message(channel, "Welcome to " .. channel .. ", " .. nick .. "!")
            end
        end)
        
        -- Message logger
        irc.register_handler("PRIVMSG", function(msg)
            local nick = msg:get_nick()
            local channel = msg:get_channel()
            local text = msg:get_text()
            
            irc.print(string.format("[%s] <%s> %s", channel, nick, text))
            
            -- Store last message
            irc.set_var("last_msg_" .. channel, nick .. ": " .. text)
        end)
    "#;
    
    client.load_script("auto_features", script1, 100).await?;
    
    // Load a command script
    let script2 = r#"
        -- Custom commands
        irc.command("hello", function(args)
            irc.print("Hello from Lua!")
            irc.send_message("#test", "Hello everyone!")
        end)
        
        irc.command("calc", function(args)
            local expr = table.concat(args, " ")
            local fn = load("return " .. expr)
            if fn then
                local ok, result = pcall(fn)
                if ok then
                    irc.print("Result: " .. tostring(result))
                else
                    irc.print("Error: " .. tostring(result))
                end
            else
                irc.print("Invalid expression")
            end
        end)
        
        irc.command("lastmsg", function(args)
            local channel = args[1] or "#test"
            local last = irc.get_var("last_msg_" .. channel)
            if last then
                irc.print("Last message in " .. channel .. ": " .. last)
            else
                irc.print("No messages logged for " .. channel)
            end
        end)
    "#;
    
    client.load_script("commands", script2, 50).await?;
    
    // Simulate some IRC events
    info!("\n--- Simulating IRC Events ---");
    
    // Simulate JOIN
    let join_msg = IrcMessage {
        prefix: Some("friend!user@host".to_string()),
        command: "JOIN".to_string(),
        params: vec!["#rust".to_string()],
    };
    client.trigger_event("JOIN", &join_msg).await?;
    
    // Simulate PRIVMSG
    let privmsg = IrcMessage {
        prefix: Some("alice!alice@example.com".to_string()),
        command: "PRIVMSG".to_string(),
        params: vec!["#rust".to_string(), "Hello, world!".to_string()],
    };
    client.trigger_event("PRIVMSG", &privmsg).await?;
    
    // Test commands
    info!("\n--- Testing Commands ---");
    client.execute_command("hello", vec![]).await?;
    client.execute_command("calc", vec!["2".to_string(), "+".to_string(), "2".to_string()]).await?;
    client.execute_command("lastmsg", vec!["#rust".to_string()]).await?;
    
    // Test script sandboxing
    info!("\n--- Testing Sandboxing ---");
    let bad_script = r#"
        -- This should fail - no file I/O allowed
        irc.register_handler("TEST", function(msg)
            local f = io.open("/etc/passwd", "r")
            irc.print("This shouldn't work")
        end)
    "#;
    
    match client.load_script("bad_script", bad_script, 1).await {
        Ok(_) => warn!("Script loaded but io operations should fail when called"),
        Err(e) => info!("Script properly rejected: {}", e),
    }
    
    info!("\n--- Scripting Prototype Complete ---");
    
    Ok(())
}