//! Datastore to record engines' information

use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use crate::engine::EngineId;
use crate::error::{Result, RucatError};
use crate::{
    config::Credentials,
    engine::{EngineInfo, EngineState},
};
use ::anyhow::anyhow;
use ::surrealdb::opt::auth::Root;
use rand::Rng;
use serde::Deserialize;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};
use tracing::error;

/// Response of updating an engine state.
/// # Fields
/// - `before_state`: The engine state before the update.
/// - `update_success`: Whether the update is successful.
#[derive(Deserialize)]
pub struct UpdateEngineStateResponse {
    pub before_state: EngineState,
    pub update_success: bool,
}

/// Client to interact with the database.
/// Store the metadata of Engines
#[derive(Clone)]
pub struct DatabaseClient {
    /// preserve `uri` for easily converting to [DatabaseType]
    uri: String,
    credentials: Option<Credentials>,
    client: Surreal<Client>,
}

impl DatabaseClient {
    const TABLE: &'static str = "engines";
    const NAMESPACE: &'static str = "rucat";
    const DATABASE: &'static str = "rucat";
    const MAX_ATTEMPTS_TO_CONNECT_EMBEDDED_DB: u8 = 10;

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn get_credentials(&self) -> Option<&Credentials> {
        self.credentials.as_ref()
    }

    /// embedded db will be killed when the server is killed
    /// Return the [DataBase] and the db process.
    pub async fn create_embedded_db(credentials: Option<&Credentials>) -> Result<(Self, Child)> {
        // create db using command line and connect to it.
        let address = {
            let port = rand::thread_rng().gen_range(1024..=65535);
            format!("127.0.0.1:{}", port)
        };
        let args = {
            let mut authentication_args = match credentials {
                Some(Credentials { username, password }) => vec!["-u", username, "-p", password],
                None => vec!["--unauthenticated"],
            };
            let mut args = vec!["start", "-b", &address, "--log", "none"];
            args.append(&mut authentication_args);
            args
        };
        let process = Command::new("surreal")
            .args(args)
            // TODO: store database's log in a file
            .stdout(Stdio::null())
            .spawn()
            .map_err(RucatError::fail_to_create_database)
            .inspect_err(|e| error!("{}", e))?;

        // Wait for the database to be ready
        let mut attempts = 0;
        let delay = Duration::from_secs(1);

        loop {
            match Self::connect_local_db(credentials, address.clone()).await {
                Ok(db) => return Ok((db, process)),
                Err(_) if attempts < Self::MAX_ATTEMPTS_TO_CONNECT_EMBEDDED_DB => {
                    attempts += 1;
                    sleep(delay);
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// data store that connects to a SurrealDB
    pub async fn connect_local_db(credentials: Option<&Credentials>, uri: String) -> Result<Self> {
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

    pub async fn add_engine(&self, engine: EngineInfo) -> Result<EngineId> {
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
        let id = record.map(EngineId::from);
        id.ok_or_else(|| RucatError::fail_to_update_database(anyhow!("Failed to create engine")))
    }

    pub async fn delete_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
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

    /// Update the engine state to `after` only when
    /// the engine exists and the current state is in `before`.
    /// # Return
    /// - `Ok(None)` if the engine does not exist.
    /// - `Ok(Some(UpdateEngineStateResponse))` if the engine exists.
    /// - `Err(_)` if any error occurs in the database.
    pub async fn update_engine_state<const N: usize>(
        &self,
        id: &EngineId,
        before: [EngineState; N],
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
            .bind(("before", before.to_vec()))
            .bind(("after", after))
            .await
            .map_err(RucatError::fail_to_update_database)?
            .take(1)
            .map_err(RucatError::fail_to_update_database)?; // The 1st statement is the if-else which is what we want

        Ok(before_state)
    }

    /// Return `Ok(None)` if the engine does not exist
    pub async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
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
    pub async fn list_engines(&self) -> Result<Vec<EngineId>> {
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
        let mut ids: Vec<_> = ids.into_iter().map(EngineId::from).collect();
        ids.sort();
        Ok(ids)
    }
}
