use ::serde::{Deserialize, Serialize};
use ::std::{borrow::Cow, collections::BTreeMap};

mod engine_id;
mod engine_info;
mod engine_state;
mod engine_time;
mod engine_type;

pub use engine_id::EngineId;
pub use engine_info::EngineInfo;
pub use engine_state::EngineState;
pub use engine_time::EngineTime;
pub use engine_type::EngineType;

pub type EngineVersion = String;
pub type EngineConfig = BTreeMap<Cow<'static, str>, Cow<'static, str>>;

/// Request body to create an engine.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CreateEngineRequest {
    // The name of the engine
    pub name: String,
    pub engine_type: EngineType,
    pub version: EngineVersion,
    // Engine configurations
    pub config: Option<EngineConfig>,
}
