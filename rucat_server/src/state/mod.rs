//! Shared state between handlers.

use ::std::sync::Arc;

use rucat_common::database::Database;

pub(crate) struct AppState<DB: Database> {
    db: Arc<DB>,
}

// TODO: I don't know why derive(Clone) is not working here.
impl<DB: Database> Clone for AppState<DB> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}

impl<DB: Database> AppState<DB> {
    pub(crate) fn new(db: DB) -> Self {
        Self { db: Arc::new(db) }
    }

    pub(crate) fn get_db(&self) -> &DB {
        &self.db
    }
}
