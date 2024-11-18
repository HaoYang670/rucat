//! Shared state between handlers.

use rucat_common::database::DatabaseClient;

#[derive(Clone)]
pub(crate) struct AppState<DB: DatabaseClient> {
    db: DB,
}

impl<DB: DatabaseClient> AppState<DB> {
    pub(crate) fn new(db: DB) -> Self {
        Self { db }
    }

    pub(crate) fn get_db(&self) -> &DB {
        &self.db
    }
}
