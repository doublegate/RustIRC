//! RustIRC - Modern IRC Client
//!
//! A powerful IRC client combining the best features of mIRC, HexChat, and WeeChat.

use anyhow::Result;
use clap::Parser;
use tracing::{info, warn};
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

    if args.cli {
        run_cli(args)?;
    } else if args.tui {
        run_tui(args)?;
    } else {
        run_gui(args)?;
    }

    Ok(())
}

fn init_logging(debug: bool) -> Result<()> {
    let filter = if debug {
        "debug"
    } else {
        "info"
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

fn run_gui(args: Args) -> Result<()> {
    info!("Starting GUI mode with Iced");
    
    // Use simplified GUI for now while complex widgets are being updated
    use rustirc_gui::SimpleRustIrcGui;
    
    // Run Iced GUI application - it uses its own static method
    SimpleRustIrcGui::run()
        .map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;
    
    Ok(())
}

fn run_tui(args: Args) -> Result<()> {
    info!("Starting TUI mode with Ratatui");
    
    // Initialize TUI application
    use rustirc_tui::TuiApp;
    
    // Create TUI app and run it
    let mut app = TuiApp::new()?;
    
    // Run TUI in async runtime
    tokio::runtime::Runtime::new()?.block_on(async {
        app.run().await
    })?;
    
    Ok(())
}

fn run_cli(args: Args) -> Result<()> {
    info!("Starting CLI mode for testing");
    
    // Initialize CLI application
    use rustirc_core::{Config, run_cli_prototype};
    
    let config = load_config(args.config.as_deref())?;
    
    // Run CLI prototype (blocking)
    tokio::runtime::Runtime::new()?.block_on(async {
        run_cli_prototype(config).await
    })?;
    
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