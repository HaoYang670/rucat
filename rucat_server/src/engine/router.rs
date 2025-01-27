//! Restful API for engine management.

use ::std::time::SystemTime;

use ::rucat_common::{
    anyhow::anyhow,
    database::Database,
    engine::{
        CreateEngineRequest, EngineId, EngineInfo,
        EngineState::{self, *},
    },
    error::RucatError,
    tracing::info,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::{error::RucatServerError, state::AppState};

type Result<T> = std::result::Result<T, RucatServerError>;

/// start an engine with the given configuration
async fn create_engine<DB: Database>(
    State(state): State<AppState<DB>>,
    Json(body): Json<CreateEngineRequest>,
) -> Result<Json<EngineId>> {
    let id = state
        .get_db()
        .add_engine(body, Some(SystemTime::now()))
        .await?;
    info!("Creating engine {}, wait to start", id);
    Ok(Json(id))
}

async fn delete_engine<DB>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()>
where
    DB: Database,
{
    let db_client = state.get_db();
    let mut current_state = get_engine_state(&id, db_client).await?;

    loop {
        match current_state {
            s @ (WaitToStart | Terminated | ErrorClean(_)) => {
                let response = db_client
                    .remove_engine(&id, &s)
                    .await?
                    .ok_or_else(|| RucatError::engine_not_found(&id))?;
                if response.update_success {
                    info!("Engine {} is in {:?} state, delete it", id, s);
                    return Ok(());
                }
                info!(
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
                ))
                .into())
            }
        }
    }
}

/// Stop an engine to release resources. But engine info is still kept in the data store.
async fn stop_engine<DB: Database>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<()> {
    let db_client = state.get_db();
    let mut current_state = get_engine_state(&id, db_client).await?;

    loop {
        let (new_state, next_update_time) = match current_state {
            WaitToStart => (Terminated, None),
            StartInProgress | Running => (WaitToTerminate, Some(SystemTime::now())),
            other => {
                return Err(RucatError::not_allowed(anyhow!(
                    "Engine {} is in {:?} state, cannot be stopped",
                    id,
                    other
                ))
                .into())
            }
        };
        let response = db_client
            .update_engine_state(&id, &current_state, &new_state, next_update_time)
            .await?
            .ok_or_else(|| RucatError::engine_not_found(&id))?;
        if response.update_success {
            info!(
                "Update Engine {} from {:?} to {:?}",
                id, current_state, new_state
            );
            return Ok(());
        }
        info!(
            "Engine {} has been updated from {:?} to {:?}, retry to stop",
            id, current_state, response.before_state
        );
        current_state = response.before_state;
    }
}

/// Restart a stopped engine with the same configuration.
async fn restart_engine<DB: Database>(
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
                ))
                .into())
            }
        };
        let response = db_client
            // For Running state, we set next_update_time to current time to trigger the state monitor immediately because
            // rucat server does not know the check interval of the state monitor.
            .update_engine_state(&id, &current_state, &new_state, Some(SystemTime::now()))
            .await?
            .ok_or_else(|| RucatError::engine_not_found(&id))?;
        if response.update_success {
            info!(
                "Update Engine {} from {:?} to {:?}",
                id, current_state, new_state
            );
            return Ok(());
        }
        info!(
            "Engine {} has been updated from {:?} to {:?}, retry to restart",
            id, current_state, response.before_state
        );
        current_state = response.before_state;
    }
}

async fn get_engine<DB: Database>(
    Path(id): Path<EngineId>,
    State(state): State<AppState<DB>>,
) -> Result<Json<EngineInfo>> {
    state
        .get_db()
        .get_engine(&id)
        .await?
        .map(Json)
        .ok_or(RucatError::engine_not_found(&id).into())
}

async fn list_engines<DB: Database>(
    State(state): State<AppState<DB>>,
) -> Result<Json<Vec<EngineId>>> {
    state
        .get_db()
        .list_engines()
        .await
        .map(Json)
        .map_err(|e| e.into())
}

/// helper function to get the engine state
async fn get_engine_state<DB>(id: &EngineId, db_client: &DB) -> Result<EngineState>
where
    DB: Database,
{
    db_client.get_engine(id).await?.map_or_else(
        || Err(RucatError::engine_not_found(id).into()),
        |info| Ok(info.state),
    )
}

/// Pass the data store endpoint later
pub(crate) fn get_engine_router<DB: Database>() -> Router<AppState<DB>> {
    Router::new()
        .route("/", post(create_engine::<DB>).get(list_engines::<DB>))
        .route("/{id}", get(get_engine::<DB>).delete(delete_engine::<DB>))
        .route("/{id}/stop", post(stop_engine::<DB>))
        .route("/{id}/restart", post(restart_engine::<DB>))
}
