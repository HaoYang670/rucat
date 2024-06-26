//! Datastore to record engines' infomation

use crate::engine::router::{EngineInfo, EngineState};
use rucat_common::{
    error::{Result, RucatError},
    EngineId,
};
use serde::Deserialize;
use surrealdb::{engine::local::Db, Surreal};

type SurrealDBURI = &'static str;

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
pub(crate) enum DataStore {
    /// embedded database in memory
    Embedded {
        store: Surreal<Db>, //embedded surrealdb?
    },
    /// SurrealDB server
    Remote { uri: SurrealDBURI },
}

/// pub functions are those need to call outside from the rucat server (for example users need to construct a dataStore to create the rest server)
/// pub(crate) are those only called inside the rucat server
impl DataStore {
    const TABLE: &'static str = "engines";

    /// use an in memory data store
    pub(crate) fn connect_embedded_db(db: Surreal<Db>) -> Self {
        Self::Embedded { store: db }
    }

    /// data store that connects to a SurrealDB
    pub(crate) fn connect_remote_db(uri: SurrealDBURI) -> Self {
        Self::Remote { uri }
    }

    pub(crate) async fn add_engine(&self, engine: EngineInfo) -> Result<EngineId> {
        match self {
            Self::Embedded { store } => {
                let sql = r#"
                    CREATE ONLY type::table($table)
                    SET info = $engine
                    RETURN VALUE meta::id(id);
                "#;

                let record: Option<EngineId> = store
                    .query(sql)
                    .bind(("table", Self::TABLE))
                    .bind(("engine", engine))
                    .await?
                    .take(0)?;
                record.ok_or_else(|| RucatError::DataStoreError("Add engine fails".to_owned()))
            }
            Self::Remote { .. } => todo!(),
        }
    }

    pub(crate) async fn delete_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
        match self {
            Self::Embedded { store } => {
                let sql = r#"
                    SELECT VALUE info from
                    (DELETE ONLY type::thing($tb, $id) RETURN BEFORE);
                "#;
                let result: Option<EngineInfo> = store
                    .query(sql)
                    .bind(("tb", Self::TABLE))
                    .bind(("id", id))
                    .await?
                    .take(0)?;
                Ok(result)
            }
            Self::Remote { .. } => {
                todo!()
            }
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
        match self {
            Self::Embedded { store } => {
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

                let before_state: Option<UpdateEngineStateResponse> = store
                    .query(sql)
                    .bind(("tb", Self::TABLE))
                    .bind(("id", id))
                    // convert to vec because array cannot be serialized
                    .bind(("before", before.to_vec()))
                    .bind(("after", after))
                    .await?
                    .take(2)?; // The 3rd statement is the if-else which is what we want

                Ok(before_state)
            }
            Self::Remote { .. } => {
                todo!()
            }
        }
    }

    /// Return Ok(None) if the engine does not exist
    pub(crate) async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>> {
        match self {
            Self::Embedded { store } => {
                let sql = r#"
                    SELECT VALUE info
                    FROM ONLY type::thing($tb, $id);
                "#;
                let info: Option<EngineInfo> = store
                    .query(sql)
                    .bind(("tb", Self::TABLE))
                    .bind(("id", id))
                    .await?
                    .take(0)?;
                Ok(info)
            }
            Self::Remote { .. } => {
                todo!()
            }
        }
    }

    /// Return a sorted list of all engine ids
    pub(crate) async fn list_engines(&self) -> Result<Vec<EngineId>> {
        match self {
            DataStore::Embedded { store } => {
                let sql = r#"
                    SELECT VALUE meta::id(id) FROM type::table($tb);
                "#;

                let mut ids: Vec<EngineId> =
                    store.query(sql).bind(("tb", Self::TABLE)).await?.take(0)?;
                ids.sort();
                Ok(ids)
            }
            DataStore::Remote { .. } => todo!(),
        }
    }
}
