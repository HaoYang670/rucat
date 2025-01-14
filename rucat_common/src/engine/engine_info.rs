use super::*;
use crate::{
    engine::EngineState::WaitToStart,
    error::{Result, RucatError},
};
use ::serde::{Deserialize, Serialize};

/// Whole information of an engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineInfo {
    pub name: String,
    pub engine_type: EngineType,
    pub version: EngineVersion,
    pub state: EngineState,
    pub config: EngineConfig,
    /// time when the engine is created.
    /// Note, this is **not** the start time when the engine is RUNNING.
    create_time: EngineTime,
}

impl EngineInfo {
    pub fn new(
        name: String,
        engine_type: EngineType,
        version: EngineVersion,
        state: EngineState,
        config: EngineConfig,
        create_time: EngineTime,
    ) -> Self {
        Self {
            name,
            engine_type,
            version,
            state,
            config,
            create_time,
        }
    }
}

impl TryFrom<CreateEngineRequest> for EngineInfo {
    type Error = RucatError;

    fn try_from(value: CreateEngineRequest) -> Result<Self> {
        Ok(EngineInfo::new(
            value.name,
            value.engine_type,
            value.version,
            WaitToStart,
            value.config.unwrap_or_default(),
            EngineTime::now(),
        ))
    }
}
