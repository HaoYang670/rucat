//! Shared state between handlers.

use self::data_store::DataStore;

pub mod data_store;

#[derive(Clone)]
pub struct AppState<'a> {
    data_store: DataStore<'a>,
}

impl<'a> AppState<'a> {
    pub fn new(data_store: DataStore<'a>) -> Self {
        Self { data_store }
    }

    pub fn get_data_store(&self) -> &DataStore {
        &self.data_store
    }
}
