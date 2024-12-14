use ::rucat_common::{
    database_client::DatabaseClient,
    engine::EngineState::*,
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
                        WaitToStart => todo!(),
                        WaitToTerminate => todo!(),
                        WaitToDelete => todo!(),
                        ErrorWaitToClean(_) => todo!(),

                        old_state @ (Running | StartInProgress | TerminateInProgress | DeleteInProgress | ErrorCleanInProgress(_)) => {
                            let resource_state = resource_client.get_resource_state(&id).await;
                            let new_state = resource_state.update_engine_state(&old_state);
                            match new_state {
                                Some(new_state) => {
                                    let response = db_client.update_engine_state(&id, vec![old_state.clone()], new_state.clone()).await;
                                    match response {
                                        Ok(Some(response)) => {
                                            if response.update_success {
                                                info!("Engine {} state updated from {:?} to {:?}", id, old_state, new_state);
                                            } else {
                                                error!("Failed to update engine {} as its state has been updated by others, from {:?} to {:?}", id, old_state, response.before_state);
                                            }
                                        }
                                        Ok(None) => {
                                            error!("Failed to update engine {} as it has been removed", id);
                                        }
                                        Err(e) => {
                                            error!("Database error when updating the state of engine {}: {}", id, e);
                                        }
                                    }
                                }
                                None => {
                                    let response = db_client.delete_engine(&id, vec![old_state.clone()]).await;
                                    match response {
                                        Ok(Some(response)) => {
                                            if response.update_success {
                                                info!("Delete engine {}", id);
                                            } else {
                                                error!("Failed to delete engine {} as its state has been updated by others, from {:?} to {:?}", id, old_state, response.before_state);
                                            }
                                        }
                                        Ok(None) => {
                                            warn!("Engine {} has been removed", id);
                                        }
                                        Err(e) => {
                                            error!("Database error when deleting engine {}: {}", id, e);
                                        }
                                    }
                                }
                            }
                        },

                        Terminated | ErrorClean(_) => {
                            unreachable!(
                                "list_engines_need_update should not return engine in state {:?}",
                                info.state
                            );
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
