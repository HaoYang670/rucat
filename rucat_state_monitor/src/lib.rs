use ::std::borrow::Cow;

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
    check_interval_millis: u64,
) -> !
where
    DB: Database,
    RSManager: ResourceManager,
{
    let check_interval = std::time::Duration::from_millis(check_interval_millis);
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
                            let acquired = inspect_engine_state_updating(
                                &db_client,
                                &id,
                                &WaitToStart,
                                &TriggerStart,
                            )
                            .await;
                            if acquired {
                                info!("Start engine {}", id);
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
                                release_engine(&db_client, &TriggerStart, &id, err_msg).await;
                            }
                        }
                        WaitToTerminate => {
                            let acquired = inspect_engine_state_updating(
                                &db_client,
                                &id,
                                &WaitToTerminate,
                                &TriggerTermination,
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
                                release_engine(&db_client, &TriggerTermination, &id, err_msg).await;
                            }
                        }
                        ErrorWaitToClean(s) => {
                            let acquired = inspect_engine_state_updating(
                                &db_client,
                                &id,
                                &ErrorWaitToClean(s.clone()),
                                &ErrorTriggerClean(s.clone()),
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
                                release_engine(&db_client, &ErrorTriggerClean(s), &id, err_msg)
                                    .await;
                            }
                        }

                        old_state @ (Running
                        | StartInProgress
                        | TerminateInProgress
                        | ErrorCleanInProgress(_)) => {
                            let resource_state = resource_manager.get_resource_state(&id).await;
                            let new_state = resource_state.get_new_engine_state(&old_state);
                            match new_state {
                                Some(new_state) => {
                                    inspect_engine_state_updating(
                                        &db_client, &id, &old_state, &new_state,
                                    )
                                    .await;
                                }
                                None => {
                                    debug!("Engine {} state remains unchanged", id);
                                }
                            }
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
        info!(
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
    err_msg: Option<Cow<'static, str>>,
) where
    DB: Database,
{
    // TODO: wrap `Trigger*` states in a new type
    let new_state = match (current_state, err_msg) {
        (TriggerStart, None) => StartInProgress,
        (TriggerStart, Some(s)) => ErrorClean(s),
        (TriggerTermination, None) => TerminateInProgress,
        (TriggerTermination, Some(s)) => ErrorWaitToClean(s),
        (ErrorTriggerClean(s), None) => ErrorCleanInProgress(s.clone()),
        (ErrorTriggerClean(s1), Some(s2)) => {
            ErrorWaitToClean(Cow::Owned(format!("{}\n\n{}", s1, s2)))
        }
        _ => unreachable!("Should not release engine in state {:?}", current_state),
    };
    let response = db_client
        .update_engine_state(id, current_state, &new_state)
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

/// Update the state of an engine in the database and log the result.
/// Return whether the state is updated successfully.
async fn inspect_engine_state_updating<DB>(
    db_client: &DB,
    id: &EngineId,
    old_state: &EngineState,
    new_state: &EngineState,
) -> bool
where
    DB: Database,
{
    let response = db_client
        .update_engine_state(id, old_state, new_state)
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
