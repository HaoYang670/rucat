//! Common types and utilities for the Rucat projects.

pub mod config;
pub mod database;
pub mod engine;
pub mod error;

pub mod engine_grpc {
    tonic::include_proto!("engine_grpc");
}
/// Unique identifier for an engine.
pub type EngineId = String;
