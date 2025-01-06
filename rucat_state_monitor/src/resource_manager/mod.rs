pub mod k8s_client;

use ::core::future::Future;

use ::rucat_common::{
    engine::{EngineId, EngineInfo, EngineState},
    error::Result,
};

pub trait ResourceState {
    /// get the new engine state based on old engine state and resource state.
    /// if new state is same as old state, return None.
    fn get_new_engine_state(&self, old_state: &EngineState) -> Option<EngineState>;
}

pub trait ResourceManager {
    type ResourceState: ResourceState;

    /// Create Engine and associated resources
    fn create_resource(&self, id: &EngineId, info: &EngineInfo)
        -> impl Future<Output = Result<()>>;

    fn get_resource_state(&self, id: &EngineId) -> impl Future<Output = Self::ResourceState>;

    /// Remove all resources related to the Engine
    fn clean_resource(&self, id: &EngineId) -> impl Future<Output = Result<()>>;
}
