use std::net::{Ipv6Addr, SocketAddrV6};

use clap::Parser;
use rucat_common::error::Result;
use rucat_server::get_server;
use tracing::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path to the engine binary (local)
    #[arg(long)]
    engine_binary_path: String,
}

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    let Args { engine_binary_path } = Args::parse();
    // setup tracing
    tracing_subscriber::fmt::init();

    let endpoint = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 3000, 0, 0);
    let app = get_server(true, engine_binary_path).await?;

    // run it
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    info!("Rucat server is listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
