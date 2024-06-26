//! Shared state between handlers.

use self::data_store::DataStore;

pub(crate) mod data_store;

#[derive(Clone)]
pub(crate) struct AppState {
    data_store: DataStore,
    engine_binary_path: String,
}

impl AppState {
    pub(crate) fn new(data_store: DataStore, engine_binary_path: String) -> Self {
        Self {
            data_store,
            engine_binary_path,
        }
    }

    pub(crate) fn get_data_store(&self) -> &DataStore {
        &self.data_store
    }

    pub(crate) fn get_engine_binary_path(&self) -> &str {
        self.engine_binary_path.as_str()
    }
}
