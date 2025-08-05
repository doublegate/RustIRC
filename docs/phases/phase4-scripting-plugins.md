# Phase 4: Scripting and Plugin System

**Duration**: 3-6 weeks  
**Goal**: Implement a powerful, safe, and extensible scripting and plugin system

## Overview

Phase 4 introduces the extensibility layer that will set RustIRC apart. We'll implement sandboxed scripting engines for both Lua and Python, along with a binary plugin API for high-performance extensions. The system will be inspired by mIRC's powerful scripting, HexChat's multi-language support, and WeeChat's comprehensive plugin architecture.

## Objectives

1. Embed Lua scripting engine with sandboxing
2. Embed Python scripting engine with sandboxing
3. Design and implement plugin API
4. Create script/plugin manager
5. Implement event system for extensions
6. Add custom command support
7. Ensure security and stability across all scripting languages

## Lua Scripting Engine

### Integration Architecture
```rust
// rustirc-scripting/src/lua_engine.rs
pub struct LuaEngine {
    lua: Lua,
    scripts: HashMap<ScriptId, LoadedScript>,
    event_handlers: HashMap<Event, Vec<ScriptId>>,
    sandboxed: bool,
}

pub struct LoadedScript {
    id: ScriptId,
    name: String,
    path: PathBuf,
    metadata: ScriptMetadata,
    state: ScriptState,
}

impl LuaEngine {
    pub fn new(sandboxed: bool) -> Result<Self> {
        let lua = Lua::new();
        
        if sandboxed {
            Self::apply_sandbox(&lua)?;
        }
        
        Self::register_api(&lua)?;
        
        Ok(Self {
            lua,
            scripts: HashMap::new(),
            event_handlers: HashMap::new(),
            sandboxed,
        })
    }
}
```

### Sandboxing
```rust
fn apply_sandbox(lua: &Lua) -> Result<()> {
    lua.context(|ctx| {
        // Remove dangerous functions
        let globals = ctx.globals();
        
        // File system restrictions
        globals.set("loadfile", mlua::Nil)?;
        globals.set("dofile", mlua::Nil)?;
        
        // OS restrictions
        let os = ctx.create_table()?;
        os.set("date", ctx.create_function(safe_os_date)?)?;
        os.set("time", ctx.create_function(safe_os_time)?)?;
        // Remove: execute, exit, remove, rename, etc.
        globals.set("os", os)?;
        
        // IO restrictions
        let io = ctx.create_table()?;
        // Only allow specific, safe IO operations
        globals.set("io", io)?;
        
        Ok(())
    })
}
```

### IRC API
```rust
fn register_api(lua: &Lua) -> Result<()> {
    let irc = lua.create_table()?;
    
    // Connection functions
    irc.set("connect", lua.create_function(api_connect)?)?;
    irc.set("disconnect", lua.create_function(api_disconnect)?)?;
    
    // Messaging functions
    irc.set("send", lua.create_function(api_send)?)?;
    irc.set("privmsg", lua.create_function(api_privmsg)?)?;
    irc.set("notice", lua.create_function(api_notice)?)?;
    irc.set("action", lua.create_function(api_action)?)?;
    
    // Channel functions
    irc.set("join", lua.create_function(api_join)?)?;
    irc.set("part", lua.create_function(api_part)?)?;
    irc.set("topic", lua.create_function(api_topic)?)?;
    
    // Event registration
    irc.set("on", lua.create_function(api_on_event)?)?;
    irc.set("off", lua.create_function(api_off_event)?)?;
    
    // UI functions
    irc.set("print", lua.create_function(api_print)?)?;
    irc.set("echo", lua.create_function(api_echo)?)?;
    irc.set("add_command", lua.create_function(api_add_command)?)?;
    
    // State queries
    irc.set("servers", lua.create_function(api_get_servers)?)?;
    irc.set("channels", lua.create_function(api_get_channels)?)?;
    irc.set("users", lua.create_function(api_get_users)?)?;
    
    lua.globals().set("irc", irc)?;
    Ok(())
}
```

