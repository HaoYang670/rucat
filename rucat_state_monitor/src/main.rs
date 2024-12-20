use ::rucat_common::{
    config::{load_config, DatabaseConfig},
    database_client::surrealdb_client::SurrealDBClient,
    error::Result,
    tokio,
    tracing::info,
    tracing_subscriber,
};
use ::rucat_state_monitor::{
    config::{StateMonitorConfig, CONFIG_FILE_PATH},
    resource_manager::k8s_client::K8sClient,
    run_state_monitor,
};

// TODO: Convert the return type to `Result<!>` when it's stable
// See <https://github.com/rust-lang/rust/issues/35121>
#[tokio::main]
async fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt::init();
    info!("Start rucat state monitor");

    let StateMonitorConfig {
        check_interval_millis,
        database: DatabaseConfig { credentials, uri },
    } = load_config(CONFIG_FILE_PATH)?;

    let db_client = SurrealDBClient::new(credentials.as_ref(), uri).await?;
    let resource_manager = K8sClient::new().await?;

    run_state_monitor(db_client, resource_manager, check_interval_millis.get()).await
}
