use ::rucat_common::{
    config::{load_config, DatabaseVariant},
    database::surrealdb_client::SurrealDBClient,
    error::Result,
    tokio,
    tracing::info,
    tracing_subscriber,
};
use ::rucat_state_monitor::{
    config::{StateMonitorConfig, CONFIG_FILE_PATH},
    resource_manager::k8s_client::K8sClient,
    StateMonitor,
};

// TODO: Convert the return type to `Result<!>` when it's stable
// See <https://github.com/rust-lang/rust/issues/35121>
#[tokio::main]
async fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt::init();
    info!("Start rucat state monitor");

    let StateMonitorConfig {
        check_interval_secs,
        trigger_state_timeout_secs,
        database: DatabaseVariant::Surreal { credentials, uri },
    } = load_config(CONFIG_FILE_PATH)?;

    let db_client = SurrealDBClient::new(credentials.as_ref(), uri).await?;
    let resource_manager = K8sClient::new().await?;
    let state_monitor = StateMonitor::new(
        db_client,
        resource_manager,
        check_interval_secs,
        trigger_state_timeout_secs,
    );
    state_monitor.run_state_monitor().await
}
