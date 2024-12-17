//! Datastore to record engines' information

pub mod surrealdb_client;

use axum::async_trait;

use crate::engine::{EngineId, StartEngineRequest};
use crate::engine::{EngineInfo, EngineState};
use crate::error::Result;
use serde::Deserialize;

/// Response of updating an engine state.
/// # Fields
/// - `before_state`: The engine state before the update.
/// - `update_success`: Whether the update is successful.
#[derive(Deserialize)]
pub struct UpdateEngineStateResponse {
    pub before_state: EngineState,
    pub update_success: bool,
}

/// Client of the database to store the Engine.
/// Engine is stored in the format of using [EngineId] as key and [EngineInfo] as value.
// TODO: replace #[async_trait] by #[trait_variant::make(HttpService: Send)] in the future: https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html#should-i-still-use-the-async_trait-macro
#[async_trait]
pub trait DatabaseClient: Sized + Send + Sync + 'static {
    /// Add the metadata of a new engine in the database,
    /// generate an id for the engine and return it.
    async fn add_engine(&self, engine: StartEngineRequest) -> Result<EngineId>;

    /// Remove Engine.
    /// # Return
    /// - `Ok(None)` if the engine does not exist.
    /// - `Ok(Some(UpdateEngineStateResponse))` if the engine exists.
    /// - `Err(_)` if any error occurs in the database.
    async fn delete_engine(
        &self,
        id: &EngineId,
        current_state: &EngineState,
    ) -> Result<Option<UpdateEngineStateResponse>>;

    /// Update the engine state to `after` only when
    /// the engine exists and the current state is `before`.
    /// # Return
    /// - `Ok(None)` if the engine does not exist.
    /// - `Ok(Some(UpdateEngineStateResponse))` if the engine exists.
    /// - `Err(_)` if any error occurs in the database.
    async fn update_engine_state(
        &self,
        id: &EngineId,
        before: &EngineState,
        after: &EngineState,
    ) -> Result<Option<UpdateEngineStateResponse>>;

    /// Return `Ok(None)` if the engine does not exist
    async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>>;

    /// Return a sorted list of all engine ids
    async fn list_engines(&self) -> Result<Vec<EngineId>>;

    /// Return all engines that need to be updated.
    /// This includes engines in state `WaitTo*`,
    /// or those in `Running` and `*InProgress`, and the engine info has been outdated.
    async fn list_engines_need_update(&self) -> Result<Vec<(EngineId, EngineInfo)>>;
}
