//! Datastore to record clusters' infomation

use crate::cluster_router::{ClusterId, ClusterInfo, ClusterState};
use rucat_common::error::{Result, RucatError};
use serde::Deserialize;
use surrealdb::{engine::local::Db, sql::Thing, Surreal};

type SurrealDBURI<'a> = &'a str;

/// Id of the [Cluster] in [DataStore]
#[derive(Debug, Deserialize)]
struct Record {
    id: Thing,
}

impl From<Record> for ClusterId {
    fn from(record: Record) -> Self {
        record.id.id.to_string()
    }
}

/// Store the metadata of Cluster
/// The lifetime here reprensent that of the URI of the DB server.
#[derive(Clone)]
pub(crate) enum DataStore<'a> {
    /// embedded database in memory
    Embedded {
        store: Surreal<Db>, //embedded surrealdb?
    },
    /// SurrealDB server
    Remote { uri: SurrealDBURI<'a> },
}

/// pub functions are those need to call outside from the rucat server (for example users need to construct a dataStore to create the rest server)
/// pub(crate) are those only called inside the rucat server
impl<'a> DataStore<'a> {
    const TABLE: &'static str = "clusters";

    /// use an in memory data store
    pub(crate) fn connect_embedded_db(db: Surreal<Db>) -> Self {
        Self::Embedded { store: db }
    }

    /// data store that connects to a SurrealDB
    pub(crate) fn connect_remote_db(uri: SurrealDBURI<'a>) -> Self {
        Self::Remote { uri }
    }

    pub(crate) async fn add_cluster(&self, cluster: ClusterInfo) -> Result<ClusterId> {
        match self {
            Self::Embedded { store } => {

                // TODO: return an Option, not a Vec
                let record: Vec<Record> = store.create(Self::TABLE).content(cluster).await?;
                record.first().map_or_else(
                    || Err(RucatError::DataStoreError("Add cluster fails".to_owned())),
                    |rd| Ok(rd.id.id.to_string()),
                )
            }
            Self::Remote { .. } => todo!(),
        }
    }

    pub(crate) async fn delete_cluster(&self, id: &ClusterId) -> Result<Option<ClusterInfo>> {
        match self {
            Self::Embedded { store } => Ok(store.delete((Self::TABLE, id)).await?),
            Self::Remote { .. } => {
                todo!()
            }
        }
    }

    /// Update the cluster with the given info
    pub(crate) async fn update_cluster(&self, id: &ClusterId, cluster: ClusterInfo) -> Result<()> {
        match self {
            Self::Embedded { store } => {
                let _: Option<Record> = store.update((Self::TABLE, id)).content(cluster).await?;
                Ok(())
            }
            Self::Remote { .. } => {
                todo!()
            }
        }
    }

    /// Return Ok(None) if the cluster does not exist
    pub(crate) async fn get_cluster(&self, id: &ClusterId) -> Result<Option<ClusterInfo>> {
        match self {
            Self::Embedded { store } => {
                // have to do this redundant format to pass the type checker
                Ok(store.select((Self::TABLE, id)).await?)
            }
            Self::Remote { .. } => {
                todo!()
            }
        }
    }

    /// Return a sorted list of all cluster ids
    pub(crate) async fn list_clusters(&self) -> Result<Vec<ClusterId>> {
        match self {
            DataStore::Embedded { store } => {
                let records: Vec<Record> = store.select(Self::TABLE).await?;
                let mut ids: Vec<ClusterId> = records.into_iter().map(Record::into).collect();

                ids.sort();
                Ok(ids)
            }
            DataStore::Remote { .. } => todo!(),
        }
    }
}
