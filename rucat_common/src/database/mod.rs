//! Datastore to record engines' information

pub mod surrealdb_client;

use axum::async_trait;

use crate::engine::EngineId;
use crate::error::Result;
use crate::{
    config::Credentials,
    engine::{EngineInfo, EngineState},
};
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
    /// get URI of the database
    fn get_uri(&self) -> &str;

    /// get credentials of the database
    fn get_credentials(&self) -> Option<&Credentials>;

    /// Connect to an existing local database, return the client.
    async fn connect_local_db(credentials: Option<&Credentials>, uri: String) -> Result<Self>;

    /// Add the metadata of an Engine in the database,
    /// generate an id for the Engine and return it.
    async fn add_engine(&self, engine: EngineInfo) -> Result<EngineId>;

    /// Remove Engine. Return `Ok(None)` if the Engine
    /// does not exist, otherwise return the metadata.
    async fn delete_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>>;

    /// Update the engine state to `after` only when
    /// the engine exists and the current state is in `before`.
    /// # Return
    /// - `Ok(None)` if the engine does not exist.
    /// - `Ok(Some(UpdateEngineStateResponse))` if the engine exists.
    /// - `Err(_)` if any error occurs in the database.
    /// TODO: convert `before` to `[EngineState; N]` when mockall supports const generics.
    async fn update_engine_state(
        &self,
        id: &EngineId,
        before: Vec<EngineState>,
        after: EngineState,
    ) -> Result<Option<UpdateEngineStateResponse>>;

    /// Return `Ok(None)` if the engine does not exist
    async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>>;

    /// Return a sorted list of all engine ids
    async fn list_engines(&self) -> Result<Vec<EngineId>>;
}
