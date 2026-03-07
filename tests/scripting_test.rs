//! Integration tests for Lua scripting engine

use rustirc_core::config::ScriptingConfig;
use rustirc_scripting::{ScriptEngine, ScriptMessage};

#[test]
fn test_engine_from_config_and_load() {
    let config = ScriptingConfig {
        enable: true,
        scripts_path: "scripts".to_string(),
        auto_load: vec![],
        sandbox_memory_limit: 50 * 1024 * 1024,
        sandbox_timeout_ms: 3000,
    };

    let engine = ScriptEngine::from_config(&config).unwrap();
    engine
        .load_script("test", "irc.print('integration test')", 0)
        .unwrap();

    let scripts = engine.list_scripts();
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0], "test");
}

#[test]
fn test_script_event_handler_fires() {
    let engine = ScriptEngine::new().unwrap();
    engine
        .load_script(
            "handler",
            r##"
        _result = "not fired"
        irc.register_handler("PRIVMSG", function(msg)
            _result = msg:get_nick() .. " said " .. msg:get_text()
        end)
    "##,
            0,
        )
        .unwrap();

    let msg = ScriptMessage::new(
        Some("alice!user@host".to_string()),
        "PRIVMSG".to_string(),
        vec!["#channel".to_string(), "Hello!".to_string()],
    );

    engine.trigger_event("PRIVMSG", &msg).unwrap();
}

#[test]
fn test_script_command_execution() {
    let engine = ScriptEngine::new().unwrap();
    engine
        .load_script(
            "commands",
            r##"
        irc.command("greet", function(args)
            irc.print("Hello " .. (args[1] or "world"))
        end)
    "##,
            0,
        )
        .unwrap();

    assert!(engine
        .execute_command("greet", vec!["Alice".to_string()])
        .unwrap());
    assert!(!engine.execute_command("unknown", vec![]).unwrap());
}

#[test]
fn test_sandbox_blocks_dangerous_operations() {
    let engine = ScriptEngine::new().unwrap();

    // io module should be blocked
    assert!(engine
        .load_script("bad_io", "io.open('/etc/passwd')", 0)
        .is_err());

    // require should be blocked
    assert!(engine
        .load_script("bad_require", "require('os')", 0)
        .is_err());

    // dofile should be blocked
    assert!(engine
        .load_script("bad_dofile", "dofile('/etc/passwd')", 0)
        .is_err());
}

#[test]
fn test_script_variables_persist() {
    let engine = ScriptEngine::new().unwrap();
    engine
        .load_script(
            "setter",
            r##"
        irc.set_var("key1", "value1")
    "##,
            0,
        )
        .unwrap();

    // Load another script that reads the variable
    engine
        .load_script(
            "getter",
            r##"
        local v = irc.get_var("key1")
        if v ~= "value1" then
            error("Expected 'value1' but got '" .. tostring(v) .. "'")
        end
    "##,
            0,
        )
        .unwrap();
}

#[test]
fn test_multiple_scripts_priority() {
    let engine = ScriptEngine::new().unwrap();
    engine.load_script("low", "irc.print('low')", 10).unwrap();
    engine
        .load_script("high", "irc.print('high')", 100)
        .unwrap();
    engine.load_script("mid", "irc.print('mid')", 50).unwrap();

    let scripts = engine.list_scripts();
    assert_eq!(scripts, vec!["high", "mid", "low"]);
}

#[test]
fn test_script_message_methods() {
    let engine = ScriptEngine::new().unwrap();
    engine
        .load_script(
            "msg_test",
            r##"
        irc.register_handler("JOIN", function(msg)
            local nick = msg:get_nick()
            local channel = msg:get_channel()
            local cmd = msg:get_command()
            local prefix = msg:get_prefix()

            if nick ~= "bob" then error("bad nick: " .. tostring(nick)) end
            if channel ~= "#rust" then error("bad channel: " .. tostring(channel)) end
            if cmd ~= "JOIN" then error("bad command: " .. tostring(cmd)) end
        end)
    "##,
            0,
        )
        .unwrap();

    let msg = ScriptMessage::new(
        Some("bob!user@host".to_string()),
        "JOIN".to_string(),
        vec!["#rust".to_string()],
    );
    engine.trigger_event("JOIN", &msg).unwrap();
}
