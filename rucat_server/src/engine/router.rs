//! Restful API for engine management.

use ::rucat_common::{
    anyhow::anyhow,
    database_client::DatabaseClient,
    engine::{
        EngineId, EngineInfo,
        EngineState::{self, *},
        StartEngineRequest,
    },
    error::{Result, RucatError},
    tracing::{debug, info},
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
    Ok(Json(id))
}

async fn delete_engine<DBClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DBClient>>,
) -> Result<()>
where
    DBClient: DatabaseClient,
{
    let db_client = state.get_db();
    let mut current_state = get_engine_state(&id, db_client).await?;

    loop {
        match current_state {
            s @ (WaitToStart | Terminated | ErrorClean(_)) => {
                let response = db_client
                    .delete_engine(&id, &s)
                    .await?
                    .ok_or_else(|| RucatError::engine_not_found(&id))?;
                if response.update_success {
                    info!("Engine {} is deleted", id);
                    return Ok(());
                }
                debug!(
                    "Engine {} has been updated from {:?} to {:?}, retry to delete",
                    id, s, response.before_state
                );
                current_state = response.before_state;
            }
            other => {
                return Err(RucatError::not_allowed(anyhow!(
                    "Engine {} is in {:?} state, cannot be deleted",
                    id,
                    other
                )))
            }
        }
    }
}

/// Stop an engine to release resources. But engine info is still kept in the data store.
async fn stop_engine<DB: DatabaseClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()> {
    let db_client = state.get_db();
    let mut current_state = get_engine_state(&id, db_client).await?;

    loop {
        let new_state = match current_state {
            WaitToStart => Terminated,
            StartInProgress | Running => WaitToTerminate,
            other => {
                return Err(RucatError::not_allowed(anyhow!(
                    "Engine {} is in {:?} state, cannot be stopped",
                    id,
                    other
                )))
            }
        };
        let response = db_client
            .update_engine_state(&id, &current_state, &new_state)
            .await?
            .ok_or_else(|| RucatError::engine_not_found(&id))?;
        if response.update_success {
            info!("Engine {} is stopped", id);
            return Ok(());
        }
        debug!(
            "Engine {} has been updated from {:?} to {:?}, retry to stop",
            id, current_state, response.before_state
        );
        current_state = response.before_state;
    }
}

/// Restart a stopped engine with the same configuration.
async fn restart_engine<DB: DatabaseClient>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()> {
    let db_client = state.get_db();
    let mut current_state = get_engine_state(&id, db_client).await?;

    loop {
        let new_state = match current_state {
            WaitToTerminate => Running,
            Terminated => WaitToStart,
            other => {
                return Err(RucatError::not_allowed(anyhow!(
                    "Engine {} is in {:?} state, cannot be restarted",
                    id,
                    other
                )))
            }
        };
        let response = db_client
            .update_engine_state(&id, &current_state, &new_state)
            .await?
            .ok_or_else(|| RucatError::engine_not_found(&id))?;
        if response.update_success {
            info!("Wait for Engine {} to restart", id);
            return Ok(());
        }
        debug!(
            "Engine {} has been updated from {:?} to {:?}, retry to restart",
            id, current_state, response.before_state
        );
        current_state = response.before_state;
    }
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

/// helper function to get the engine state
async fn get_engine_state<DBClient>(id: &EngineId, db_client: &DBClient) -> Result<EngineState>
where
    DBClient: DatabaseClient,
{
    db_client.get_engine(id).await?.map_or_else(
        || Err(RucatError::engine_not_found(id)),
        |info| Ok(info.state),
    )
}

/// Pass the data store endpoint later
pub(crate) fn get_engine_router<DB: DatabaseClient>() -> Router<AppState<DB>> {
    Router::new()
        .route("/", post(start_engine::<DB>).get(list_engines::<DB>))
        .route("/:id", get(get_engine::<DB>).delete(delete_engine::<DB>))
        .route("/:id/stop", post(stop_engine::<DB>))
        .route("/:id/restart", post(restart_engine::<DB>))
}
