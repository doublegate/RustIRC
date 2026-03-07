//! Script sandbox for security
//!
//! Provides sandboxing for Lua scripts to prevent access to dangerous
//! system functions like file I/O, process execution, and network access.

use mlua::Lua;
use rustirc_core::config::ScriptingConfig;

/// Security sandbox for Lua scripts
pub struct Sandbox {
    memory_limit: usize,
    timeout_ms: u64,
}

impl Sandbox {
    pub fn new(memory_limit: usize, timeout_ms: u64) -> Self {
        Self {
            memory_limit,
            timeout_ms,
        }
    }

    pub fn from_config(config: &ScriptingConfig) -> Self {
        Self::new(config.sandbox_memory_limit, config.sandbox_timeout_ms)
    }

    /// Apply sandbox restrictions to a Lua instance
    pub fn apply(&self, lua: &Lua) -> mlua::Result<()> {
        // Set memory limit
        if self.memory_limit > 0 {
            lua.set_memory_limit(self.memory_limit)?;
        }

        // Remove dangerous modules and functions
        let globals = lua.globals();

        // Remove io module entirely
        globals.set("io", mlua::Value::Nil)?;

        // Remove debug module
        globals.set("debug", mlua::Value::Nil)?;

        // Remove loadfile/dofile/require
        globals.set("loadfile", mlua::Value::Nil)?;
        globals.set("dofile", mlua::Value::Nil)?;
        globals.set("require", mlua::Value::Nil)?;

        // Restrict os module to safe subset (time-related functions only)
        let safe_os = lua.create_table()?;
        if let Ok(os_table) = globals.get::<mlua::Table>("os") {
            if let Ok(clock) = os_table.get::<mlua::Function>("clock") {
                safe_os.set("clock", clock)?;
            }
            if let Ok(date) = os_table.get::<mlua::Function>("date") {
                safe_os.set("date", date)?;
            }
            if let Ok(difftime) = os_table.get::<mlua::Function>("difftime") {
                safe_os.set("difftime", difftime)?;
            }
            if let Ok(time) = os_table.get::<mlua::Function>("time") {
                safe_os.set("time", time)?;
            }
        }
        globals.set("os", safe_os)?;

        // Set instruction count hook for CPU timeout
        if self.timeout_ms > 0 {
            let max_instructions = self.timeout_ms * 1000;
            let _ = lua.set_hook(
                mlua::HookTriggers::new().every_nth_instruction(10000),
                move |_lua, _debug| {
                    static COUNTER: std::sync::atomic::AtomicU64 =
                        std::sync::atomic::AtomicU64::new(0);
                    let count = COUNTER.fetch_add(10000, std::sync::atomic::Ordering::Relaxed);
                    if count > max_instructions {
                        COUNTER.store(0, std::sync::atomic::Ordering::Relaxed);
                        Err(mlua::Error::RuntimeError(
                            "Script execution timeout exceeded".to_string(),
                        ))
                    } else {
                        Ok(mlua::VmState::Continue)
                    }
                },
            );
        }

        Ok(())
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new(100 * 1024 * 1024, 5000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_io() {
        let lua = Lua::new();
        let sandbox = Sandbox::default();
        sandbox.apply(&lua).unwrap();

        let result = lua.load("io.open('/etc/passwd', 'r')").exec();
        assert!(result.is_err());
    }

    #[test]
    fn test_blocks_os_functions() {
        let lua = Lua::new();
        let sandbox = Sandbox::default();
        sandbox.apply(&lua).unwrap();

        // os.remove should be blocked
        let result = lua.load("os.remove('/tmp/test')").exec();
        assert!(result.is_err());
    }

    #[test]
    fn test_blocks_loadfile() {
        let lua = Lua::new();
        let sandbox = Sandbox::default();
        sandbox.apply(&lua).unwrap();

        let result = lua.load("loadfile('/etc/passwd')").exec();
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_os_works() {
        let lua = Lua::new();
        let sandbox = Sandbox::default();
        sandbox.apply(&lua).unwrap();

        // os.time() should still work
        let result = lua.load("return os.time()").eval::<i64>();
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_config() {
        let config = ScriptingConfig::default();
        let sandbox = Sandbox::from_config(&config);
        assert_eq!(sandbox.memory_limit, config.sandbox_memory_limit);
        assert_eq!(sandbox.timeout_ms, config.sandbox_timeout_ms);
    }
}
