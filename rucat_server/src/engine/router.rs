//! Restful API for engine management.

use ::std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;

use ::rucat_common::{
    anyhow::anyhow,
    database::DatabaseClient,
    engine::{EngineId, EngineInfo, EngineState::*},
    error::{Result, RucatError},
    serde::{Deserialize, Serialize},
    tracing::{error, info},
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::state::AppState;

use super::k8s;

/// Request body to create an engine.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(crate = "rucat_common::serde")]
pub(crate) struct CreateEngineRequest {
    // The name of the engine
    name: String,
    // Spark configurations
    configs: Option<HashMap<Cow<'static, str>, Cow<'static, str>>>,
}

impl TryFrom<CreateEngineRequest> for EngineInfo {
    type Error = RucatError;

    fn try_from(value: CreateEngineRequest) -> Result<Self> {
        Ok(EngineInfo::new(
            value.name,
            Pending,
            value.configs.unwrap_or_default().try_into()?,
        ))
    }
}

/// create an engine with the given configuration
async fn create_engine<DB: DatabaseClient>(
    State(state): State<AppState<DB>>,
    Json(body): Json<CreateEngineRequest>,
) -> Result<Json<EngineId>> {
    let body: EngineInfo = body.try_into()?;
    let id = state.get_db().add_engine(body.clone()).await?;
    info!("Creating engine {}", id);
    let success = k8s::create_engine(&id, &body.config).await;
    // If fail to create the engine, delete the engine record from database.
    match success {
        Ok(()) => {
            info!("Engine {} created", id);
            Ok(Json(id))
        }
        Err(e0) => {
            error!(
                "Failed to create engine {}, start to clean the resource",
                id
            );
            delete_engine(Path(id), State(state)).await.map_err(|e1| {
                RucatError::fail_to_create_engine(anyhow!(
                    "Failed to start engine: {:?} and failed to clean up: {:?}",
                    e0,
                    e1
                ))
            })?;
            Err(e0)
        }
    }
}

async fn delete_engine<DB: DatabaseClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()> {
    info!("Deleting engine {}", id);
    state
        .get_db()
        .delete_engine(&id)
        .await?
        .map(|_| ())
        .ok_or(RucatError::not_found(anyhow!("Engine {} not found", id)))?;
    k8s::delete_engine(&id).await
}

/// Stop an engine to release resources. But engine info is still kept in the data store.
async fn stop_engine<DB: DatabaseClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()> {
    state
        .get_db()
        .update_engine_state(&id, [Pending, Running], Stopped)
        .await?
        .map_or_else(
            || Err(RucatError::engine_not_found(&id)),
            |response| {
                if response.update_success {
                    Ok(())
                } else {
                    Err(RucatError::not_allowed(anyhow!(
                        "Engine {} is in {:?} state, cannot be stopped",
                        id,
                        response.before_state
                    )))
                }
            },
        )
}

/// Restart a stopped engine with the same configuration.
async fn restart_engine<DB: DatabaseClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()> {
    state
        .get_db()
        .update_engine_state(&id, [Stopped], Pending)
        .await?
        .map_or_else(
            || Err(RucatError::engine_not_found(&id)),
            |response| {
                if response.update_success {
                    Ok(())
                } else {
                    Err(RucatError::not_allowed(anyhow!(
                        "Engine {} is in {:?} state, cannot be restarted",
                        id,
                        response.before_state
                    )))
                }
            },
        )
}

async fn get_engine<DB: DatabaseClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<Json<EngineInfo>> {
    state
        .get_db()
        .get_engine(&id)
        .await?
        .map(Json)
        .ok_or(RucatError::engine_not_found(&id))
}

async fn list_engines<DB: DatabaseClient>(
    State(state): State<AppState<DB>>,
) -> Result<Json<Vec<EngineId>>> {
    state.get_db().list_engines().await.map(Json)
}

/// Pass the data store endpoint later
pub(crate) fn get_engine_router<DB: DatabaseClient>() -> Router<AppState<DB>> {
    Router::new()
        .route("/", post(create_engine::<DB>).get(list_engines::<DB>))
        .route("/:id", get(get_engine::<DB>).delete(delete_engine::<DB>))
        .route("/:id/stop", post(stop_engine::<DB>))
        .route("/:id/restart", post(restart_engine::<DB>))
}
