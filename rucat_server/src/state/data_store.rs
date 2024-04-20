//! Datastore to record clusters' infomation

use std::collections::HashMap;

use crate::cluster_router::{Cluster, ClusterId};
use rucat_common::error::Result;

/// Store the metadata of Cluster
/// The lifetime here reprensent that of the endpoint of the SurrealDB.
// DO I need to add a rwlock for it? Or the mutable reference can make sure the r-w policy?
// the problem here is that `Self` is shared between threads.
// Achive the Component in Spring by using Axum::state?
#[derive(Clone)]
pub enum DataStore<'a> {
    InMemoryDataStore {
        store: HashMap<ClusterId, Cluster>,
    },
    /// I want to find a distributed database for storing.
    SurrealDB {
        endpoint: &'a str,
    },
}

impl<'a> DataStore<'a> {
    pub fn new_in_memory() -> Self {
        Self::InMemoryDataStore {
            store: HashMap::new(),
        }
    }

    fn add_cluster(&mut self, cluster: Cluster) -> Result<()> {
        let id = cluster.get_id();
        todo!()
    }

    fn get_cluster(&self, id: ClusterId) -> Option<&Cluster> {
        todo!()
    }

    fn delete_cluster(&mut self, id: ClusterId) -> Result<()> {
        todo!()
    }

    // the returned reference in Box has the same lifetime as self
    fn get_all_clusters(&self) -> Box<dyn Iterator<Item = &Cluster> + '_> {
        match self {
            DataStore::InMemoryDataStore { store } => Box::new(store.values()),
            DataStore::SurrealDB { endpoint: _ } => todo!(),
        }
    }
}
