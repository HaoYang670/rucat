use rucat_common::{config::Args, error::Result};
use rucat_server::get_server;
use std::net::{Ipv6Addr, SocketAddrV6};
use tracing::info;

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    let Args { config_path } = Args::parse_args();
    // setup tracing
    tracing_subscriber::fmt::init();

    let endpoint = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 3000, 0, 0);
    let app = get_server(config_path.as_str()).await?;

    // run it
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    info!("Rucat server is listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
