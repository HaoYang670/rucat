use ::core::time::Duration;
use ::std::{borrow::Cow, time::SystemTime};

use ::rucat_common::{
    database::{Database, EngineIdAndInfo},
    engine::{
        EngineId,
        EngineState::{self, *},
    },
    tokio,
    tracing::{debug, error, info, warn},
};
use resource_manager::{ResourceManager, ResourceState};

pub mod config;
pub mod resource_manager;

/// This function runs forever to monitor the state of engines.
pub async fn run_state_monitor<DB, RSManager>(
    db_client: DB,
    resource_manager: RSManager,
    check_interval_secs: u8,
) -> !
where
    DB: Database,
    RSManager: ResourceManager,
{
    let check_interval = Duration::from_secs(check_interval_secs as u64);
    // TODO: make this configurable
    let trigger_state_timeout = Duration::from_secs(60);
    loop {
        let start_time = std::time::Instant::now();
        match db_client.list_engines_need_update().await {
            Ok(engines) => {
                info!("Detect {} engines need to update", engines.len());
                // TODO: make this execute in parallel
                for EngineIdAndInfo { id, info } in engines {
                    match info.state {
                        WaitToStart => {
                            // acquire the engine
                            let acquired = acquire_engine(
                                &db_client,
                                &id,
                                &WaitToStart,
                                trigger_state_timeout,
                            )
                            .await;
                            if acquired {
                                info!("Create engine {}", id);
                                // create engine resource
                                let err_msg =
                                    match resource_manager.create_resource(&id, &info).await {
                                        Ok(()) => {
                                            info!("Create engine resource for {}", id);
                                            None
                                        }
                                        Err(e) => {
                                            error!(
                                                "Failed to create engine resource for {}: {}",
                                                id, e
                                            );
                                            Some(Cow::Owned(e.to_string()))
                                        }
                                    };
                                release_engine(
                                    &db_client,
                                    &TriggerStart,
                                    &id,
                                    check_interval,
                                    err_msg,
                                )
                                .await;
                            }
                        }
                        WaitToTerminate => {
                            let acquired = acquire_engine(
                                &db_client,
                                &id,
                                &WaitToTerminate,
                                trigger_state_timeout,
                            )
                            .await;
                            if acquired {
                                info!("Terminate engine {}", id);
                                // clean engine resource
                                let err_msg = match resource_manager.clean_resource(&id).await {
                                    Ok(()) => {
                                        info!("Clean engine resource for {}", id);
                                        None
                                    }
                                    Err(e) => {
                                        error!("Failed to clean engine resource for {}: {}", id, e);
                                        Some(Cow::Owned(e.to_string()))
                                    }
                                };
                                release_engine(
                                    &db_client,
                                    &TriggerTermination,
                                    &id,
                                    check_interval,
                                    err_msg,
                                )
                                .await;
                            }
                        }
                        ErrorWaitToClean(s) => {
                            let acquired = acquire_engine(
                                &db_client,
                                &id,
                                &ErrorWaitToClean(s.clone()),
                                trigger_state_timeout,
                            )
                            .await;
                            if acquired {
                                info!("Clean resource for error state engine {}", id);
                                // clean engine resource
                                let err_msg = match resource_manager.clean_resource(&id).await {
                                    Ok(()) => {
                                        info!("Clean engine resource for {}", id);
                                        None
                                    }
                                    Err(e) => {
                                        error!("Failed to clean engine resource for {}: {}", id, e);
                                        Some(Cow::Owned(e.to_string()))
                                    }
                                };
                                release_engine(
                                    &db_client,
                                    &ErrorTriggerClean(s),
                                    &id,
                                    check_interval,
                                    err_msg,
                                )
                                .await;
                            }
                        }

                        old_state @ (Running
                        | StartInProgress
                        | TerminateInProgress
                        | ErrorCleanInProgress(_)) => {
                            let resource_state = resource_manager.get_resource_state(&id).await;
                            let new_state = resource_state
                                .get_new_engine_state(&old_state)
                                .unwrap_or(old_state.clone());
                            inspect_engine_state_updating(
                                &db_client,
                                &id,
                                &old_state,
                                &new_state,
                                get_next_update_time(
                                    &new_state,
                                    check_interval,
                                    trigger_state_timeout,
                                ),
                            )
                            .await;
                        }
                        other => {
                            error!("Should not monitor engine {} in state {:?}", id, other);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to get engine list: {}", e);
            }
        }
        let elapsed = start_time.elapsed();
        let sleep_duration = check_interval.checked_sub(elapsed).unwrap_or_default();
        debug!(
            "Takes {:?} to finish one round monitoring, sleep for {:?}",
            elapsed, sleep_duration
        );
        tokio::time::sleep(sleep_duration).await;
    }
}

/// For engine in state `Trigger*`, release it by updating its state to `*InProgress`,
/// or to Error states if error message is provided.
async fn release_engine<DB>(
    db_client: &DB,
    current_state: &EngineState,
    id: &EngineId,
    check_interval: Duration,
    err_msg: Option<Cow<'static, str>>,
) where
    DB: Database,
{
    let now = SystemTime::now();
    let next_update_time = Some(now + check_interval);
    // TODO: wrap `Trigger*` states in a new type
    let (new_state, next_update_time) = match (current_state, err_msg) {
        (TriggerStart, None) => (StartInProgress, next_update_time),
        (TriggerStart, Some(s)) => (ErrorClean(s), None),
        (TriggerTermination, None) => (TerminateInProgress, next_update_time),
        (TriggerTermination, Some(s)) => (ErrorWaitToClean(s), Some(now)),
        (ErrorTriggerClean(s), None) => (ErrorCleanInProgress(s.clone()), next_update_time),
        (ErrorTriggerClean(s1), Some(s2)) => (
            ErrorWaitToClean(Cow::Owned(format!("{}\n\n{}", s1, s2))),
            Some(now),
        ),
        _ => unreachable!("Should not release engine in state {:?}", current_state),
    };
    let response = db_client
        .update_engine_state(id, current_state, &new_state, next_update_time)
        .await;
    match response {
        Ok(Some(response)) => {
            if response.update_success {
                info!(
                    "Engine {} state updated from {:?} to {:?}",
                    id, current_state, new_state
                );
            } else {
                unreachable!(
                    "Bug: engine {} in {:?} start is updated to {:?} by others",
                    id, current_state, response.before_state
                );
            }
        }
        Ok(None) => {
            unreachable!(
                "Bug: engine {} in {:?} state is removed by others",
                id, current_state
            );
        }
        Err(e) => {
            // In this case, we keep the engine in the current state and let other monitors
            // find and update it after it times out.
            warn!(
                "Database error when updating the state of engine {}: {}",
                id, e
            );
        }
    }
}

/// Acquire the engine by updating its state from *WaitTo* to *Trigger*.
/// # Parameters
/// - `db_client`: The database client.
/// - `id`: The id of the engine.
/// - `current_state`: The expected state of the engine before the update. It should be *WaitTo*.
/// - `trigger_state_timeout`: *Trigger* states are expected to exist only for a very short time,
///   and then be updated to *InProgress* or *Error* states. However, there is a possibility that
///   the state monitor is down when the engine is in *Trigger* state, so we need to set a timeout
///   to avoid the engine being stuck in *Trigger* state. State monitor will pick up those timed out engines
///   and retrigger them.
async fn acquire_engine<DB>(
    db_client: &DB,
    id: &EngineId,
    current_state: &EngineState,
    trigger_state_timeout: Duration,
) -> bool
where
    DB: Database,
{
    let next_update_time = Some(SystemTime::now() + trigger_state_timeout);
    let new_state = match current_state {
        WaitToStart => TriggerStart,
        WaitToTerminate => TriggerTermination,
        ErrorWaitToClean(s) => ErrorTriggerClean(s.clone()),
        _ => unreachable!("Should not acquire engine in state {:?}", current_state),
    };
    inspect_engine_state_updating(db_client, id, current_state, &new_state, next_update_time).await
}

/// Update the state of an engine in the database and log the result.
/// Return whether the state is updated successfully.
async fn inspect_engine_state_updating<DB>(
    db_client: &DB,
    id: &EngineId,
    old_state: &EngineState,
    new_state: &EngineState,
    next_update_time: Option<SystemTime>,
) -> bool
where
    DB: Database,
{
    let response = db_client
        .update_engine_state(id, old_state, new_state, next_update_time)
        .await;
    match response {
        Ok(Some(response)) => {
            if response.update_success {
                info!(
                    "Engine {} state updated from {:?} to {:?}",
                    id, old_state, new_state
                );
                true
            } else {
                warn!(
                    "Failed to update engine {} as its state has been updated by others, \
                    from {:?} to {:?}",
                    id, old_state, response.before_state
                );
                false
            }
        }
        Ok(None) => {
            warn!("Failed to update engine {} as it has been removed", id);
            false
        }
        Err(e) => {
            error!(
                "Database error when updating the state of engine {}: {}",
                id, e
            );
            false
        }
    }
}

fn get_next_update_time(
    state: &EngineState,
    check_interval: Duration,
    trigger_state_timeout: Duration,
) -> Option<SystemTime> {
    let now = SystemTime::now();
    match state {
        WaitToStart | WaitToTerminate | ErrorWaitToClean(_) => Some(now),
        TriggerStart | TriggerTermination | ErrorTriggerClean(_) => {
            Some(now + trigger_state_timeout)
        }
        StartInProgress | Running | TerminateInProgress | ErrorCleanInProgress(_) => {
            Some(now + check_interval)
        }
        Terminated | ErrorClean(_) => None,
    }
}
