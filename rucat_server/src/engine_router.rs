use std::fmt::{Debug, Display};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rucat_common::error::{Result, RucatError};
use serde::{Deserialize, Serialize};

use crate::state::{data_store::DataStore, AppState};
use EngineState::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum EngineState {
    /// Engine is pending to be started.
    Pending,
    /// Engine is running.
    Running,
    /// Engine is stopped.
    Stopped,
}

impl Display for EngineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pending => write!(f, "Pending"),
            Running => write!(f, "Running"),
            Stopped => write!(f, "Stopped"),
        }
    }
}

/// Ballista first on k8s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum EngineType {
    /// Ballista in local mode
    BallistaLocal,
    /// Ballista in remote mode, e.g. on k8s.
    BallistaRemote,
    Rucat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EngineInfo {
    name: String,
    engine_type: EngineType,
    state: EngineState,
}

impl EngineInfo {
    pub(crate) fn update_state(mut self, state: EngineState) -> Self {
        self.state = state;
        self
    }
}

impl From<CreateEngineRequest> for EngineInfo {
    fn from(value: CreateEngineRequest) -> Self {
        EngineInfo {
            name: value.name,
            engine_type: value.engine_type,
            state: Pending,
        }
    }
}

pub(crate) type EngineId = String;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateEngineRequest {
    name: String,
    engine_type: EngineType,
}

/// create an engine with the given configuration
async fn create_engine(
    State(state): State<AppState<'_>>,
    Json(body): Json<CreateEngineRequest>,
) -> Result<EngineId> {
    state.get_data_store().add_engine(body.into()).await
}

async fn delete_engine(Path(id): Path<EngineId>, State(state): State<AppState<'_>>) -> Result<()> {
    state
        .get_data_store()
        .delete_engine(&id)
        .await?
        .map(|_| ())
        .ok_or_else(|| engine_not_found(&id))
}

/// Stop an engine to release resources. But engine info is still kept in the data store.
/// TODO: make the state checking and updating atomic
async fn stop_engine(Path(id): Path<EngineId>, State(state): State<AppState<'_>>) -> Result<()> {
    let data_store = state.get_data_store();

    let engine = get_engine_helper(data_store, &id).await?;

    match engine.state {
        Pending | Running => {
            data_store
                .update_engine(&id, engine.update_state(Stopped))
                .await
        }
        Stopped => RucatError::NotAllowedError(format!("Engine {} is already stopped", &id)).into(),
    }
}

/// Restart a stopped engine with the same configuration.
/// TODO: make the state checking and updating atomic
async fn restart_engine(Path(id): Path<EngineId>, State(state): State<AppState<'_>>) -> Result<()> {
    let data_store = state.get_data_store();
    let engine = get_engine_helper(data_store, &id).await?;

    match engine.state {
        Stopped => {
            data_store
                .update_engine(&id, engine.update_state(Pending))
                .await
        }
        other => RucatError::NotAllowedError(format!(
            "Engine {} is in {} state, cannot be restart",
            &id, other
        ))
        .into(),
    }
}

async fn get_engine(
    Path(id): Path<EngineId>,
    State(state): State<AppState<'_>>,
) -> Result<Json<EngineInfo>> {
    state
        .get_data_store()
        .get_engine(&id)
        .await?
        .map(Json)
        .ok_or_else(|| engine_not_found(&id))
}

async fn list_engines(State(state): State<AppState<'_>>) -> Result<Json<Vec<EngineId>>> {
    state.get_data_store().list_engines().await.map(Json)
}

/// Pass the data store endpoint later
pub(crate) fn get_engine_router() -> Router<AppState<'static>> {
    Router::new()
        .route("/", post(create_engine).get(list_engines))
        .route("/:id", get(get_engine).delete(delete_engine))
        .route("/:id/stop", post(stop_engine))
        .route("/:id/restart", post(restart_engine))
}

// ----------------- helper functions -----------------

/// helper function to create a NotFoundError
fn engine_not_found(id: &EngineId) -> RucatError {
    RucatError::NotFoundError(format!("Engine {} not found", id))
}

async fn get_engine_helper(data_store: &DataStore<'_>, id: &EngineId) -> Result<EngineInfo> {
    data_store
        .get_engine(id)
        .await?
        .ok_or_else(|| engine_not_found(id))
}
