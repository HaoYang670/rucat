//! Shared state between handlers.

use ::std::sync::Arc;

use rucat_common::database::Database;

pub(crate) struct AppState<DB> {
    db: Arc<DB>,
}

// Manually implement `Clone` because derive(Clone) requires `DB: Clone`
impl<DB> Clone for AppState<DB> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}

impl<DB> AppState<DB>
where
    DB: Database,
{
    pub(crate) fn new(db: DB) -> Self {
        Self { db: Arc::new(db) }
    }

    pub(crate) fn get_db(&self) -> &DB {
        &self.db
    }
}