### Event System
```lua
-- Example Lua script
local script = {
    name = "AutoGreet",
    version = "1.0.0",
    author = "RustIRC User",
    description = "Automatically greet users who join"
}

-- Event handler for user joins
function on_join(event)
    -- Don't greet ourselves
    if event.nick == irc.my_nick() then
        return
    end
    
    -- Check if it's a channel we want to greet in
    local greet_channels = {"#welcome", "#help"}
    for _, chan in ipairs(greet_channels) do
        if event.channel == chan then
            irc.privmsg(event.channel, 
                string.format("Welcome to %s, %s!", 
                    event.channel, event.nick))
            break
        end
    end
end

-- Register the event handler
irc.on("join", on_join)

-- Custom command
function cmd_greet(args)
    local target = args[1] or "everyone"
    irc.privmsg(irc.current_channel(), 
        string.format("Hello, %s!", target))
end

irc.add_command("greet", cmd_greet, "Greet someone")

return script
```

### Script Lifecycle
```rust
pub enum ScriptEvent {
    Load,
    Unload,
    Enable,
    Disable,
    Reload,
}

impl LuaEngine {
    pub fn load_script(&mut self, path: &Path) -> Result<ScriptId> {
        let id = ScriptId::new();
        let lua_code = std::fs::read_to_string(path)?;
        
        // Create isolated environment for script
        let env = self.create_script_env(id)?;
        
        // Load and execute script
        let script_table: Table = self.lua.context(|ctx| {
            ctx.load(&lua_code)
                .set_name(path.to_str().unwrap())?
                .set_environment(env)?
                .eval()
        })?;
        
        // Extract metadata
        let metadata = self.extract_metadata(&script_table)?;
        
        // Store script
        self.scripts.insert(id, LoadedScript {
            id,
            name: metadata.name.clone(),
            path: path.to_owned(),
            metadata,
            state: ScriptState::Loaded,
        });
        
        // Call load event
        self.trigger_script_event(id, ScriptEvent::Load)?;
        
        Ok(id)
    }
}
```

## Python Scripting Engine

### Python Integration with PyO3

```rust
// rustirc-scripting/src/python_engine.rs
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

pub struct PythonEngine {
    gil: GILPool,
    scripts: HashMap<ScriptId, LoadedPythonScript>,
    event_handlers: HashMap<Event, Vec<ScriptId>>,
    sandboxed: bool,
}

pub struct LoadedPythonScript {
    id: ScriptId,
    name: String,
    path: PathBuf,
    module: Py<PyModule>,
    metadata: ScriptMetadata,
}

impl PythonEngine {
    pub fn new(sandboxed: bool) -> Result<Self> {
        // Initialize Python interpreter
        pyo3::prepare_freethreaded_python();
        
        Python::with_gil(|py| {
            if sandboxed {
                Self::apply_sandbox(py)?;
            }
            
            Self::register_api(py)?;
            
            Ok(Self {
                gil: unsafe { GILPool::new() },
                scripts: HashMap::new(),
                event_handlers: HashMap::new(),
                sandboxed,
            })
        })
    }
}
```

### Python Sandboxing

