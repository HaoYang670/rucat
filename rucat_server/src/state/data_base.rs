//! Datastore to record engines' infomation

use crate::engine::router::{EngineInfo, EngineState};
use rucat_common::{
    error::{Result, RucatError},
    EngineId,
};
use serde::Deserialize;
use surrealdb::{
    engine::{
        local::{Db, Mem},
        remote::ws::{Client, Ws},
    },
    Surreal,
};
use DataBase::*;

/// Response of updating engine state
/// The response contains the engine state before the update
/// and whether the update is successful.
#[derive(Deserialize)]
pub(crate) struct UpdateEngineStateResponse {
    before: EngineState,
    success: bool,
}

impl UpdateEngineStateResponse {
    pub(crate) fn update_success(&self) -> bool {
        self.success
    }

    pub(crate) fn get_before_state(&self) -> &EngineState {
        &self.before
    }
}

/// Store the metadata of Engine
/// The lifetime here reprensent that of the URI of the DB server.
#[derive(Clone)]
pub(crate) enum DataBase {
    /// embedded database in memory
    Embedded(Surreal<Db>),
    /// local database
    Local(Surreal<Client>),
}

/// pub functions are those need to call outside from the rucat server (for example users need to construct a dataStore to create the rest server)
/// pub(crate) are those only called inside the rucat server
impl DataBase {
    const TABLE: &'static str = "engines";
    const NAMESPACE: &'static str = "rucat";
    const DATABASE: &'static str = "rucat";

    /// use an in memory data store
    pub(crate) async fn create_embedded_db() -> Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(Self::NAMESPACE).use_db(Self::DATABASE).await?;
        Ok(Embedded(db))
    }

    /// data store that connects to a SurrealDB
    pub(crate) async fn connect_local_db(uri: String) -> Result<Self> {
        let db = Surreal::new::<Ws>(uri).await?;
        db.use_ns(Self::NAMESPACE).use_db(Self::DATABASE).await?;
        Ok(Local(db))
    }

    pub(crate) async fn add_engine(&self, engine: EngineInfo) -> Result<EngineId> {
        macro_rules! execute_sql {
            ($db:expr) => {{
                let sql = r#"
                    CREATE ONLY type::table($table)
                    SET info = $engine
                    RETURN VALUE meta::id(id);
                "#;

                let record: Option<EngineId> = $db
                    .query(sql)
                    .bind(("table", Self::TABLE))
                    .bind(("engine", engine))
                    .await?
                    .take(0)?;
                record.ok_or_else(|| RucatError::DataStoreError("Add engine fails".to_owned()))
            }};
        }

        match self {
            Embedded(db) => execute_sql!(db),
            Local(db) => execute_sql!(db),
        }
    }

    pub(crate) async fn delete_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
        macro_rules! execute_sql {
            ($db:expr) => {{
                let sql = r#"
                    SELECT VALUE info from
                    (DELETE ONLY type::thing($tb, $id) RETURN BEFORE);
                "#;
                let result: Option<EngineInfo> = $db
                    .query(sql)
                    .bind(("tb", Self::TABLE))
                    .bind(("id", id))
                    .await?
                    .take(0)?;
                Ok(result)
            }};
        }

        match self {
            Embedded(db) => execute_sql!(db),
            Local(db) => execute_sql!(db),
        }
    }

    /// Update the engine state to **after** only when
    /// the engine exists and the current state is the same as the **before**.
    /// Return the engine state before the update.
    /// Return None if the engine does not exist.
    /// Throws an error if the engine state is not in the expected state.
    pub(crate) async fn update_engine_state<const N: usize>(
        &self,
        id: &EngineId,
        before: [EngineState; N],
        after: EngineState,
    ) -> Result<Option<UpdateEngineStateResponse>> {
        macro_rules! execute_sql {
            ($db:expr) => {{
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
                        UPDATE ONLY $record_id SET info.state = $after;
                        RETURN {before: $current_state, success: true};                  // 2nd return value
                    } ELSE {
                        RETURN {before: $current_state, success: false};                 // 2nd return value
                    };
                    COMMIT TRANSACTION;
                "#;

                let before_state: Option<UpdateEngineStateResponse> = $db
                    .query(sql)
                    .bind(("tb", Self::TABLE))
                    .bind(("id", id))
                    // convert to vec because array cannot be serialized
                    .bind(("before", before.to_vec()))
                    .bind(("after", after))
                    .await?
                    .take(2)?; // The 3rd statement is the if-else which is what we want

                Ok(before_state)
            }};
        }

        match self {
            Embedded(db) => execute_sql!(db),
            Local(db) => execute_sql!(db),
        }
    }

    /// Return Ok(None) if the engine does not exist
    pub(crate) async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
        macro_rules! execute_sql {
            ($db:expr) => {{
                let sql = r#"
                    SELECT VALUE info
                    FROM ONLY type::thing($tb, $id);
                "#;
                let info: Option<EngineInfo> = $db
                    .query(sql)
                    .bind(("tb", Self::TABLE))
                    .bind(("id", id))
                    .await?
                    .take(0)?;
                Ok(info)
            }};
        }

        match self {
            Embedded(db) => execute_sql!(db),
            Local(db) => execute_sql!(db),
        }
    }

    /// Return a sorted list of all engine ids
    pub(crate) async fn list_engines(&self) -> Result<Vec<EngineId>> {
        macro_rules! execute_sql {
            ($db:expr) => {{
                let sql = r#"
                    SELECT VALUE meta::id(id) FROM type::table($tb);
                "#;

                let mut ids: Vec<EngineId> =
                    $db.query(sql).bind(("tb", Self::TABLE)).await?.take(0)?;
                ids.sort();
                Ok(ids)
            }};
        }

        match self {
            Embedded(db) => execute_sql!(db),
            Local(db) => execute_sql!(db),
        }
    }
}
