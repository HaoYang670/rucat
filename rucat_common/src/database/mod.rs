//! Datastore to record engines' information

pub mod surrealdb_client;
use ::core::future::Future;
use ::std::time::SystemTime;

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
    /// # Parameters
    /// - `engine`: create engine request
    /// - `next_update_time`: The time when the engine should be updated by the state monitor.
    ///                       `None` means the engine does not need to be updated anymore.
    /// # Return
    /// - `Ok(EngineId)` if the engine is successfully added.
    /// - `Err(_)` if any error occurs in the database.
    fn add_engine(
        &self,
        engine: CreateEngineRequest,
        next_update_time: Option<SystemTime>,
    ) -> impl Future<Output = Result<EngineId>> + Send;

    /// Remove Engine.
    /// # Return
    /// - `Ok(None)` if the engine does not exist.
    /// - `Ok(Some(UpdateEngineStateResponse))` if the engine exists.
    /// - `Err(_)` if any error occurs in the database.
    fn remove_engine(
        &self,
        id: &EngineId,
        current_state: &EngineState,
    ) -> impl Future<Output = Result<Option<UpdateEngineStateResponse>>> + Send;

    /// Update the engine state to `after` only when
    /// the engine exists and the current state is `before`.
    /// # Parameters
    /// - `id`: The id of the engine.
    /// - `before`: The expected state of the engine before the update.
    /// - `after`: The state that engine is wanted to be updated to.
    /// - `next_update_time`: The time when the engine should be updated by the state monitor.
    ///                              `None` means the engine does not need to be updated anymore.
    /// # Return
    /// - `Ok(None)` if the engine does not exist.
    /// - `Ok(Some(UpdateEngineStateResponse))` if the engine exists.
    /// - `Err(_)` if any error occurs in the database.
    fn update_engine_state(
        &self,
        id: &EngineId,
        before: &EngineState,
        after: &EngineState,
        next_update_time: Option<SystemTime>,
    ) -> impl Future<Output = Result<Option<UpdateEngineStateResponse>>> + Send;

    /// Return `Ok(None)` if the engine does not exist
    fn get_engine(&self, id: &EngineId) -> impl Future<Output = Result<Option<EngineInfo>>> + Send;

    /// Return a sorted list of all engine ids
    fn list_engines(&self) -> impl Future<Output = Result<Vec<EngineId>>> + Send;

    /// Return all out-of-date engines that need to be updated.
    fn list_engines_need_update(&self)
        -> impl Future<Output = Result<Vec<EngineIdAndInfo>>> + Send;
}
