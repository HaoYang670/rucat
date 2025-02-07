use ::core::time::Duration;
use ::std::{
    borrow::Cow,
    time::{Instant, SystemTime},
};

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
    pub async fn run(&self) -> ! {
        loop {
            let start_time = Instant::now();
            match self.db_client.list_engines_need_update().await {
                Ok(engines) => {
                    info!("Detect {} engines need to update", engines.len());
                    // TODO: make this execute in parallel
                    for e in engines {
                        self.sync_engine(e).await;
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

    /// Sync the engine state with the resource manager.
    /// And update the engine state in the database.
    async fn sync_engine(&self, engine: EngineIdAndInfo) {
        let EngineIdAndInfo { id, info } = engine;
        match info.state {
            WaitToStart => {
                if self.acquire_engine(&id, &WaitToStart).await {
                    info!("Create engine {}", id);
                    // create engine resource
                    let err_msg = match self.resource_manager.create_resource(&id, &info).await {
                        Ok(()) => {
                            info!("Create engine resource for {}", id);
                            None
                        }
                        Err(e) => {
                            error!("Failed to create engine resource for {}: {}", id, e);
                            Some(Cow::Owned(e.to_string()))
                        }
                    };
                    self.release_engine(&id, &TriggerStart, err_msg).await;
                }
            }
            WaitToTerminate => {
                if self.acquire_engine(&id, &WaitToTerminate).await {
                    info!("Terminate engine {}", id);
                    // clean engine resource
                    let err_msg = match self.resource_manager.clean_resource(&id).await {
                        Ok(()) => {
                            info!("Clean engine resource for {}", id);
                            None
                        }
                        Err(e) => {
                            error!("Failed to clean engine resource for {}: {}", id, e);
                            Some(Cow::Owned(e.to_string()))
                        }
                    };
                    self.release_engine(&id, &TriggerTermination, err_msg).await;
                }
            }
            ErrorWaitToClean(s) => {
                if self.acquire_engine(&id, &ErrorWaitToClean(s.clone())).await {
                    info!("Clean resource for error state engine {}", id);
                    // clean engine resource
                    let err_msg = match self.resource_manager.clean_resource(&id).await {
                        Ok(()) => {
                            info!("Clean engine resource for {}", id);
                            None
                        }
                        Err(e) => {
                            error!("Failed to clean engine resource for {}: {}", id, e);
                            Some(Cow::Owned(e.to_string()))
                        }
                    };
                    self.release_engine(&id, &ErrorTriggerClean(s), err_msg)
                        .await;
                }
            }

            in_progress_state @ (Running
            | StartInProgress
            | TerminateInProgress
            | ErrorCleanInProgress(_)) => {
                let resource_state = self.resource_manager.get_resource_state(&id).await;
                let new_state = resource_state
                    .get_new_engine_state(&in_progress_state)
                    .unwrap_or(in_progress_state.clone());
                self.inspect_engine_state_updating(&id, &in_progress_state, &new_state)
                    .await;
            }
            // For timed out Trigger* states, switch back to the WaitTo* state to retry.
            timed_out_triggered_state @ (TriggerStart | TriggerTermination
            | ErrorTriggerClean(_)) => {
                self.retry_triggering_engine(&id, &timed_out_triggered_state)
                    .await;
            }
            stable_state @ (Terminated | ErrorClean(_)) => {
                unreachable!(
                    "Should not monitor engine {} in state {:?}",
                    id, stable_state
                );
            }
        }
    }

    /// For timed out Trigger* states, retry triggering the engine by updating its state to WaitTo*.
    async fn retry_triggering_engine(&self, id: &EngineId, current_state: &EngineState) {
        let new_state = match current_state {
            TriggerStart => WaitToStart,
            TriggerTermination => WaitToTerminate,
            ErrorTriggerClean(s) => ErrorWaitToClean(s.clone()),
            _ => unreachable!(
                "Should not retry triggering engine in state {:?}",
                current_state
            ),
        };
        warn!(
            "Engine {} in state {:?} times out, retry triggering it",
            id, current_state
        );
        self.inspect_engine_state_updating(id, current_state, &new_state)
            .await;
    }

    /// For engine in state `Trigger*`, release it by updating its state to `*InProgress`,
    /// or to Error states if error message is provided.
    async fn release_engine(
        &self,
        id: &EngineId,
        current_state: &EngineState,
        err_msg: Option<Cow<'static, str>>,
    ) {
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
        let next_update_time = self.get_next_update_time(&new_state);
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
        let new_state = match current_state {
            WaitToStart => TriggerStart,
            WaitToTerminate => TriggerTermination,
            ErrorWaitToClean(s) => ErrorTriggerClean(s.clone()),
            _ => unreachable!("Should not acquire engine in state {:?}", current_state),
        };
        self.inspect_engine_state_updating(id, current_state, &new_state)
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
    ) -> bool {
        let next_update_time = self.get_next_update_time(new_state);
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
        get_next_update_time(
            state,
            SystemTime::now(),
            self.check_interval,
            self.trigger_state_timeout,
        )
    }
}

/// Get the next update time of the engine.
/// # Parameters
/// - `state`: The state of the engine.
/// - `now`: The current time.
/// - `check_interval`: The interval between two updates.
/// - `trigger_state_timeout`: The timeout of trigger states.
/// # Return
/// `None` means the engine does not need to be updated anymore.
/// `Some(SystemTime)` means the engine should be updated at the returned time.
fn get_next_update_time(
    state: &EngineState,
    now: SystemTime,
    check_interval: Duration,
    trigger_state_timeout: Duration,
) -> Option<SystemTime> {
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

#[cfg(test)]
mod tests {
    use ::std::collections::BTreeMap;

    use super::*;
    use crate::resource_manager::k8s_client::K8sPodState;
    use ::mockall::{mock, predicate};
    use ::rucat_common::{
        anyhow::anyhow,
        database::UpdateEngineStateResponse,
        engine::{CreateEngineRequest, EngineInfo, EngineTime, EngineType::Spark, EngineVersion},
        error::{Result, RucatError},
    };

    #[test]
    fn test_get_next_update_time() {
        let now = SystemTime::UNIX_EPOCH;
        let check_interval = Duration::from_secs(3);
        let trigger_state_timeout = Duration::from_secs(5);
        let get_next_update_time =
            |state| get_next_update_time(state, now, check_interval, trigger_state_timeout);

        let check_time = Some(now + check_interval);
        let trigger_time = Some(now + trigger_state_timeout);
        let now = Some(now);
        assert_eq!(get_next_update_time(&WaitToStart), now);
        assert_eq!(get_next_update_time(&WaitToTerminate), now);
        assert_eq!(
            get_next_update_time(&ErrorWaitToClean(Cow::Borrowed("error")),),
            now
        );
        assert_eq!(get_next_update_time(&TriggerStart), trigger_time);
        assert_eq!(get_next_update_time(&TriggerTermination), trigger_time);
        assert_eq!(
            get_next_update_time(&ErrorTriggerClean(Cow::Borrowed("error"))),
            trigger_time
        );
        assert_eq!(get_next_update_time(&StartInProgress), check_time);
        assert_eq!(get_next_update_time(&Running), check_time);
        assert_eq!(get_next_update_time(&TerminateInProgress), check_time);
        assert_eq!(
            get_next_update_time(&ErrorCleanInProgress(Cow::Borrowed("error"))),
            check_time
        );
        assert_eq!(get_next_update_time(&Terminated), None);
        assert_eq!(
            get_next_update_time(&ErrorClean(Cow::Borrowed("error"))),
            None
        );
    }

    mock! {
        DB{}
        impl Database for DB {
            async fn add_engine(&self, engine: CreateEngineRequest, next_update_time: Option<SystemTime>) -> Result<EngineId>;
            async fn remove_engine(&self, id: &EngineId, current_state: &EngineState) -> Result<Option<UpdateEngineStateResponse>>;
            async fn update_engine_state(
                &self,
                id: &EngineId,
                before: &EngineState,
                after: &EngineState,
                next_update_time: Option<SystemTime>,
            ) -> Result<Option<UpdateEngineStateResponse>>;
            async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>>;
            async fn list_engines(&self) -> Result<Vec<EngineId>>;
            async fn list_engines_need_update(&self) -> Result<Vec<EngineIdAndInfo>>;
        }
    }
    mock! {
        RM{}
        impl ResourceManager for RM {
            type ResourceState = K8sPodState;
            async fn create_resource(&self, id: &EngineId, info: &EngineInfo) -> Result<()>;
            async fn clean_resource(&self, id: &EngineId) -> Result<()>;
            async fn get_resource_state(&self, id: &EngineId) -> K8sPodState;
        }
    }

    fn create_mock_state_monitor(db: MockDB, rm: MockRM) -> StateMonitor<MockDB, MockRM> {
        // check intervals are not tested.
        StateMonitor::new(db, rm, 0, 0)
    }

    #[tokio::test]
    async fn inspect_engine_state_updating_success() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToStart),
                predicate::eq(&Terminated),
                predicate::eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: WaitToStart,
                    update_success: true,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        assert_eq!(
            monitor
                .inspect_engine_state_updating(&engine_id, &WaitToStart, &Terminated)
                .await,
            true
        );
    }

    #[tokio::test]
    async fn inspect_engine_state_updating_fail_1() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToStart),
                predicate::eq(&Terminated),
                predicate::eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerStart,
                    update_success: false,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        assert_eq!(
            monitor
                .inspect_engine_state_updating(&engine_id, &WaitToStart, &Terminated)
                .await,
            false
        );
    }

    #[tokio::test]
    async fn inspect_engine_state_updating_fail_2() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToStart),
                predicate::eq(&Terminated),
                predicate::eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(None));
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        assert_eq!(
            monitor
                .inspect_engine_state_updating(&engine_id, &WaitToStart, &Terminated)
                .await,
            false
        );
    }

    #[tokio::test]
    async fn inspect_engine_state_updating_fail_3() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToStart),
                predicate::eq(&Terminated),
                predicate::eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Err(RucatError::fail_to_connect_database(anyhow!(""))));
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        assert_eq!(
            monitor
                .inspect_engine_state_updating(&engine_id, &WaitToStart, &Terminated)
                .await,
            false
        );
    }

    #[tokio::test]
    async fn acquire_engine_success() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToStart),
                predicate::eq(&TriggerStart),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: WaitToStart,
                    update_success: true,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        assert_eq!(monitor.acquire_engine(&engine_id, &WaitToStart).await, true);
    }

    #[tokio::test]
    #[should_panic(expected = "Should not acquire engine in state Running")]
    async fn acquire_engine_panic() {
        let engine_id = EngineId::try_from("123").unwrap();
        let db = MockDB::new();
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        monitor.acquire_engine(&engine_id, &Running).await;
    }

    #[tokio::test]
    async fn release_engine_success() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::eq(&StartInProgress),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerStart,
                    update_success: true,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .release_engine(&engine_id, &TriggerStart, None)
            .await
    }

    #[tokio::test]
    async fn release_engine_to_err_state_success() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::eq(&ErrorClean(Cow::Borrowed("error"))),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerStart,
                    update_success: true,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .release_engine(&engine_id, &TriggerStart, Some(Cow::Borrowed("error")))
            .await
    }

    #[tokio::test]
    #[should_panic(expected = "Should not release engine in state WaitToStart")]
    async fn release_engine_panic_on_unexpected_state() {
        let engine_id = EngineId::try_from("123").unwrap();
        let db = MockDB::new();
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        monitor.release_engine(&engine_id, &WaitToStart, None).await
    }

    #[tokio::test]
    #[should_panic(
        expected = "Bug: engine 123 in TriggerStart start is updated to StartInProgress by others"
    )]
    async fn release_engine_panic_on_unexpected_conflict_1() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::eq(&StartInProgress),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: StartInProgress,
                    update_success: false,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        monitor
            .release_engine(&engine_id, &TriggerStart, None)
            .await
    }

    #[tokio::test]
    #[should_panic(expected = "Bug: engine 123 in TriggerStart state is removed by others")]
    async fn release_engine_panic_on_unexpected_conflict_2() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::eq(&StartInProgress),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(None));
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        monitor
            .release_engine(&engine_id, &TriggerStart, None)
            .await
    }

    #[tokio::test]
    async fn release_engine_not_panic_on_db_error() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::eq(&StartInProgress),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| Err(RucatError::fail_to_connect_database(anyhow!(""))));
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .release_engine(&engine_id, &TriggerStart, None)
            .await
    }

    #[tokio::test]
    async fn retry_triggering_engine_success() {
        let engine_id = EngineId::try_from("123").unwrap();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::eq(&WaitToStart),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerStart,
                    update_success: true,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .retry_triggering_engine(&engine_id, &TriggerStart)
            .await
    }

    #[tokio::test]
    #[should_panic(expected = "Should not retry triggering engine in state WaitToStart")]
    async fn retry_triggering_engine_panic_on_unexpected_state() {
        let engine_id = EngineId::try_from("123").unwrap();
        let db = MockDB::new();
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        monitor
            .retry_triggering_engine(&engine_id, &WaitToStart)
            .await
    }

    #[tokio::test]
    async fn sync_wait_to_start_engine_success() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            WaitToStart,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // acquire engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToStart),
                predicate::eq(&TriggerStart),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: WaitToStart,
                    update_success: true,
                }))
            });
        // release engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::eq(&StartInProgress),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerStart,
                    update_success: true,
                }))
            });
        let mut rm = MockRM::new();
        rm.expect_create_resource()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(engine_info.clone()),
            )
            .times(1)
            .returning(|_, _| Ok(()));
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_wait_to_start_engine_error() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.5"),
            WaitToStart,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // acquire engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToStart),
                predicate::eq(&TriggerStart),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: WaitToStart,
                    update_success: true,
                }))
            });
        // release engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::function(|s| matches!(s, ErrorClean(_))),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerStart,
                    update_success: true,
                }))
            });
        let mut rm = MockRM::new();
        rm.expect_create_resource()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(engine_info.clone()),
            )
            .times(1)
            .returning(|_, _| Err(RucatError::not_allowed(anyhow!("3.5.5 is invalid"))));
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_wait_to_start_engine_skipped() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            WaitToStart,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // engine has been acquired by others
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToStart),
                predicate::eq(&TriggerStart),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerStart,
                    update_success: false,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_wait_to_terminate_engine_success() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            WaitToTerminate,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // acquire engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToTerminate),
                predicate::eq(&TriggerTermination),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: WaitToTerminate,
                    update_success: true,
                }))
            });
        // release engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerTermination),
                predicate::eq(&TerminateInProgress),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerTermination,
                    update_success: true,
                }))
            });
        let mut rm = MockRM::new();
        rm.expect_clean_resource()
            .with(predicate::eq(engine_id.clone()))
            .times(1)
            .returning(|_| Ok(()));
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_wait_to_terminate_engine_error() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.5"),
            WaitToTerminate,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // acquire engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToTerminate),
                predicate::eq(&TriggerTermination),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: WaitToTerminate,
                    update_success: true,
                }))
            });
        // release engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerTermination),
                predicate::function(|s| matches!(s, ErrorWaitToClean(_))),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerTermination,
                    update_success: true,
                }))
            });
        let mut rm = MockRM::new();
        rm.expect_clean_resource()
            .with(predicate::eq(engine_id.clone()))
            .times(1)
            .returning(|_| Err(RucatError::fail_to_delete_engine(anyhow!("some error"))));
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_wait_to_terminate_engine_skipped() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            WaitToTerminate,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // engine has been acquired by others
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&WaitToTerminate),
                predicate::eq(&TriggerTermination),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerTermination,
                    update_success: false,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_error_wait_to_clean_engine_success() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            ErrorWaitToClean(Cow::Borrowed("error")),
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // acquire engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(ErrorWaitToClean(Cow::Borrowed("error"))),
                predicate::eq(ErrorTriggerClean(Cow::Borrowed("error"))),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: ErrorWaitToClean(Cow::Borrowed("error")),
                    update_success: true,
                }))
            });
        // release engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(ErrorTriggerClean(Cow::Borrowed("error"))),
                predicate::eq(ErrorCleanInProgress(Cow::Borrowed("error"))),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: ErrorTriggerClean(Cow::Borrowed("error")),
                    update_success: true,
                }))
            });
        let mut rm = MockRM::new();
        rm.expect_clean_resource()
            .with(predicate::eq(engine_id.clone()))
            .times(1)
            .returning(|_| Ok(()));
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_error_wait_to_clean_engine_error() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.5"),
            ErrorWaitToClean(Cow::Borrowed("error")),
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // acquire engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(ErrorWaitToClean(Cow::Borrowed("error"))),
                predicate::eq(ErrorTriggerClean(Cow::Borrowed("error"))),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: ErrorWaitToClean(Cow::Borrowed("error")),
                    update_success: true,
                }))
            });
        // release engine
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(ErrorTriggerClean(Cow::Borrowed("error"))),
                predicate::function(|s| matches!(s, ErrorWaitToClean(_))),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: ErrorTriggerClean(Cow::Borrowed("error")),
                    update_success: true,
                }))
            });
        let mut rm = MockRM::new();
        rm.expect_clean_resource()
            .with(predicate::eq(engine_id.clone()))
            .times(1)
            .returning(|_| Err(RucatError::fail_to_delete_engine(anyhow!("some error"))));
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_error_wait_to_clean_engine_skipped() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            ErrorWaitToClean(Cow::Borrowed("error")),
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut db = MockDB::new();
        // engine has been acquired by others
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(ErrorWaitToClean(Cow::Borrowed("error"))),
                predicate::eq(ErrorTriggerClean(Cow::Borrowed("error"))),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: ErrorTriggerClean(Cow::Borrowed("error")),
                    update_success: false,
                }))
            });
        let rm = MockRM::new();
        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_in_progress_state_engine_with_state_update() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            StartInProgress,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut rm = MockRM::new();
        rm.expect_get_resource_state()
            .with(predicate::eq(engine_id.clone()))
            .times(1)
            .returning(|_| K8sPodState::Running);
        let mut db = MockDB::new();

        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&StartInProgress),
                predicate::eq(&Running),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: StartInProgress,
                    update_success: true,
                }))
            });

        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_in_progress_state_engine_without_state_update() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            StartInProgress,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let mut rm = MockRM::new();
        rm.expect_get_resource_state()
            .with(predicate::eq(engine_id.clone()))
            .times(1)
            .returning(|_| K8sPodState::Pending);
        let mut db = MockDB::new();

        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&StartInProgress),
                predicate::eq(&StartInProgress),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: StartInProgress,
                    update_success: true,
                }))
            });

        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    async fn sync_timed_out_trigger_state_engine() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            TriggerStart,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let rm = MockRM::new();
        let mut db = MockDB::new();
        db.expect_update_engine_state()
            .with(
                predicate::eq(engine_id.clone()),
                predicate::eq(&TriggerStart),
                predicate::eq(&WaitToStart),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _| {
                Ok(Some(UpdateEngineStateResponse {
                    before_state: TriggerStart,
                    update_success: true,
                }))
            });

        let monitor = create_mock_state_monitor(db, rm);
        // this should not panic
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }

    #[tokio::test]
    #[should_panic(expected = "Should not monitor engine 123 in state Terminated")]
    async fn sync_stable_state_engine_panic() {
        let engine_id = EngineId::try_from("123").unwrap();
        let engine_info = EngineInfo::new(
            "abc".to_owned(),
            Spark,
            EngineVersion::from("3.5.4"),
            Terminated,
            BTreeMap::new(),
            EngineTime::now(),
        );
        let rm = MockRM::new();
        let db = MockDB::new();

        let monitor = create_mock_state_monitor(db, rm);
        monitor
            .sync_engine(EngineIdAndInfo {
                id: engine_id,
                info: engine_info,
            })
            .await
    }
}
