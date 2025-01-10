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

/// State monitor to monitor the state of engines.
pub struct StateMonitor<DB, RSManager> {
    db_client: DB,
    resource_manager: RSManager,
    check_interval: Duration,
    trigger_state_timeout: Duration,
}

impl<DB, RSManager> StateMonitor<DB, RSManager>
where
    DB: Database,
    RSManager: ResourceManager,
{
    pub fn new(
        db_client: DB,
        resource_manager: RSManager,
        check_interval_secs: u8,
        trigger_state_timeout_secs: u16,
    ) -> Self {
        let check_interval = Duration::from_secs(check_interval_secs as u64);
        let trigger_state_timeout = Duration::from_secs(trigger_state_timeout_secs as u64);
        info!(
            "Create state monitor with check interval {:?} and trigger state timeout {:?}",
            check_interval, trigger_state_timeout
        );
        Self {
            db_client,
            resource_manager,
            check_interval,
            trigger_state_timeout,
        }
    }

    /// This function runs forever to monitor the state of engines.
    pub async fn run_state_monitor(&self) -> ! {
        loop {
            let start_time = std::time::Instant::now();
            match self.db_client.list_engines_need_update().await {
                Ok(engines) => {
                    info!("Detect {} engines need to update", engines.len());
                    // TODO: make this execute in parallel
                    for EngineIdAndInfo { id, info } in engines {
                        match info.state {
                            WaitToStart => {
                                if self.acquire_engine(&id, &WaitToStart).await {
                                    info!("Create engine {}", id);
                                    // create engine resource
                                    let err_msg = match self
                                        .resource_manager
                                        .create_resource(&id, &info)
                                        .await
                                    {
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
                                    self.release_engine(&TriggerStart, &id, err_msg).await;
                                }
                            }
                            WaitToTerminate => {
                                if self.acquire_engine(&id, &WaitToTerminate).await {
                                    info!("Terminate engine {}", id);
                                    // clean engine resource
                                    let err_msg =
                                        match self.resource_manager.clean_resource(&id).await {
                                            Ok(()) => {
                                                info!("Clean engine resource for {}", id);
                                                None
                                            }
                                            Err(e) => {
                                                error!(
                                                    "Failed to clean engine resource for {}: {}",
                                                    id, e
                                                );
                                                Some(Cow::Owned(e.to_string()))
                                            }
                                        };
                                    self.release_engine(&TriggerTermination, &id, err_msg).await;
                                }
                            }
                            ErrorWaitToClean(s) => {
                                if self.acquire_engine(&id, &ErrorWaitToClean(s.clone())).await {
                                    info!("Clean resource for error state engine {}", id);
                                    // clean engine resource
                                    let err_msg =
                                        match self.resource_manager.clean_resource(&id).await {
                                            Ok(()) => {
                                                info!("Clean engine resource for {}", id);
                                                None
                                            }
                                            Err(e) => {
                                                error!(
                                                    "Failed to clean engine resource for {}: {}",
                                                    id, e
                                                );
                                                Some(Cow::Owned(e.to_string()))
                                            }
                                        };
                                    self.release_engine(&ErrorTriggerClean(s), &id, err_msg)
                                        .await;
                                }
                            }

                            old_state @ (Running
                            | StartInProgress
                            | TerminateInProgress
                            | ErrorCleanInProgress(_)) => {
                                let resource_state =
                                    self.resource_manager.get_resource_state(&id).await;
                                let new_state = resource_state
                                    .get_new_engine_state(&old_state)
                                    .unwrap_or(old_state.clone());
                                self.inspect_engine_state_updating(
                                    &id,
                                    &old_state,
                                    &new_state,
                                    self.get_next_update_time(&new_state),
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
            let sleep_duration = self.check_interval.checked_sub(elapsed).unwrap_or_default();
            debug!(
                "Takes {:?} to finish one round monitoring, sleep for {:?}",
                elapsed, sleep_duration
            );
            tokio::time::sleep(sleep_duration).await;
        }
    }

    /// For engine in state `Trigger*`, release it by updating its state to `*InProgress`,
    /// or to Error states if error message is provided.
    async fn release_engine(
        &self,
        current_state: &EngineState,
        id: &EngineId,
        err_msg: Option<Cow<'static, str>>,
    ) {
        let now = SystemTime::now();
        let next_update_time = Some(now + self.check_interval);
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
        let response = self
            .db_client
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
    /// - `id`: The id of the engine.
    /// - `current_state`: The expected state of the engine before the update. It should be *WaitTo*.
    /// # Return
    /// Whether the engine is acquired successfully.
    async fn acquire_engine(&self, id: &EngineId, current_state: &EngineState) -> bool {
        let next_update_time = Some(SystemTime::now() + self.trigger_state_timeout);
        let new_state = match current_state {
            WaitToStart => TriggerStart,
            WaitToTerminate => TriggerTermination,
            ErrorWaitToClean(s) => ErrorTriggerClean(s.clone()),
            _ => unreachable!("Should not acquire engine in state {:?}", current_state),
        };
        self.inspect_engine_state_updating(id, current_state, &new_state, next_update_time)
            .await
    }

    /// Update the state of an engine in the database and log the result.
    /// # Return
    /// Whether the state is updated successfully.
    async fn inspect_engine_state_updating(
        &self,
        id: &EngineId,
        old_state: &EngineState,
        new_state: &EngineState,
        next_update_time: Option<SystemTime>,
    ) -> bool {
        let response = self
            .db_client
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

    fn get_next_update_time(&self, state: &EngineState) -> Option<SystemTime> {
        let now = SystemTime::now();
        match state {
            WaitToStart | WaitToTerminate | ErrorWaitToClean(_) => Some(now),
            TriggerStart | TriggerTermination | ErrorTriggerClean(_) => {
                Some(now + self.trigger_state_timeout)
            }
            StartInProgress | Running | TerminateInProgress | ErrorCleanInProgress(_) => {
                Some(now + self.check_interval)
            }
            Terminated | ErrorClean(_) => None,
        }
    }
}
