use ::rucat_common::{
    config::{load_config, DatabaseConfig},
    error::Result,
    tracing::info,
    tracing_subscriber,
};
use ::rucat_state_monitor::{StateMonitorConfig, CONFIG_FILE_PATH};

// TODO: Convert the return type to `Result<!>` when it's stable
// See <https://github.com/rust-lang/rust/issues/35121>
fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt::init();
    info!("Start rucat state monitor");

    let StateMonitorConfig {
        check_interval_millis,
        database: DatabaseConfig {
            credentials: _,
            variant: _,
        },
    } = load_config(CONFIG_FILE_PATH)?;

    loop {
        info!("Checking Spark state...");
        // wait for some seconds
        std::thread::sleep(std::time::Duration::from_millis(
            check_interval_millis.get(),
        ));
    }
}
