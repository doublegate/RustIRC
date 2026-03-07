//! RustIRC - Modern IRC Client
//!
//! A powerful IRC client combining the best features of mIRC, HexChat, and WeeChat.

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "rustirc")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in TUI mode instead of GUI
    #[arg(long)]
    tui: bool,

    /// Server to connect to
    #[arg(short, long)]
    server: Option<String>,

    /// Port to connect to
    #[arg(short, long, default_value = "6667")]
    port: u16,

    /// Use TLS for connection
    #[arg(long)]
    tls: bool,

    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Run in CLI mode for testing
    #[arg(long)]
    cli: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.debug)?;

    info!("Starting RustIRC v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = load_config(args.config.as_deref())?;

    if args.cli {
        run_cli(config)?;
    } else if args.tui {
        run_tui(args, config)?;
    } else {
        run_gui(config)?;
    }

    Ok(())
}

fn init_logging(debug: bool) -> Result<()> {
    let filter = if debug { "debug" } else { "info" };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

fn run_gui(config: rustirc_core::Config) -> Result<()> {
    info!("Starting GUI mode with Dioxus");

    // Initialize scripting and plugins
    let _script_engine = init_scripting(&config);
    let _plugin_manager = init_plugins(&config);

    // Launch the Dioxus desktop GUI with config
    rustirc_gui::run_gui_with_config(config).map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    Ok(())
}

fn run_tui(args: Args, config: rustirc_core::Config) -> Result<()> {
    info!("Starting TUI mode with Ratatui");

    use rustirc_tui::TuiApp;

    info!(
        "TUI configuration loaded from: {:?}",
        args.config.as_deref().unwrap_or("default")
    );

    let mut app = TuiApp::new()?;

    if let Some(first_server) = config.servers.first() {
        info!(
            "TUI using config: server={}:{}, tls={}",
            first_server.address, first_server.port, first_server.use_tls
        );
    } else {
        info!("TUI using config: no servers configured, using default settings");
    }

    if let Some(server) = &args.server {
        info!("TUI will connect to server: {}", server);
    }

    if args.debug {
        info!("Debug mode enabled for TUI");
    }

    if args.tls {
        info!("TLS connection enabled for TUI");
    }

    info!("TUI connecting to port: {}", args.port);

    // Run TUI in async runtime
    tokio::runtime::Runtime::new()?.block_on(async { app.run().await })?;

    Ok(())
}

fn run_cli(config: rustirc_core::Config) -> Result<()> {
    info!("Starting CLI mode for testing");

    use rustirc_core::run_cli_prototype;

    // Run CLI prototype (blocking)
    tokio::runtime::Runtime::new()?.block_on(async { run_cli_prototype(config).await })?;

    Ok(())
}

fn load_config(config_path: Option<&str>) -> Result<rustirc_core::Config> {
    use rustirc_core::Config;

    if let Some(path) = config_path {
        info!("Loading config from: {}", path);
        Config::from_file(path).map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))
    } else {
        Ok(Config::load_or_default())
    }
}

fn init_scripting(config: &rustirc_core::Config) -> Option<rustirc_scripting::ScriptEngine> {
    if !config.scripting.enable {
        info!("Scripting disabled in configuration");
        return None;
    }

    match rustirc_scripting::ScriptEngine::from_config(&config.scripting) {
        Ok(engine) => {
            // Auto-load scripts from configured path
            let scripts_loaded = engine.auto_load_scripts();
            info!(
                "Script engine initialized, loaded {} scripts",
                scripts_loaded
            );
            Some(engine)
        }
        Err(e) => {
            tracing::warn!("Failed to initialize script engine: {}", e);
            None
        }
    }
}

fn init_plugins(config: &rustirc_core::Config) -> rustirc_plugins::PluginManager {
    let mut manager = rustirc_plugins::PluginManager::new();

    // Register built-in plugins
    use rustirc_plugins::builtin::{HighlightPlugin, LoggerPlugin};

    let log_path = config.logging.path.clone();
    let _ = manager.register_plugin(Box::new(LoggerPlugin::new(log_path)));
    let _ = manager.register_plugin(Box::new(HighlightPlugin::new(
        config.notifications.highlight_words.clone(),
    )));

    info!(
        "Plugin manager initialized with {} plugins",
        manager.list_plugins().len()
    );
    manager
}
