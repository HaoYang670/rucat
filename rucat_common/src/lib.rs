//! Common types and utilities for the Rucat projects.

pub mod config;
pub mod database;
pub mod engine;
pub mod error;

// re-export the dependencies
pub use anyhow;
pub use k8s_openapi;
pub use kube;
pub use serde;
pub use serde_json;
pub use tracing;
pub use tracing_subscriber;

pub mod client_grpc {
    tonic::include_proto!("client_grpc");
}
