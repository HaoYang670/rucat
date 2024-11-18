use ::rucat_common::{
    database::surrealdb_client::SurrealDBClient, error::RucatError, tokio, tracing::info,
    tracing_subscriber,
};
use rucat_common::{config::Args, error::Result};
use rucat_server::get_server;
use std::net::{Ipv4Addr, SocketAddrV4};

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt::init();

    let Args { config_path } = Args::parse_args();
    let endpoint = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000);
    let app = get_server::<SurrealDBClient>(config_path.as_str()).await?;

    // run it
    let listener = tokio::net::TcpListener::bind(endpoint)
        .await
        .map_err(RucatError::fail_to_start_server)?;
    info!(
        "Rucat server is listening on {}",
        listener
            .local_addr()
            .map_err(RucatError::fail_to_start_server)?
    );
    axum::serve(listener, app)
        .await
        .map_err(RucatError::fail_to_start_server)
}