```rust
fn apply_sandbox(py: Python) -> PyResult<()> {
    // Create restricted builtins
    let builtins = PyModule::import(py, "builtins")?;
    let restricted_builtins = PyDict::new(py);
    
    // Allow safe built-ins only
    let safe_builtins = [
        "True", "False", "None", "int", "float", "str", "list", 
        "dict", "set", "tuple", "len", "range", "enumerate",
        "zip", "map", "filter", "sorted", "min", "max", "sum",
        "abs", "round", "pow", "print", "type", "isinstance"
    ];
    
    for name in &safe_builtins {
        if let Ok(obj) = builtins.getattr(*name) {
            restricted_builtins.set_item(name, obj)?;
        }
    }
    
    // Create sandboxed environment
    let sandbox_code = r#"
import sys
import types

# Remove dangerous modules
dangerous_modules = [
    'os', 'subprocess', 'shutil', 'socket', 'http', 
    'urllib', 'requests', 'pathlib', 'tempfile'
]

for module in dangerous_modules:
    if module in sys.modules:
        del sys.modules[module]

# Custom import hook to restrict imports
class RestrictedImporter:
    def __init__(self, allowed_modules):
        self.allowed_modules = allowed_modules
    
    def find_module(self, fullname, path=None):
        if fullname not in self.allowed_modules:
            raise ImportError(f"Import of '{fullname}' is not allowed")
        return None

# Set up restricted import
allowed_modules = [
    're', 'json', 'datetime', 'time', 'math', 'random',
    'collections', 'itertools', 'functools', 'hashlib'
]

sys.meta_path.insert(0, RestrictedImporter(allowed_modules))
"#;
    
    py.run(sandbox_code, None, None)?;
    Ok(())
}
```

### Python IRC API

```rust
fn register_api(py: Python) -> PyResult<()> {
    let irc_module = PyModule::new(py, "irc")?;
    
    // Connection functions
    irc_module.add_function(wrap_pyfunction!(py_connect, irc_module)?)?;
    irc_module.add_function(wrap_pyfunction!(py_disconnect, irc_module)?)?;
    
    // Messaging functions  
    irc_module.add_function(wrap_pyfunction!(py_send_message, irc_module)?)?;
    irc_module.add_function(wrap_pyfunction!(py_send_notice, irc_module)?)?;
    irc_module.add_function(wrap_pyfunction!(py_send_action, irc_module)?)?;
    
    // Channel operations
    irc_module.add_function(wrap_pyfunction!(py_join, irc_module)?)?;
    irc_module.add_function(wrap_pyfunction!(py_part, irc_module)?)?;
    
    // State queries
    irc_module.add_function(wrap_pyfunction!(py_get_channels, irc_module)?)?;
    irc_module.add_function(wrap_pyfunction!(py_get_users, irc_module)?)?;
    
    // Add to Python sys.modules
    let sys = py.import("sys")?;
    let modules: &PyDict = sys.getattr("modules")?.downcast()?;
    modules.set_item("irc", irc_module)?;
    
    Ok(())
}

#[pyfunction]
fn py_send_message(server_id: u64, target: &str, message: &str) -> PyResult<()> {
    // Bridge to Rust IRC client
    Python::with_gil(|py| {
        py.allow_threads(|| {
            // Send through actual IRC connection
            Ok(())
        })
    })
}
```

### Python Script Loading

```rust
impl PythonEngine {
    pub fn load_script(&mut self, path: &Path) -> Result<ScriptId> {
        Python::with_gil(|py| {
            // Read script file
            let code = std::fs::read_to_string(path)?;
            
            // Create module for script
            let module = PyModule::new(py, &path.file_stem().unwrap().to_string_lossy())?;
            
            // Set up script globals
            let globals = module.dict();
            globals.set_item("__file__", path.to_str())?;
            
            // Execute script
            py.run(&code, Some(globals), None)?;
            
            // Extract metadata
            let metadata = self.extract_metadata(py, &module)?;
            
            // Register event handlers
            self.register_handlers(py, &module, script_id)?;
            
            let script = LoadedPythonScript {
                id: script_id,
                name: metadata.name.clone(),
                path: path.to_owned(),
                module: module.into(),
                metadata,
            };
            
            self.scripts.insert(script_id, script);
            Ok(script_id)
        })
    }
    
    fn extract_metadata(&self, py: Python, module: &PyModule) -> Result<ScriptMetadata> {
        let dict = module.dict();
        
        Ok(ScriptMetadata {
            name: dict.get_item("__name__")
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_else(|| "Unknown".to_string()),
            version: dict.get_item("__version__")
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_else(|| "1.0.0".to_string()),
            author: dict.get_item("__author__")
                .and_then(|v| v.extract::<String>().ok()),
            description: dict.get_item("__description__")
                .and_then(|v| v.extract::<String>().ok()),
        })
    }
}
```

