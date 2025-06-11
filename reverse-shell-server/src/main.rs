pub mod auth;
pub mod server;
pub mod types;

use anyhow::Result;
use clap::Parser;
use server::ReverseShellServer;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "reverse-shell-server")]
#[command(about = "A secure reverse shell server with WebSocket and SSL support")]
struct Args {
    #[arg(long, default_value = "8080")]
    client_port: u16,

    #[arg(long, default_value = "8081")]
    admin_port: u16,

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

    info!("Starting Reverse Shell Server");
    info!("Client port: {}", args.client_port);
    info!("Admin port: {}", args.admin_port);
    info!("Log level: {}", args.log_level);

    let server = ReverseShellServer::new(args.client_port, args.admin_port);
    server.start().await?;

    Ok(())
}