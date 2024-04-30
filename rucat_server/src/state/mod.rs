//! Shared state between handlers.

use self::data_store::DataStore;

pub(crate) mod data_store;

#[derive(Clone)]
pub(crate) struct AppState<'a> {
    data_store: DataStore<'a>,
}

impl<'a> AppState<'a> {
    pub(crate) fn new(data_store: DataStore<'a>) -> Self {
        Self { data_store }
    }

    pub(crate) fn get_data_store(&self) -> &DataStore {
        &self.data_store
    }
}