### Python Event Handling

```rust
impl PythonEngine {
    pub fn handle_event(&mut self, event: &Event) -> Result<()> {
        Python::with_gil(|py| {
            let handlers = self.event_handlers.get(event).cloned().unwrap_or_default();
            
            for script_id in handlers {
                if let Some(script) = self.scripts.get(&script_id) {
                    let module = script.module.as_ref(py);
                    
                    // Convert event to Python args
                    let args = self.event_to_python(py, event)?;
                    
                    // Call appropriate handler function
                    let handler_name = match event {
                        Event::Message { .. } => "on_message",
                        Event::Join { .. } => "on_join",
                        Event::Part { .. } => "on_part",
                        // ... other events
                    };
                    
                    if let Ok(handler) = module.getattr(handler_name) {
                        // Call with timeout to prevent hanging
                        let result = py.allow_threads(|| {
                            with_timeout(Duration::from_secs(5), || {
                                handler.call1(args)
                            })
                        });
                        
                        if let Err(e) = result {
                            error!("Python script error in {}: {}", script.name, e);
                        }
                    }
                }
            }
            
            Ok(())
        })
    }
}
```

### Python Script Example

```python
"""example_bot.py - Example RustIRC Python script"""

__name__ = "Example Bot"
__version__ = "1.0.0"
__author__ = "Your Name"
__description__ = "A simple Python bot for RustIRC"

import irc
import re
import json
from datetime import datetime

# Configuration
config = {
    "greeting": "Hello",
    "auto_op": ["trusted_user1", "trusted_user2"],
    "log_messages": True
}

# State
seen_users = {}

def on_load():
    """Called when script is loaded"""
    print(f"{__name__} v{__version__} loaded!")
    
    # Register commands
    irc.register_command("seen", handle_seen_command, "Check when user was last seen")
    irc.register_command("config", handle_config_command, "Manage bot configuration")
    
    # Load saved state
    try:
        with open("seen_users.json", "r") as f:
            global seen_users
            seen_users = json.load(f)
    except FileNotFoundError:
        pass

def on_unload():
    """Called when script is unloaded"""
    # Save state
    with open("seen_users.json", "w") as f:
        json.dump(seen_users, f)
    
    print(f"{__name__} unloaded!")

def on_message(server_id, target, nick, message):
    """Handle incoming messages"""
    # Update seen database
    seen_users[nick.lower()] = {
        "time": datetime.now().isoformat(),
        "channel": target,
        "message": message
    }
    
    # Log if enabled
    if config["log_messages"]:
        with open(f"log_{target}.txt", "a") as f:
            f.write(f"[{datetime.now()}] <{nick}> {message}\n")
    
    # Respond to mentions
    if irc.current_nick(server_id).lower() in message.lower():
        irc.send_message(server_id, target, f"{nick}: {config['greeting']}!")

def on_join(server_id, channel, nick):
    """Handle user joins"""
    # Auto-op trusted users
    if nick in config["auto_op"]:
        irc.set_mode(server_id, channel, "+o", nick)

def handle_seen_command(args):
    """Handle !seen command"""
    if not args:
        return "Usage: !seen <nickname>"
    
    nick = args[0].lower()
    if nick in seen_users:
        info = seen_users[nick]
        last_seen = datetime.fromisoformat(info["time"])
        time_ago = datetime.now() - last_seen
        
        return f"{args[0]} was last seen {format_time_ago(time_ago)} in {info['channel']}: {info['message']}"
    else:
        return f"I haven't seen {args[0]}"

def handle_config_command(args):
    """Handle !config command"""
    if len(args) < 1:
        return "Usage: !config <get|set> [key] [value]"
    
    action = args[0]
    
    if action == "get":
        if len(args) < 2:
            return f"Available keys: {', '.join(config.keys())}"
        key = args[1]
        return f"{key} = {config.get(key, 'Not set')}"
    
    elif action == "set":
        if len(args) < 3:
            return "Usage: !config set <key> <value>"
        key = args[1]
        value = " ".join(args[2:])
        
        # Parse value type
        if value.lower() in ["true", "false"]:
            value = value.lower() == "true"
        elif value.isdigit():
            value = int(value)
        
        config[key] = value
        return f"Set {key} = {value}"

def format_time_ago(delta):
    """Format timedelta to human-readable string"""
    seconds = int(delta.total_seconds())
    
    if seconds < 60:
        return f"{seconds} seconds ago"
    elif seconds < 3600:
        return f"{seconds // 60} minutes ago"
    elif seconds < 86400:
        return f"{seconds // 3600} hours ago"
    else:
        return f"{seconds // 86400} days ago"
```

