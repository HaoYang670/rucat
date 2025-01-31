//! Shared state between handlers.

use ::std::sync::Arc;

use rucat_common::database::Database;

use crate::Authenticate;

pub(crate) struct AppState<DB, AuthProvider> {
    db: Arc<DB>,
    auth_provider: Option<Arc<AuthProvider>>,
}

// TODO: I don't know why derive(Clone) is not working here.
impl<DB, AuthProvider> Clone for AppState<DB, AuthProvider> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            auth_provider: self.auth_provider.clone(),
        }
    }
}

impl<DB, AuthProvider> AppState<DB, AuthProvider>
where
    DB: Database,
    AuthProvider: Authenticate,
{
    pub(crate) fn new(db: DB, auth_provider: Option<AuthProvider>) -> Self {
        Self {
            db: Arc::new(db),
            auth_provider: auth_provider.map(Arc::new),
        }
    }

    pub(crate) fn get_db(&self) -> &DB {
        &self.db
    }

    pub(crate) fn get_auth_provider(&self) -> Option<&AuthProvider> {
        self.auth_provider.as_deref()
    }
}
