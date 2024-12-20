pub mod k8s_client;

use ::rucat_common::{
    engine::{EngineConfigs, EngineId, EngineState},
    error::Result,
};

pub trait ResourceState {
    /// get the new engine state based on old engine state and resource state.
    /// if new state is same as old state, return None.
    fn get_new_engine_state(&self, old_state: &EngineState) -> Option<EngineState>;
}

#[allow(async_fn_in_trait)]
pub trait ResourceManager {
    type ResourceState: ResourceState;

    /// Create Engine and associated resources
    async fn create_resource(&self, id: &EngineId, config: &EngineConfigs) -> Result<()>;

    async fn get_resource_state(&self, id: &EngineId) -> Self::ResourceState;

    /// Remove all resources related to the Engine
    async fn clean_resource(&self, id: &EngineId) -> Result<()>;
}
