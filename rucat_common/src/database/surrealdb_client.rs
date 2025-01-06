//! Client of SurrealDB

use ::serde::Deserialize;
use async_trait::async_trait;

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

use super::{Database, EngineIdAndInfo, UpdateEngineStateResponse};

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
}

// TODO: replace #[async_trait] by #[trait_variant::make(HttpService: Send)] in the future: https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html#should-i-still-use-the-async_trait-macro
#[async_trait]
impl Database for SurrealDBClient {
    async fn add_engine(&self, engine: CreateEngineRequest) -> Result<EngineId> {
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
            SET info = $info, next_update_time = time::now()
            RETURN VALUE record::id(id);
        "#;

        let record: Option<String> = self
            .client
            .query(sql)
            .bind(("table", Self::TABLE))
            .bind(("info", info))
            .await
            .map_err(RucatError::fail_to_update_database)?
            .take(1)
            .map_err(RucatError::fail_to_update_database)?;
        let id = record.map(EngineId::try_from);
        id.unwrap_or_else(|| {
            RucatError::fail_to_update_database(anyhow!("Failed to add engine")).into()
        })
    }

    async fn delete_engine(
        &self,
        id: &EngineId,
        current_state: &EngineState,
    ) -> Result<Option<UpdateEngineStateResponse>> {
        let sql = r#"
            let $record_id = type::thing($tb, $id);             // 0th return value

            BEGIN TRANSACTION;
            {
                LET $current_state = (SELECT VALUE info.state from only $record_id);
                IF $current_state IS NONE {
                    RETURN NONE;                                                     // 1st return value
                } ELSE IF $current_state == $before {
                    DELETE $record_id;
                    RETURN {before_state: $current_state, update_success: true};                  // 1st return value
                } ELSE {
                    RETURN {before_state: $current_state, update_success: false};                 // 1st return value
                }
            };
            COMMIT TRANSACTION;
        "#;
        let result: Option<UpdateEngineStateResponse> = self
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
    ) -> Result<Option<UpdateEngineStateResponse>> {
        let sql = r#"
            let $record_id = type::thing($tb, $id);             // 0th return value

            BEGIN TRANSACTION;
            {
                LET $current_state = (SELECT VALUE info.state from only $record_id);
                IF $current_state IS NONE {
                    RETURN NONE;                                                     // 1st return value
                } ELSE IF $current_state == $before {
                    UPDATE ONLY $record_id SET info.state = $after;
                    RETURN {before_state: $current_state, update_success: true};                  // 1st return value
                } ELSE {
                    RETURN {before_state: $current_state, update_success: false};                 // 1st return value
                }
            };
            COMMIT TRANSACTION;
        "#;

        let before_state: Option<UpdateEngineStateResponse> = self
            .client
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("id", id.to_string()))
            // convert to vec because array cannot be serialized
            .bind(("before", before.clone()))
            .bind(("after", after.clone()))
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
            WHERE (info.state IN ["WaitToStart", "WaitToTerminate", "WaitToDelete"] OR info.state.ErrorWaitToClean)
                OR ((info.state IN ["Running", "StartInProgress", "TerminateInProgress", "DeleteInProgress"] OR info.state.ErrorCleanInProgress)
                    AND next_update_time < time::now());
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
