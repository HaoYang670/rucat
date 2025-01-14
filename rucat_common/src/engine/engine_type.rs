use ::serde::{Deserialize, Serialize};

/// Type of engine.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum EngineType {
    Spark,
}
