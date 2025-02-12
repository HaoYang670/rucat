//! Client of SurrealDB

use ::std::time::{SystemTime, UNIX_EPOCH};

use ::serde::Deserialize;

use crate::engine::{CreateEngineRequest, EngineId};
use crate::error::{Result, RucatError};
use crate::{
    config::Credentials,
    engine::{EngineInfo, EngineState},
};
use ::anyhow::anyhow;
use ::surrealdb::opt::auth::Root;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};

use super::{Database, EngineIdAndInfo, UpdateEngineStateResult};

/// Client to interact with the database.
/// Store the metadata of Engines
/// Record format in the database:
/// ```json
/// {
///   "id": "record id",
///   "info": "engine info",
///   "next_update_time": "timestamp that state monitor should do info update after it"
/// }
#[derive(Clone)]
pub struct SurrealDBClient {
    client: Surreal<Client>,
}

impl SurrealDBClient {
    const TABLE: &'static str = "engines";
    const NAMESPACE: &'static str = "rucat";
    const DATABASE: &'static str = "rucat";

    /// Create a new [SurrealDBClient] to connect to an existing surreal database.
    pub async fn new(credentials: Option<&Credentials>, uri: String) -> Result<Self> {
        let client = Surreal::new::<Ws>(&uri)
            .await
            .map_err(RucatError::fail_to_connect_database)?;
        if let Some(Credentials { username, password }) = credentials {
            client
                .signin(Root { username, password })
                .await
                .map_err(RucatError::fail_to_connect_database)?;
        }
        client
            .use_ns(Self::NAMESPACE)
            .use_db(Self::DATABASE)
            .await
            .map_err(RucatError::fail_to_connect_database)?;
        Ok(Self { client })
    }

    fn convert_system_time_to_secs(time: SystemTime) -> u64 {
        time.duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
}

impl Database for SurrealDBClient {
    async fn add_engine(
        &self,
        engine: CreateEngineRequest,
        next_update_time: Option<SystemTime>,
    ) -> Result<EngineId> {
        let info: EngineInfo = engine.try_into()?;
        // always set next_update_time to now  when adding a new engine,
        // so that the state monitor will update the engine info immediately
        let sql = r#"
            DEFINE FIELD IF NOT EXISTS info.state ON engines TYPE
                'WaitToStart' |
                'TriggerStart' |
                'StartInProgress' |
                'Running' |
                'WaitToTerminate' |
                'TriggerTermination' |
                'TerminateInProgress' |
                'Terminated' |
                { ErrorWaitToClean: string} |
                { ErrorTriggerClean: string } |
                { ErrorCleanInProgress: string } |
                { ErrorClean: string };

            CREATE ONLY type::table($table)
            SET info = $info, next_update_time = $next_update_time
            RETURN VALUE record::id(id);
        "#;

        let record: Option<String> = self
            .client
            .query(sql)
            .bind(("table", Self::TABLE))
            .bind(("info", info))
            // the next_update_time field is not set in surreal when it is None
            .bind((
                "next_update_time",
                next_update_time.map(Self::convert_system_time_to_secs),
            ))
            .await
            .map_err(RucatError::fail_to_update_database)?
            .take(1)
            .map_err(RucatError::fail_to_update_database)?;
        let id = record.map(EngineId::try_from);
        id.unwrap_or_else(|| {
            RucatError::fail_to_update_database(anyhow!("Failed to add engine")).into()
        })
    }

