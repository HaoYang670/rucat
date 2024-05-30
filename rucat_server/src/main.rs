use std::net::{Ipv4Addr, SocketAddrV4};

use rucat_common::error::Result;
use rucat_server::get_server;
use tracing::info;

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt::init();

    let endpoint = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000);
    let app = get_server(true).await?;

    // run it
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    info!("Rucat server is listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