### Python Script Management

```rust
pub struct PythonScriptManager {
    engine: PythonEngine,
    script_dir: PathBuf,
    auto_load: bool,
}

impl PythonScriptManager {
    pub fn new(config: ScriptConfig) -> Result<Self> {
        let engine = PythonEngine::new(config.sandboxed)?;
        
        Ok(Self {
            engine,
            script_dir: config.script_dir,
            auto_load: config.auto_load,
        })
    }
    
    pub fn discover_scripts(&self) -> Result<Vec<ScriptInfo>> {
        let mut scripts = Vec::new();
        
        for entry in fs::read_dir(&self.script_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension() == Some(OsStr::new("py")) {
                // Extract script info without loading
                let info = self.get_script_info(&path)?;
                scripts.push(info);
            }
        }
        
        Ok(scripts)
    }
}
```

## Binary Plugin System

### Plugin API
```rust
// rustirc-plugin-api/src/lib.rs
#[repr(C)]
pub struct PluginInfo {
    pub api_version: u32,
    pub name: *const c_char,
    pub version: *const c_char,
    pub author: *const c_char,
    pub description: *const c_char,
}

#[repr(C)]
pub struct PluginContext {
    pub send_message: extern "C" fn(*const c_char, *const c_char),
    pub register_command: extern "C" fn(*const c_char, *const c_void),
    pub get_state: extern "C" fn() -> *const c_void,
    // ... more function pointers
}

pub type PluginInitFn = extern "C" fn(*const PluginContext) -> *const PluginInfo;
pub type PluginDeinitFn = extern "C" fn();
```

### Plugin Loader
```rust
// rustirc-plugins/src/loader.rs
use libloading::{Library, Symbol};

pub struct PluginLoader {
    plugins: HashMap<PluginId, LoadedPlugin>,
    context: PluginContext,
}

pub struct LoadedPlugin {
    id: PluginId,
    library: Library,
    info: PluginInfo,
    init_fn: PluginInitFn,
    deinit_fn: PluginDeinitFn,
}

impl PluginLoader {
    pub fn load_plugin(&mut self, path: &Path) -> Result<PluginId> {
        unsafe {
            let library = Library::new(path)?;
            
            // Get required symbols
            let init_fn: Symbol<PluginInitFn> = 
                library.get(b"plugin_init")?;
            let deinit_fn: Symbol<PluginDeinitFn> = 
                library.get(b"plugin_deinit")?;
            
            // Initialize plugin
            let info_ptr = init_fn(&self.context);
            let info = self.read_plugin_info(info_ptr)?;
            
            // Validate API version
            if info.api_version != PLUGIN_API_VERSION {
                return Err(Error::IncompatibleApiVersion);
            }
            
            let id = PluginId::new();
            self.plugins.insert(id, LoadedPlugin {
                id,
                library,
                info,
                init_fn: *init_fn,
                deinit_fn: *deinit_fn,
            });
            
            Ok(id)
        }
    }
}
```

