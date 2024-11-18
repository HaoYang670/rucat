//! Client of SurrealDB

use axum::async_trait;

use crate::engine::EngineId;
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

use super::{DatabaseClient, UpdateEngineStateResponse};

/// Client to interact with the database.
/// Store the metadata of Engines
#[derive(Clone)]
pub struct SurrealDBClient {
    /// preserve `uri` for easily converting to [DatabaseType]
    uri: String,
    credentials: Option<Credentials>,
    client: Surreal<Client>,
}

impl SurrealDBClient {
    const TABLE: &'static str = "engines";
    const NAMESPACE: &'static str = "rucat";
    const DATABASE: &'static str = "rucat";
}

// TODO: replace #[async_trait] by #[trait_variant::make(HttpService: Send)] in the future: https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits.html#should-i-still-use-the-async_trait-macro
#[async_trait]
impl DatabaseClient for SurrealDBClient {
    fn get_uri(&self) -> &str {
        &self.uri
    }

    fn get_credentials(&self) -> Option<&Credentials> {
        self.credentials.as_ref()
    }

    async fn connect_local_db(credentials: Option<&Credentials>, uri: String) -> Result<Self> {
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
        Ok(Self {
            uri,
            client,
            credentials: credentials.cloned(),
        })
    }

    async fn add_engine(&self, engine: EngineInfo) -> Result<EngineId> {
        let sql = r#"
            CREATE ONLY type::table($table)
            SET info = $engine
            RETURN VALUE record::id(id);
        "#;

        let record: Option<String> = self
            .client
            .query(sql)
            .bind(("table", Self::TABLE))
            .bind(("engine", engine))
            .await
            .map_err(RucatError::fail_to_update_database)?
            .take(0)
            .map_err(RucatError::fail_to_update_database)?;
        let id = record.map(EngineId::try_from);
        id.unwrap_or_else(|| {
            RucatError::fail_to_update_database(anyhow!("Failed to create engine")).into()
        })
    }

    async fn delete_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
        let sql = r#"
            LET $id = type::thing($tb, $id);
            IF $id.exists() THEN
                SELECT VALUE info from (DELETE ONLY $id RETURN BEFORE)
            ELSE
                None
            END;
        "#;
        let result: Option<EngineInfo> = self
            .client
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("id", id.to_string()))
            .await
            .map_err(RucatError::fail_to_update_database)?
            .take(1)
            .map_err(RucatError::fail_to_update_database)?;
        Ok(result)
    }

    async fn update_engine_state(
        &self,
        id: &EngineId,
        before: Vec<EngineState>,
        after: EngineState,
    ) -> Result<Option<UpdateEngineStateResponse>> {
        // The query returns None if the engine does not exist
        // Throws an error if the engine state is not in the expected state
        // Otherwise, update the engine state and returns the engine state before update
        let sql = r#"
            let $record_id = type::thing($tb, $id);             // 0th return value
            BEGIN TRANSACTION;
            {
                LET $current_state = (SELECT VALUE info.state from only $record_id);
                IF $current_state IS NONE {
                    RETURN NONE;                                                     // 1st return value
                } ELSE IF $current_state IN $before {
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
            .bind(("before", before))
            .bind(("after", after))
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

    /// Return a sorted list of all engine ids
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
}
