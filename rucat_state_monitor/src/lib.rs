use ::rucat_common::{
    database_client::DatabaseClient,
    engine::{
        EngineId,
        EngineState::{self, *},
    },
    tracing::{debug, error, info, warn},
};
use resource_client::{ResourceClient, ResourceState};

pub mod config;
pub mod resource_client;

/// This function runs forever to monitor the state of engines.
pub async fn run_state_monitor<DBClient, RSClient>(
    db_client: DBClient,
    resource_client: RSClient,
    check_interval_millis: u64,
) -> !
where
    DBClient: DatabaseClient,
    RSClient: ResourceClient,
{
    let check_interval = std::time::Duration::from_millis(check_interval_millis);
    loop {
        match db_client.list_engines_need_update().await {
            Ok(engines) => {
                debug!("Detect {} engines need to update", engines.len());
                // TODO: make this execute in parallel
                for (id, info) in engines {
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
                                match resource_client.create_resource(&id, &info.config).await {
                                    Ok(()) => {
                                        info!("Create engine resource for {}", id);
                                        // release the engine
                                        release_engine(&db_client, &TriggerStart, &id).await;
                                    }
                                    Err(e) => {
                                        error!(
                                            "Failed to create engine resource for {}: {}",
                                            id, e
                                        );
                                        release_engine_after_error(
                                            &db_client,
                                            &TriggerStart,
                                            &id,
                                            &e.to_string(),
                                        )
                                        .await;
                                    }
                                }
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
                                match resource_client.clean_resource(&id).await {
                                    Ok(()) => {
                                        info!("Clean engine resource for {}", id);
                                        // release the engine
                                        release_engine(&db_client, &TriggerTermination, &id).await;
                                    }
                                    Err(e) => {
                                        error!("Failed to clean engine resource for {}: {}", id, e);
                                        release_engine_after_error(
                                            &db_client,
                                            &TriggerTermination,
                                            &id,
                                            &e.to_string(),
                                        )
                                        .await;
                                    }
                                }
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
                                match resource_client.clean_resource(&id).await {
                                    Ok(()) => {
                                        info!("Clean engine resource for {}", id);
                                        // release the engine
                                        release_engine(&db_client, &ErrorTriggerClean(s), &id)
                                            .await;
                                    }
                                    Err(e) => {
                                        error!("Failed to clean engine resource for {}: {}", id, e);
                                        release_engine_after_error(
                                            &db_client,
                                            &ErrorTriggerClean(s),
                                            &id,
                                            &e.to_string(),
                                        )
                                        .await;
                                    }
                                }
                            }
                        }

                        old_state @ (Running
                        | StartInProgress
                        | TerminateInProgress
                        | ErrorCleanInProgress(_)) => {
                            let resource_state = resource_client.get_resource_state(&id).await;
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
        // wait for some seconds
        std::thread::sleep(check_interval);
    }
}

/// Release engine in state `Trigger*`, after getting error when operating the engine resource.
async fn release_engine_after_error<DBClient>(
    db_client: &DBClient,
    current_state: &EngineState,
    id: &EngineId,
    err_msg: &String,
) where
    DBClient: DatabaseClient,
{
    // TODO: wrap `Trigger*` states in a new type
    let new_state = match current_state {
        TriggerStart => ErrorClean(err_msg.clone()),
        TriggerTermination => ErrorWaitToClean(err_msg.clone()),
        ErrorTriggerClean(s) => {
            let err_msg = format!("{}\n{}", s, err_msg);
            ErrorWaitToClean(err_msg)
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
                error!(
                    "Bug: engine {} in {:?} state is updated to {:?} by others",
                    id, current_state, response.before_state
                );
            }
        }
        Ok(None) => {
            error!(
                "Bug: engine {} in {:?} state is removed by others",
                id, current_state
            );
        }
        Err(e) => {
            // in this case we keep the engine in the current state and let other monitors to
            // to find and update it after it is timed out.
            // TODO: engine might hang in the current state forever, need to handle this case
            warn!(
                "Database error when updating engine {} from {:?} to {:?}: {}",
                id, current_state, new_state, e
            );
        }
    }
}

/// For engine in state `Trigger*`, release it by updating its state to `*InProgress`.
async fn release_engine<DBClient>(db_client: &DBClient, current_state: &EngineState, id: &EngineId)
where
    DBClient: DatabaseClient,
{
    // TODO: wrap `Trigger*` states in a new type
    let new_state = match current_state {
        TriggerStart => StartInProgress,
        TriggerTermination => TerminateInProgress,
        ErrorTriggerClean(s) => ErrorCleanInProgress(s.clone()),
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
                error!(
                    "Bug: engine {} in {:?} start is updated to {:?} by others",
                    id, current_state, response.before_state
                );
            }
        }
        Ok(None) => {
            error!(
                "Bug: engine {} in {:?} state is removed by others",
                id, current_state
            );
        }
        Err(e) => {
            // in this case we keep the engine in the current state and let other monitors to
            // to find and update it after it is timed out.
            warn!(
                "Database error when updating the state of engine {}: {}",
                id, e
            );
        }
    }
}

/// Update the state of an engine in the database and log the result.
/// Return whether the state is updated successfully.
async fn inspect_engine_state_updating<DBClient>(
    db_client: &DBClient,
    id: &EngineId,
    old_state: &EngineState,
    new_state: &EngineState,
) -> bool
where
    DBClient: DatabaseClient,
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
                warn!("Failed to update engine {} as its state has been updated by others, from {:?} to {:?}", id, old_state, response.before_state);
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
