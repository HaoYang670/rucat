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
    state: EngineState,
}

impl EngineInfo {
    pub fn new(name: String, engine_type: EngineType, state: EngineState) -> Self {
        Self {
            name,
            engine_type,
            state,
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
