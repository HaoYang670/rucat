//! Shared state between handlers.

use rucat_common::database::DataBase;

#[derive(Clone)]
pub(crate) struct AppState {
    db: DataBase,
    engine_binary_path: String,
}

impl AppState {
    pub(crate) fn new(db: DataBase, engine_binary_path: String) -> Self {
        Self {
            db,
            engine_binary_path,
        }
    }

    pub(crate) fn get_db(&self) -> &DataBase {
        &self.db
    }

    pub(crate) fn get_engine_binary_path(&self) -> &str {
        self.engine_binary_path.as_str()
    }
}
