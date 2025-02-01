use ::rucat_common::{
    config::{load_config, DatabaseVariant},
    database::surrealdb_client::SurrealDBClient,
    error::RucatError,
    tokio,
    tracing::info,
    tracing_subscriber,
};
use ::rucat_server::{
    authentication::static_auth_provider::StaticAuthProvider, get_server,
    AuthProviderVariant::StaticAuthProviderConfig, ServerConfig,
};
use rucat_common::{config::Args, error::Result};
use std::net::{Ipv4Addr, SocketAddrV4};

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt::init();

    let Args { config_path } = Args::parse_args();
    let endpoint = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000);
    let ServerConfig {
        auth_provider,
        database: DatabaseVariant::Surreal { credentials, uri },
    } = load_config(&config_path)?;

    let db_client = SurrealDBClient::new(credentials.as_ref(), uri).await?;
    let app = match auth_provider {
        None => {
            info!("Authentication is disabled");
            get_server(db_client, None::<StaticAuthProvider>)?
        }
        Some(StaticAuthProviderConfig {
            username,
            password,
            bearer_token,
        }) => {
            info!("Static authentication is enabled");
            let auth_provider = StaticAuthProvider::new(username, password, bearer_token);
            get_server(db_client, Some(auth_provider))?
        }
    };

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
