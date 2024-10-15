//! Common types and utilities for the Rucat projects.

use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub mod config;
pub mod database;
pub mod engine;
pub mod error;

// re-export the dependencies
pub use k8s_openapi;
pub use kube;
pub use anyhow;

pub mod engine_grpc {
    tonic::include_proto!("engine_grpc");
}
/// Unique identifier for an engine.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct EngineId {
    id: String,
}

impl EngineId {
    pub fn new(id: String) -> Self {
        EngineId { id }
    }
}

impl Display for EngineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl From<String> for EngineId {
    fn from(id: String) -> Self {
        EngineId { id }
    }
}