### Example Plugin (Rust)
```rust
// example-plugin/src/lib.rs
use rustirc_plugin_api::*;
use std::ffi::CString;

static mut CONTEXT: Option<PluginContext> = None;

#[no_mangle]
pub extern "C" fn plugin_init(ctx: *const PluginContext) -> *const PluginInfo {
    unsafe {
        CONTEXT = Some(*ctx);
        
        // Register commands
        let cmd_name = CString::new("rainbow").unwrap();
        ((*ctx).register_command)(
            cmd_name.as_ptr(),
            rainbow_command as *const c_void
        );
    }
    
    &PLUGIN_INFO
}

#[no_mangle]
pub extern "C" fn plugin_deinit() {
    // Cleanup
}

extern "C" fn rainbow_command(args: *const c_char) {
    // Convert text to rainbow colors
    let text = unsafe { CStr::from_ptr(args).to_str().unwrap() };
    let rainbow = make_rainbow(text);
    
    unsafe {
        if let Some(ctx) = &CONTEXT {
            let msg = CString::new(rainbow).unwrap();
            let target = CString::new("#channel").unwrap();
            (ctx.send_message)(target.as_ptr(), msg.as_ptr());
        }
    }
}

static PLUGIN_INFO: PluginInfo = PluginInfo {
    api_version: 1,
    name: b"Rainbow Text\0".as_ptr() as *const c_char,
    version: b"1.0.0\0".as_ptr() as *const c_char,
    author: b"Example Author\0".as_ptr() as *const c_char,
    description: b"Rainbow text generator\0".as_ptr() as *const c_char,
};
```

## Script/Plugin Manager

### Manager UI
```rust
// rustirc-gui/src/dialogs/script_manager.rs
pub struct ScriptManager {
    scripts: Vec<ScriptEntry>,
    selected: Option<usize>,
    search_query: String,
    filter: ScriptFilter,
}

pub struct ScriptEntry {
    pub id: ExtensionId,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub status: ExtensionStatus,
    pub source: ExtensionSource,
}

pub enum ExtensionSource {
    Local(PathBuf),
    Repository(String),
    Bundled,
}
```

### Repository Integration
```rust
// rustirc-core/src/repository/mod.rs
pub struct ScriptRepository {
    base_url: String,
    cache_dir: PathBuf,
    index: RepositoryIndex,
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryIndex {
    pub scripts: Vec<ScriptMetadata>,
    pub plugins: Vec<PluginMetadata>,
    pub updated: DateTime<Utc>,
}

impl ScriptRepository {
    pub async fn fetch_index(&mut self) -> Result<()> {
        let response = reqwest::get(&format!("{}/index.json", self.base_url))
            .await?;
        self.index = response.json().await?;
        Ok(())
    }
    
    pub async fn install_script(&self, id: &str) -> Result<PathBuf> {
        let script = self.index.scripts.iter()
            .find(|s| s.id == id)
            .ok_or(Error::ScriptNotFound)?;
        
        let url = format!("{}/scripts/{}", self.base_url, script.file);
        let response = reqwest::get(&url).await?;
        let content = response.text().await?;
        
        let path = self.cache_dir.join(&script.file);
        std::fs::write(&path, content)?;
        
        Ok(path)
    }
}
```

## Event System Integration

### Event Types
```rust
pub enum ScriptableEvent {
    // Connection events
    Connected { server: String },
    Disconnected { server: String, reason: String },
    
    // Channel events
    Join { server: String, channel: String, nick: String },
    Part { server: String, channel: String, nick: String, reason: Option<String> },
    Kick { server: String, channel: String, nick: String, kicker: String, reason: Option<String> },
    
    // Message events
    Message { server: String, target: String, sender: String, text: String },
    Notice { server: String, target: String, sender: String, text: String },
    Action { server: String, target: String, sender: String, action: String },
    
    // User events
    NickChange { server: String, old_nick: String, new_nick: String },
    Quit { server: String, nick: String, reason: Option<String> },
    
    // UI events
    TabSwitch { old_tab: String, new_tab: String },
    Command { command: String, args: Vec<String> },
}
```

