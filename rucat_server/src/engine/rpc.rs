//! RPC between server and engine.
//!
//! Create a new process for the engine and start the gRPC server.
//! The engine will be listening on the given port. (localhost for now)

use std::process::Stdio;

use rucat_common::{
    config::EngineConfig,
    error::{Result, RucatError},
};

use tokio::{io::AsyncWriteExt, process::Command};

/// Create a new process for the engine and start the gRPC server.
/// The engine will be listening on a random port.
pub(super) async fn create_engine(engine_binary_path: &str, config: EngineConfig) -> Result<()> {
    // Start the engine process.
    let mut engine = Command::new(engine_binary_path)
        .stdin(Stdio::piped())
        .spawn()?;

    // Send the configuration to the engine.
    // TODO: write to stdin directly (serde_json::to_writer)
    match engine.stdin {
        Some(mut stdin) => {
            stdin.write_all(&serde_json::to_vec(&config)?).await?;
            stdin.flush().await?;
            // The creation is async and the engine will be started in the background.
            // do not wait for the engine to start.
            // TODO: engines may meet all kinds of error before the rpc server starts
            // How to catch them?
            Ok(())
        }
        None => {
            let stdin_err_msg = "Failed to open engine's stdin";
            // kill the engine process if failed to open stdin
            let err = match engine.kill().await {
                Ok(_) => RucatError::FailedToStartEngine(stdin_err_msg.to_string()),
                Err(e) => RucatError::FailedToStartEngine(format!(
                    "{} and failed to kill the engine process: {}",
                    stdin_err_msg, e,
                )),
            };
            Err(err)
        }
    }
}
