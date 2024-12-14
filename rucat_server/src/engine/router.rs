//! Restful API for engine management.

use ::rucat_common::{
    anyhow::anyhow,
    database_client::DatabaseClient,
    engine::{EngineId, EngineInfo, EngineState::*, StartEngineRequest},
    error::{Result, RucatError},
    tracing::info,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::state::AppState;

/// start an engine with the given configuration
async fn start_engine<DB: DatabaseClient>(
    State(state): State<AppState<DB>>,
    Json(body): Json<StartEngineRequest>,
) -> Result<Json<EngineId>> {
    let id = state.get_db().add_engine(body).await?;
    info!("Creating engine {}, wait to start", id);
    // let success = k8s::create_engine(&id, &body.config).await;
    Ok(Json(id))
}

async fn delete_engine<DB: DatabaseClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()> {
    info!("Deleting engine {}", id);
    //TODO: get engine state first and update engine state correspondingly
    state
        .get_db()
        .delete_engine(&id, vec![])
        .await?
        .map(|_| ())
        .ok_or(RucatError::not_found(anyhow!("Engine {} not found", id)))?;
    Ok(())
}

/// Stop an engine to release resources. But engine info is still kept in the data store.
async fn stop_engine<DB: DatabaseClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()> {
    state
        .get_db()
        .update_engine_state(&id, vec![Running], WaitToTerminate)
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
        .update_engine_state(&id, vec![Terminated], WaitToStart)
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
        .route("/", post(start_engine::<DB>).get(list_engines::<DB>))
        .route("/:id", get(get_engine::<DB>).delete(delete_engine::<DB>))
        .route("/:id/stop", post(stop_engine::<DB>))
        .route("/:id/restart", post(restart_engine::<DB>))
}
