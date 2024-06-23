//! Shared state between handlers.

use self::data_store::DataStore;

pub(crate) mod data_store;

#[derive(Clone)]
pub(crate) struct AppState<'a> {
    data_store: DataStore<'a>,
    engine_binary_path: String,
}

impl<'a> AppState<'a> {
    pub(crate) fn new(data_store: DataStore<'a>, engine_binary_path: String) -> Self {
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
