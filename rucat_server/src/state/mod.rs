//! Shared state between handlers.

use rucat_common::database::DatabaseClient;

#[derive(Clone)]
pub(crate) struct AppState {
    db: DatabaseClient,
}

impl AppState {
    pub(crate) fn new(db: DatabaseClient) -> Self {
        Self { db }
    }

    pub(crate) fn get_db(&self) -> &DatabaseClient {
        &self.db
    }
}
