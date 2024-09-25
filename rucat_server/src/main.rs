use rucat_common::{config::Args, error::Result};
use rucat_server::get_server;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    process::Child,
};
use tokio::signal;
use tracing::info;

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt::init();

    let Args { config_path } = Args::parse_args();
    let endpoint = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000);
    let (app, embedded_db_ps) = get_server(config_path.as_str()).await?;

    // run it
    let listener = tokio::net::TcpListener::bind(endpoint).await?;
    info!("Rucat server is listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(embedded_db_ps))
        .await?;

    Ok(())
}

async fn shutdown_signal(embedded_db_ps: Option<Child>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
        info!("Ctrl+C is pressed, shutting down...");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
        info!("Terminate signal received, shutting down...");
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    if let Some(mut ps) = embedded_db_ps {
        info!("Terminating embedded database process...");
        ps.kill().expect("failed to kill embedded database process");
    }
}