    async fn remove_engine(
        &self,
        id: &EngineId,
        current_state: &EngineState,
    ) -> Result<Option<UpdateEngineStateResult>> {
        let sql = r#"
            let $record_id = type::thing($tb, $id);             // 0th return value

            BEGIN TRANSACTION;
            {
                LET $current_state = (SELECT VALUE info.state from only $record_id);
                IF $current_state IS NONE {
                    RETURN NONE;                                                     // 1st return value
                } ELSE IF $current_state == $before {
                    DELETE $record_id;
                    RETURN "Success";                                                // 1st return value
                } ELSE {
                    RETURN {Fail: {current_state: $current_state}};                 // 1st return value
                }
            };
            COMMIT TRANSACTION;
        "#;
        let result: Option<UpdateEngineStateResult> = self
            .client
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("id", id.to_string()))
            .bind(("before", current_state.clone()))
            .await
            .map_err(RucatError::fail_to_update_database)?
            .take(1)
            .map_err(RucatError::fail_to_update_database)?;
        Ok(result)
    }

    async fn update_engine_state(
        &self,
        id: &EngineId,
        before: &EngineState,
        after: &EngineState,
        next_update_time: Option<SystemTime>,
    ) -> Result<Option<UpdateEngineStateResult>> {
        let sql = r#"
            let $record_id = type::thing($tb, $id);             // 0th return value

            BEGIN TRANSACTION;
            {
                LET $current_state = (SELECT VALUE info.state from only $record_id);
                IF $current_state IS NONE {
                    RETURN NONE;                                                     // 1st return value
                } ELSE IF $current_state == $before {
                    UPDATE ONLY $record_id SET info.state = $after, next_update_time = $next_update_time;
                    RETURN "Success";                  // 1st return value
                } ELSE {
                    RETURN {Fail: {current_state: $current_state}};                 // 1st return value
                }
            };
            COMMIT TRANSACTION;
        "#;
        let before_state: Option<UpdateEngineStateResult> = self
            .client
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("id", id.to_string()))
            .bind(("before", before.clone()))
            .bind(("after", after.clone()))
            .bind((
                "next_update_time",
                next_update_time.map(Self::convert_system_time_to_secs),
            ))
            .await
            .map_err(RucatError::fail_to_update_database)?
            .take(1)
            .map_err(RucatError::fail_to_update_database)?; // The 1st statement is the if-else which is what we want

        Ok(before_state)
    }

    async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
        let sql = r#"
            SELECT VALUE info
            FROM ONLY type::thing($tb, $id);
        "#;
        let info: Option<EngineInfo> = self
            .client
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("id", id.to_string()))
            .await
            .map_err(RucatError::fail_to_read_database)?
            .take(0)
            .map_err(RucatError::fail_to_read_database)?;
        Ok(info)
    }

    async fn list_engines(&self) -> Result<Vec<EngineId>> {
        let sql = r#"
            SELECT VALUE record::id(id) FROM type::table($tb);
        "#;

        let ids: Vec<String> = self
            .client
            .query(sql)
            .bind(("tb", Self::TABLE))
            .await
            .map_err(RucatError::fail_to_read_database)?
            .take(0)
            .map_err(RucatError::fail_to_read_database)?;
        let mut ids: Vec<_> = ids
            .into_iter()
            .map(EngineId::try_from)
            .collect::<Result<_>>()?;
        ids.sort();
        Ok(ids)
    }

    async fn list_engines_need_update(&self) -> Result<Vec<EngineIdAndInfo>> {
        let sql = r#"
            SELECT VALUE {id: record::id(id), info: info}
            FROM type::table($tb)
            WHERE next_update_time != None && next_update_time < $now;
        "#;

        #[derive(Deserialize)]
        struct EngineIdStringAndInfo {
            id: String,
            info: EngineInfo,
        }

        let id_and_info: Vec<EngineIdStringAndInfo> = self
            .client
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("now", Self::convert_system_time_to_secs(SystemTime::now())))
            .await
            .map_err(RucatError::fail_to_read_database)?
            .take(0)
            .map_err(RucatError::fail_to_read_database)?;

        id_and_info
            .into_iter()
            .map(|EngineIdStringAndInfo { id, info }| {
                Ok(EngineIdAndInfo {
                    id: EngineId::try_from(id)?,
                    info,
                })
            })
            .collect()
    }
}
