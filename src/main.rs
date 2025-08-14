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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(args.debug)?;

    info!("Starting RustIRC v{}", env!("CARGO_PKG_VERSION"));

    if args.tui {
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

fn run_gui(_args: Args) -> Result<()> {
    info!("Starting GUI mode with Iced");
    
    // GUI implementation will be completed in Phase 3
    warn!("GUI mode is not yet implemented (coming in Phase 3)");
    warn!("Use --tui flag to run in terminal mode");
    
    Ok(())
}

fn run_tui(_args: Args) -> Result<()> {
    info!("Starting TUI mode with Ratatui");
    
    // TUI implementation will be completed in Phase 3
    warn!("TUI mode is not yet implemented (coming in Phase 3)");
    
    Ok(())
}