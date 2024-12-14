//! Common types and utilities for the Rucat projects.

pub mod config;
pub mod database_client;
pub mod engine;
pub mod error;
pub mod client_grpc {
    tonic::include_proto!("client_grpc");
}

// re-export the dependencies for other crates to use
pub use anyhow;
pub use k8s_openapi;
pub use kube;
pub use serde;
pub use serde_json;
pub use tokio;
pub use tracing;
pub use tracing_subscriber;
