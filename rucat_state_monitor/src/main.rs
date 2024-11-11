use ::rucat_common::{tracing::info, tracing_subscriber};

fn main() {
    // setup tracing
    tracing_subscriber::fmt::init();
    info!("Start rucat state monitor");

    loop {
        info!("Checking Spark state...");
        // wait for some seconds
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
