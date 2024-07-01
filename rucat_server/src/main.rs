use std::net::{Ipv6Addr, SocketAddrV6};

use clap::Parser;
use rucat_common::error::Result;
use rucat_server::get_server;
use tracing::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// path to the config file
    #[arg(long)]
    config_path: String,
}

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    // setup tracing
    let Args { config_path } = Args::parse();
    tracing_subscriber::fmt::init();

    let endpoint = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 3000, 0, 0);
    let app = get_server(config_path.as_str()).await?;

    // run it
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    info!("Rucat server is listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
