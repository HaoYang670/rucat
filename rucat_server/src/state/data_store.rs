//! Datastore to record clusters' infomation

use crate::cluster_router::{Cluster, ClusterId, ClusterInfo};
use rucat_common::error::Result;
use surrealdb::{engine::local::Db, Surreal};

type SurrealDBURI<'a> = &'a str;

/// Store the metadata of Cluster
/// The lifetime here reprensent that of the URI of the DB server.
#[derive(Clone)]
pub enum DataStore<'a> {
    /// embedded database in memory
    Embedded {
        store: Surreal<Db>, //embedded surrealdb?
    },
    /// SurrealDB server
    Server { uri: SurrealDBURI<'a> },
}

/// pub functions are those need to call outside from the rucat server (for example users need to construct a dataStore to create the rest server)
/// pub(crate) are those only called inside the rucat server
impl<'a> DataStore<'a> {
    /// empty im memory data store
    pub fn connect_embedded_db(db: Surreal<Db>) -> Self {
        Self::Embedded {
            store: db,
        }
    }

    /// data store that connects to a SurrealDB
    pub fn connect_serreal_db(uri: SurrealDBURI<'a>) -> Self {
        Self::Server { uri }
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
            DataStore::Embedded { store } => todo!(),
            DataStore::Server { .. } => todo!(),
        }
    }
}
