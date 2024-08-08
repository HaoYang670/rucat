use time::OffsetDateTime;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineState {
    /// Engine is pending to be started.
    Pending,
    /// Engine is running.
    Running,
    /// Engine is stopped.
    Stopped,
}

/// Ballista first on k8s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineType {
    /// Ballista in local mode
    BallistaLocal,
    /// Ballista in remote mode, e.g. on k8s.
    BallistaRemote,
    Rucat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    name: String,
    engine_type: EngineType,
    /// The address of the engine.
    // We don't define `endpoint` in `EngineState` because SurrealQL doesn't support pattern matching.`
    endpoint: Option<String>,
    state: EngineState,
    // Use String type but not OffsetDateTime to get a more readable response.
    created_time: String,
}

impl EngineInfo {
    pub fn new(
        name: String,
        engine_type: EngineType,
        state: EngineState,
        endpoint: Option<String>,
    ) -> Self {
        Self {
            name,
            engine_type,
            state,
            endpoint,
            created_time: OffsetDateTime::now_utc().to_string(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_engine_type(&self) -> &EngineType {
        &self.engine_type
    }

    pub fn get_state(&self) -> &EngineState {
        &self.state
    }
}