### Event Dispatcher
```rust
pub struct EventDispatcher {
    lua_handlers: HashMap<ScriptableEvent, Vec<LuaHandler>>,
    plugin_handlers: HashMap<ScriptableEvent, Vec<PluginHandler>>,
}

impl EventDispatcher {
    pub async fn dispatch(&self, event: ScriptableEvent) -> Result<()> {
        // Dispatch to Lua scripts
        if let Some(handlers) = self.lua_handlers.get(&event) {
            for handler in handlers {
                if let Err(e) = handler.call(&event).await {
                    error!("Lua handler error: {}", e);
                    // Don't stop other handlers
                }
            }
        }
        
        // Dispatch to plugins
        if let Some(handlers) = self.plugin_handlers.get(&event) {
            for handler in handlers {
                if let Err(e) = handler.call(&event).await {
                    error!("Plugin handler error: {}", e);
                }
            }
        }
        
        Ok(())
    }
}
```

## Security Considerations

### Script Permissions
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPermissions {
    pub network_access: bool,
    pub file_read: Option<Vec<PathBuf>>,
    pub file_write: Option<Vec<PathBuf>>,
    pub command_execution: bool,
    pub ui_modification: bool,
}

impl Default for ScriptPermissions {
    fn default() -> Self {
        Self {
            network_access: false,
            file_read: None,
            file_write: None,
            command_execution: false,
            ui_modification: true,
        }
    }
}
```

### Resource Limits
```rust
pub struct ResourceLimits {
    pub max_memory: usize,
    pub max_cpu_time: Duration,
    pub max_file_size: usize,
    pub max_open_files: usize,
}

impl LuaEngine {
    pub fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.lua.set_memory_limit(limits.max_memory);
        self.lua.set_gc_step_multiplier(100);
        // CPU time limits enforced via instruction counting
    }
}
```

## Testing

### Script Testing Framework
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lua_sandboxing() {
        let engine = LuaEngine::new(true).unwrap();
        
        // Should fail - no file access
        let result = engine.lua.load("os.execute('rm -rf /')").exec();
        assert!(result.is_err());
        
        // Should succeed - safe operation
        let result = engine.lua.load("return 1 + 1").eval::<i32>();
        assert_eq!(result.unwrap(), 2);
    }
    
    #[test]
    fn test_event_handling() {
        let mut engine = LuaEngine::new(true).unwrap();
        
        engine.load_script_string(r#"
            local messages = {}
            irc.on("message", function(event)
                table.insert(messages, event.text)
            end)
        "#).unwrap();
        
        engine.dispatch_event(ScriptableEvent::Message {
            server: "test".to_string(),
            target: "#test".to_string(),
            sender: "user".to_string(),
            text: "Hello".to_string(),
        }).unwrap();
        
        // Verify handler was called
    }
}
```

## Deliverables

By the end of Phase 4:

1. **Lua Scripting Engine**
   - Sandboxed execution
   - Comprehensive IRC API
   - Event handling system
   - Resource limits

2. **Plugin System**
   - Binary plugin loading
   - Stable ABI
   - Example plugins
   - Development SDK

3. **Script Manager**
   - GUI for script management
   - Repository integration
   - Auto-update support
   - Permission management

4. **Documentation**
   - Scripting API reference
   - Plugin development guide
   - Example scripts
   - Security guidelines

## Success Criteria

Phase 4 is complete when:
- [ ] Can load and execute Lua scripts safely
- [ ] Scripts can handle all major IRC events
- [ ] Binary plugins load and function correctly
- [ ] Script manager provides easy installation
- [ ] Security sandboxing is effective
- [ ] Performance impact is acceptable

## Next Phase

With extensibility in place, Phase 5 will add advanced IRC features including DCC support, IRCv3 extensions, and enhanced security features.