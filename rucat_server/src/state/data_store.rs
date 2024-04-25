//! Datastore to record clusters' infomation

use std::collections::HashMap;

use crate::cluster_router::{Cluster, ClusterId, ClusterInfo};
use rucat_common::error::Result;

type SurrealDBURI<'a> = &'a str;

/// Store the metadata of Cluster
/// The lifetime here reprensent that of the uri of the SurrealDB.
#[derive(Clone)]
pub enum DataStore<'a> {
    InMemoryDataStore {
        store: HashMap<ClusterId, Cluster>, // ConcurrentMap<ClusterId, RWLock<Cluster>>
    },
    /// I want to find a distributed database for storing.
    SurrealDB { uri: SurrealDBURI<'a> },
}

/// pub functions are those need to call outside from the rucat server (for example users need to construct a dataStore to create the rest server)
/// pub(crate) are those only called inside the rucat server
impl<'a> DataStore<'a> {
    /// empty im memory data store
    pub fn new_in_memory() -> Self {
        Self::InMemoryDataStore {
            store: HashMap::new(),
        }
    }

    /// data store that connects to a SurrealDB
    pub fn connect_serreal_db(uri: SurrealDBURI<'a>) -> Self {
        Self::SurrealDB { uri }
    }

    pub(crate) fn add_cluster(&mut self, cluster: ClusterInfo) -> ClusterId {
        todo!()
    }

    pub(crate) fn get_cluster(&self, id: ClusterId) -> Option<&Cluster> {
        todo!()
    }

    pub(crate) fn delete_cluster(&mut self, id: ClusterId) -> Result<()> {
        todo!()
    }

    // the returned reference in Box has the same lifetime as self
    pub(crate) fn get_all_clusters(&self) -> Box<dyn Iterator<Item = &Cluster> + '_> {
        match self {
            DataStore::InMemoryDataStore { store } => Box::new(store.values()),
            DataStore::SurrealDB { .. } => todo!(),
        }
    }
}
