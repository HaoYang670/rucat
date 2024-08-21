//! Datastore to record engines' information

use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use crate::engine::{EngineConnection, EngineInfo, EngineState};
use crate::error::{Result, RucatError};
use crate::EngineId;
use rand::Rng;
use serde::Deserialize;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};

/// Response of updating an engine state.
/// # Fields
/// - `before_state`: The engine state before the update.
/// - `update_success`: Whether the update is successful.
#[derive(Deserialize)]
pub struct UpdateEngineStateResponse {
    pub before_state: EngineState,
    pub update_success: bool,
}

/// Store the metadata of Engines
#[derive(Clone)]
pub struct DataBase {
    /// preserve `address` for easily converting to [DatabaseType]
    address: String,
    /// database client
    db: Surreal<Client>,
}

impl DataBase {
    const TABLE: &'static str = "engines";
    const NAMESPACE: &'static str = "rucat";
    const DATABASE: &'static str = "rucat";
    const MAX_ATTEMPTS_TO_CONNECT_EMBEDDED_DB: u8 = 10;

    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// embedded db will be killed when the server is killed
    /// Return the [DataBase] and the db process.
    pub async fn create_embedded_db() -> Result<(Self, Child)> {
        // create db using command line and connect to it.
        let port = rand::thread_rng().gen_range(1024..=65535);
        let address = format!("127.0.0.1:{}", port);
        let process = Command::new("surreal")
            .args(["start", "-b", &address, "--log", "none"])
            // TODO: store database's log in a file
            .stdout(Stdio::null())
            .spawn()?;

        // Wait for the database to be ready
        let mut attempts = 0;
        let delay = Duration::from_secs(1);

        loop {
            match Self::connect_local_db(address.clone()).await {
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
    pub async fn connect_local_db(address: String) -> Result<Self> {
        let db = Surreal::new::<Ws>(&address).await?;
        db.use_ns(Self::NAMESPACE).use_db(Self::DATABASE).await?;
        Ok(Self { address, db })
    }

    pub async fn add_engine(&self, engine: EngineInfo) -> Result<EngineId> {
        let sql = r#"
            CREATE ONLY type::table($table)
            SET info = $engine
            RETURN VALUE meta::id(id);
        "#;

        let record: Option<String> = self
            .db
            .query(sql)
            .bind(("table", Self::TABLE))
            .bind(("engine", engine))
            .await?
            .take(0)?;
        let id = record.map(EngineId::from);
        id.ok_or_else(|| RucatError::DataStoreError("Add engine fails".to_owned()))
    }

    pub async fn delete_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
        let sql = r#"
            SELECT VALUE info from
            (DELETE ONLY type::thing($tb, $id) RETURN BEFORE);
        "#;
        let result: Option<EngineInfo> = self
            .db
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("id", id.as_str().to_owned()))
            .await?
            .take(0)?;
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
        // There are some cases that we need to update the endpoint
        // e.g. updating from Pending to Running
        connection: Option<EngineConnection>,
    ) -> Result<Option<UpdateEngineStateResponse>> {
        // The query returns None if the engine does not exist
        // Throws an error if the engine state is not in the expected state
        // Otherwise, update the engine state and returns the engine state before update
        let sql = r#"
            let $record_id = type::thing($tb, $id);             // 0th return value
            BEGIN TRANSACTION;
            LET $current_state = (SELECT VALUE info.state from only $record_id); // 1st return value
            IF $current_state IS NONE {
                RETURN NONE;                                                     // 2nd return value
            } ELSE IF $current_state IN $before {
                UPDATE ONLY $record_id SET info.state = $after, info.connection = $connection;
                RETURN {before_state: $current_state, update_success: true};                  // 2nd return value
            } ELSE {
                RETURN {before_state: $current_state, update_success: false};                 // 2nd return value
            };
            COMMIT TRANSACTION;
        "#;

        let before_state: Option<UpdateEngineStateResponse> = self
            .db
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("id", id.as_str().to_owned()))
            // convert to vec because array cannot be serialized
            .bind(("before", before.to_vec()))
            .bind(("after", after))
            .bind(("connection", connection))
            .await?
            .take(2)?; // The 3rd statement is the if-else which is what we want

        Ok(before_state)
    }

    /// Return `Ok(None)` if the engine does not exist
    pub async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
        let sql = r#"
            SELECT VALUE info
            FROM ONLY type::thing($tb, $id);
        "#;
        let info: Option<EngineInfo> = self
            .db
            .query(sql)
            .bind(("tb", Self::TABLE))
            .bind(("id", id.as_str().to_owned()))
            .await?
            .take(0)?;
        Ok(info)
    }

    /// Return a sorted list of all engine ids
    pub async fn list_engines(&self) -> Result<Vec<EngineId>> {
        let sql = r#"
            SELECT VALUE meta::id(id) FROM type::table($tb);
        "#;

        let ids: Vec<String> = self
            .db
            .query(sql)
            .bind(("tb", Self::TABLE))
            .await?
            .take(0)?;
        let mut ids: Vec<_> = ids.into_iter().map(EngineId::from).collect();
        ids.sort();
        Ok(ids)
    }
}
