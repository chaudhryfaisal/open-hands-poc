mod client;
mod shell;
mod types;

use anyhow::Result;
use clap::Parser;
use client::ReverseShellClient;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "reverse-shell-client")]
#[command(about = "A secure reverse shell client with WebSocket and SSL support")]
struct Args {
    #[arg(long, default_value = "localhost:8080")]
    server: String,

    #[arg(long, default_value = "client_token_123")]
    token: String,

    #[arg(long)]
    ssl: bool,

    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting Reverse Shell Client");
    info!("Server: {}", args.server);
    info!("SSL enabled: {}", args.ssl);
    info!("Log level: {}", args.log_level);

    let client = ReverseShellClient::new(args.server, args.token, args.ssl);
    client.start().await?;

    Ok(())
}