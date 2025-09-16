use anyhow::Result;
use clap::Parser;
use drift::{config::Config, server::Server};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "drift")]
#[command(about = "A modern, high-performance OCI Registry + Web UI")]
struct Cli {
    #[arg(short, long, default_value = "drift.toml")]
    config: String,

    #[arg(short, long, default_value = "0.0.0.0:5000")]
    bind: String,

    #[arg(short, long, default_value = "0.0.0.0:5001")]
    ui_bind: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "drift=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    info!("🌊 Starting Drift Registry");
    info!("📦 OCI-compatible registry for Bolt, Docker, and Podman");

    // Load configuration
    let config = Config::load(&cli.config).unwrap_or_else(|_| {
        warn!("Could not load config file, using defaults");
        Config::default()
    });

    info!("🚀 Registry API starting on {}", cli.bind);
    info!("🖥️  Web UI starting on {}", cli.ui_bind);

    // Create and start server
    let server = Server::new(config, &cli.bind, &cli.ui_bind).await?;
    server.run().await?;

    Ok(())
}