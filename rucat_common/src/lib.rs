//! Common types and utilities for the Rucat projects.

use serde::{Deserialize, Serialize};

pub mod config;
pub mod database;
pub mod engine;
pub mod error;

// re-export the dependencies
pub use k8s_openapi;
pub use kube;

pub mod engine_grpc {
    tonic::include_proto!("engine_grpc");
}
/// Unique identifier for an engine.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct EngineId {
    id: String,
}

impl EngineId {
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl From<String> for EngineId {
    fn from(id: String) -> Self {
        EngineId { id }
    }
}
