//! Shared state between handlers.

use rucat_common::database::DatabaseClient;

#[derive(Clone)]
pub(crate) struct AppState {
    db: DatabaseClient,
    engine_binary_path: String,
}

impl AppState {
    pub(crate) fn new(db: DatabaseClient, engine_binary_path: String) -> Self {
        Self {
            db,
            engine_binary_path,
        }
    }

    pub(crate) fn get_db(&self) -> &DatabaseClient {
        &self.db
    }

    pub(crate) fn get_engine_binary_path(&self) -> &str {
        self.engine_binary_path.as_str()
    }
}
