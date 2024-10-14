//! Restful API for engine management.

use std::fmt::Debug;

use ::tracing::info;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rucat_common::{
    engine::{EngineInfo, EngineState::*, EngineType},
    error::{Result, RucatError},
    EngineId,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

use super::k8s;

impl From<CreateEngineRequest> for EngineInfo {
    fn from(value: CreateEngineRequest) -> Self {
        EngineInfo::new(value.name, value.engine_type, Pending, None)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateEngineRequest {
    name: String,
    engine_type: EngineType,
}

/// create an engine with the given configuration
async fn create_engine(
    State(state): State<AppState>,
    Json(body): Json<CreateEngineRequest>,
) -> Result<Json<EngineId>> {
    let id = state.get_db().add_engine(body.into()).await?;
    let success = k8s::create_engine(&id).await;
    // If fail to create the engine, delete the engine record from database.
    match success {
        Ok(()) => Ok(Json(id)),
        Err(e0) => {
            delete_engine(Path(id), State(state)).await.map_err(|e1| {
                RucatError::FailedToStartEngine(format!(
                    "Failed to start engine: {} and failed to clean up: {}",
                    e0, e1
                ))
            })?;
            Err(e0)
        }
    }
}

async fn delete_engine(Path(id): Path<EngineId>, State(state): State<AppState>) -> Result<()> {
    info!("Deleting engine {}", id.as_str());
    k8s::delete_engine(&id).await?;
    state
        .get_db()
        .delete_engine(&id)
        .await?
        .map(|_| ())
        .ok_or(RucatError::NotFoundError(id))
}

/// Stop an engine to release resources. But engine info is still kept in the data store.
async fn stop_engine(Path(id): Path<EngineId>, State(state): State<AppState>) -> Result<()> {
    state
        .get_db()
        // TODO: bring back Running state
        .update_engine_state(&id, [Pending, Running], Stopped, None)
        .await?
        .map_or_else(
            || Err(RucatError::NotFoundError(id.clone())),
            |response| {
                if response.update_success {
                    Ok(())
                } else {
                    Err(RucatError::NotAllowedError(format!(
                        "Engine {} is in {:?} state, cannot be stopped",
                        id.as_str(),
                        response.before_state
                    )))
                }
            },
        )
}

/// Restart a stopped engine with the same configuration.
async fn restart_engine(Path(id): Path<EngineId>, State(state): State<AppState>) -> Result<()> {
    state
        .get_db()
        .update_engine_state(&id, [Stopped], Pending, None)
        .await?
        .map_or_else(
            || Err(RucatError::NotFoundError(id.clone())),
            |response| {
                if response.update_success {
                    Ok(())
                } else {
                    Err(RucatError::NotAllowedError(format!(
                        "Engine {} is in {:?} state, cannot be restarted",
                        id.as_str(),
                        response.before_state
                    )))
                }
            },
        )
}

async fn get_engine(
    Path(id): Path<EngineId>,
    State(state): State<AppState>,
) -> Result<Json<EngineInfo>> {
    state
        .get_db()
        .get_engine(&id)
        .await?
        .map(Json)
        .ok_or(RucatError::NotFoundError(id))
}

async fn list_engines(State(state): State<AppState>) -> Result<Json<Vec<EngineId>>> {
    state.get_db().list_engines().await.map(Json)
}

/// Pass the data store endpoint later
pub(crate) fn get_engine_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_engine).get(list_engines))
        .route("/:id", get(get_engine).delete(delete_engine))
        .route("/:id/stop", post(stop_engine))
        .route("/:id/restart", post(restart_engine))
}
