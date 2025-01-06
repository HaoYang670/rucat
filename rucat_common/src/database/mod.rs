//! Datastore to record engines' information

pub mod surrealdb_client;
use ::core::future::Future;

use crate::engine::{CreateEngineRequest, EngineId};
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

#[derive(Deserialize)]
pub struct EngineIdAndInfo {
    pub id: EngineId,
    pub info: EngineInfo,
}

/// Database for storing the Engine metadata.
/// Engine is stored in the format of using [EngineId] as key and [EngineInfo] as value.
pub trait Database: Sized + Send + Sync + 'static {
    /// Add the metadata of a new engine in the database,
    /// generate an id for the engine and return it.
    fn add_engine(
        &self,
        engine: CreateEngineRequest,
    ) -> impl Future<Output = Result<EngineId>> + Send;

    /// Remove Engine.
    /// # Return
    /// - `Ok(None)` if the engine does not exist.
    /// - `Ok(Some(UpdateEngineStateResponse))` if the engine exists.
    /// - `Err(_)` if any error occurs in the database.
    fn delete_engine(
        &self,
        id: &EngineId,
        current_state: &EngineState,
    ) -> impl Future<Output = Result<Option<UpdateEngineStateResponse>>> + Send;

    /// Update the engine state to `after` only when
    /// the engine exists and the current state is `before`.
    /// # Return
    /// - `Ok(None)` if the engine does not exist.
    /// - `Ok(Some(UpdateEngineStateResponse))` if the engine exists.
    /// - `Err(_)` if any error occurs in the database.
    fn update_engine_state(
        &self,
        id: &EngineId,
        before: &EngineState,
        after: &EngineState,
    ) -> impl Future<Output = Result<Option<UpdateEngineStateResponse>>> + Send;

    /// Return `Ok(None)` if the engine does not exist
    fn get_engine(&self, id: &EngineId) -> impl Future<Output = Result<Option<EngineInfo>>> + Send;

    /// Return a sorted list of all engine ids
    fn list_engines(&self) -> impl Future<Output = Result<Vec<EngineId>>> + Send;

    /// Return all engines that need to be updated.
    /// This includes engines in state `WaitTo*`,
    /// or those in `Running` and `*InProgress`, and the engine info has been outdated.
    fn list_engines_need_update(&self)
        -> impl Future<Output = Result<Vec<EngineIdAndInfo>>> + Send;
}
