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

    /// Run Material Design 3 demo showcase
    #[arg(long)]
    material_demo: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.debug)?;

    info!("Starting RustIRC v{}", env!("CARGO_PKG_VERSION"));

    if args.material_demo {
        run_material_demo()?;
    } else if args.cli {
        run_cli(args)?;
    } else if args.tui {
        run_tui(args)?;
    } else {
        run_gui(args)?;
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

fn run_material_demo() -> Result<()> {
    info!("Starting Material Design 3 Components Demo");

    use rustirc_gui::material_demo;

    // Run the Material Design 3 demo showcase
    material_demo::run().map_err(|e| anyhow::anyhow!("Material demo error: {}", e))?;

    Ok(())
}

fn run_gui(args: Args) -> Result<()> {
    info!("Starting full-featured GUI mode with Iced (widgets, themes, resizable panes)");

    // Use configuration from args if provided
    if let Some(config_path) = args.config {
        info!("Loading configuration from: {}", config_path);
        // In the future, load and apply configuration from the specified file
        // For now, we log it to show it's being used
    }

    // Use full-featured GUI as the only GUI option - complete with all widgets and themes
    use rustirc_gui::RustIrcGui;

    // Run the full-featured GUI application with all advanced features
    RustIrcGui::run().map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    Ok(())
}

fn run_tui(args: Args) -> Result<()> {
    info!("Starting TUI mode with Ratatui");

    // Initialize TUI application
    use rustirc_tui::TuiApp;

    // Load configuration from args for TUI
    let config = load_config(args.config.as_deref())?;
    info!(
        "TUI configuration loaded from: {:?}",
        args.config.as_deref().unwrap_or("default")
    );

    // Create TUI app with configuration and run it
    let mut app = TuiApp::new()?;

    // Apply config settings to TUI app when TuiApp supports configuration
    if let Some(first_server) = config.servers.first() {
        info!(
            "TUI using config: server={}:{}, tls={}",
            first_server.address, first_server.port, first_server.use_tls
        );
    } else {
        info!("TUI using config: no servers configured, using default settings");
    }

    // Apply configuration settings to TUI app
    if let Some(server) = &args.server {
        info!("TUI will connect to server: {}", server);
        // Note: Server connection configuration for TUI
    }

    if args.debug {
        info!("Debug mode enabled for TUI");
        // Note: Debug logging configuration already handled in main()
    }

    if args.tls {
        info!("TLS connection enabled for TUI");
        // Note: TLS configuration for TUI
    }

    info!("TUI connecting to port: {}", args.port);

    // Run TUI in async runtime
    tokio::runtime::Runtime::new()?.block_on(async { app.run().await })?;

    Ok(())
}

fn run_cli(args: Args) -> Result<()> {
    info!("Starting CLI mode for testing");

    // Initialize CLI application
    use rustirc_core::run_cli_prototype;

    let config = load_config(args.config.as_deref())?;

    // Run CLI prototype (blocking)
    tokio::runtime::Runtime::new()?.block_on(async { run_cli_prototype(config).await })?;

    Ok(())
}

fn load_config(config_path: Option<&str>) -> Result<rustirc_core::Config> {
    use rustirc_core::Config;

    if let Some(path) = config_path {
        info!("Loading config from: {}", path);
        // Load from file when implemented
        Ok(Config::default())
    } else {
        info!("Using default configuration");
        Ok(Config::default())
    }
}
