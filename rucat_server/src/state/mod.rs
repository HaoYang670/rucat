//! Shared state between handlers.

use ::std::sync::Arc;

use rucat_common::database::DatabaseClient;

pub(crate) struct AppState<DB: DatabaseClient> {
    db: Arc<DB>,
}

// TODO: I don't know why derive(Clone) is not working here.
impl<DB: DatabaseClient> Clone for AppState<DB> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}

impl<DB: DatabaseClient> AppState<DB> {
    pub(crate) fn new(db: DB) -> Self {
        Self { db: Arc::new(db) }
    }

    pub(crate) fn get_db(&self) -> &DB {
        &self.db
    }
}
