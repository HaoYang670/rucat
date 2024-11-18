use ::rucat_common::{
    anyhow::anyhow,
    config::{load_config, DatabaseConfig, DatabaseVariant},
    database::{surrealdb_client::SurrealDBClient, DatabaseClient},
    error::{Result, RucatError},
    tokio,
    tracing::{debug, info},
    tracing_subscriber,
};
use ::rucat_state_monitor::{StateMonitorConfig, CONFIG_FILE_PATH};

// TODO: Convert the return type to `Result<!>` when it's stable
// See <https://github.com/rust-lang/rust/issues/35121>
#[tokio::main]
async fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt::init();
    info!("Start rucat state monitor");

    let StateMonitorConfig {
        check_interval_millis,
        database: DatabaseConfig {
            credentials,
            variant,
        },
    } = load_config(CONFIG_FILE_PATH)?;

    // TODO: find a better way to forbid using embedded database
    match variant {
        DatabaseVariant::Embedded => Err(RucatError::fail_to_start_state_monitor(anyhow!(
            "Cannot use embedded database."
        ))),
        DatabaseVariant::Local { uri } => {
            let db = SurrealDBClient::connect_local_db(credentials.as_ref(), uri).await?;
            loop {
                let engines = db.list_engines().await?;
                debug!("Detect {} Spark engines", engines.len());
                info!("Checking Spark state...");
                // wait for some seconds
                std::thread::sleep(std::time::Duration::from_millis(
                    check_interval_millis.get(),
                ));
            }
        }
    }
}
