//! RPC between server and engine.
//!
//! Create a new process for the engine and start the gRPC server.
//! The engine will be listening on the given port. (localhost for now)

use rucat_common::{
    engine_grpc::{greeter_client::GreeterClient, HelloRequest},
    error::{Result, RucatError},
};

use tokio::process::Command;
use tracing::info;

/// Create a new process for the engine and start the gRPC server.
/// The engine will be listening on the given port. (localhost for now)
pub(super) async fn create_engine(engine_binary_path: &str, port: u16) -> Result<()> {
    // Start the engine process.
    Command::new(engine_binary_path)
        .args(["--ip", "::1", "--port", port.to_string().as_str()])
        .spawn()?;

    // TODO: better way to wait for the engine to start.
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let mut client = GreeterClient::connect(format!("http://[::1]:{}", port))
        .await
        .map_err(|e| RucatError::FailedToStartEngine(e.to_string()))?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client
        .say_hello(request)
        .await
        .map_err(|e| RucatError::FailedToStartEngine(e.to_string()))?;

    info!("RESPONSE={:?}", response.into_inner().message);

    Ok(())
}
