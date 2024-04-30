//! Datastore to record clusters' infomation

use crate::cluster_router::{Cluster, ClusterId, ClusterInfo};
use rucat_common::error::{Result, RucatError};
use serde::Deserialize;
use surrealdb::{engine::local::Db, sql::Thing, Surreal};

type SurrealDBURI<'a> = &'a str;

/// Id of the [Cluster] in [DataStore]
#[derive(Debug, Deserialize)]
struct Record {
    id: Thing,
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
    Server { uri: SurrealDBURI<'a> },
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
    pub(crate) fn connect_serreal_db(uri: SurrealDBURI<'a>) -> Self {
        Self::Server { uri }
    }

    pub(crate) async fn add_cluster(&self, cluster: ClusterInfo) -> Result<ClusterId> {
        match self {
            Self::Embedded { store } => {
                // TODO: return an Option, not a Vec
                let record: Vec<Record> = store.create(Self::TABLE).content(cluster).await?;
                record.first().map_or(
                    Err(RucatError::DataStoreError("Add cluster fails".to_owned())),
                    |rd| Ok(rd.id.id.to_string()),
                )
            }
            Self::Server { .. } => todo!(),
        }
    }

    /// Return Ok(None) if the cluster does not exist
    pub(crate) async fn get_cluster(&self, id: &ClusterId) -> Result<Option<ClusterInfo>> {
        match self {
            Self::Embedded { store } => {
                // have to do this redundant format to pass the type checker
                Ok(store.select((Self::TABLE, id)).await?)
            }
            Self::Server { .. } => {
                todo!()
            }
        }
    }

    pub(crate) fn delete_cluster(&self, id: &ClusterId) -> Result<()> {
        todo!()
    }

    // the returned reference in Box has the same lifetime as self
    pub(crate) fn get_all_clusters(&self) -> Box<dyn Iterator<Item = &Cluster> + '_> {
        match self {
            DataStore::Embedded { .. } => todo!(),
            DataStore::Server { .. } => todo!(),
        }
    }
}

/// surreal db test for debug, need to remove.
#[test]
fn it_works() {
    use serde::{Deserialize, Serialize};
    use surrealdb::engine::local::Mem;
    use surrealdb::sql::Thing;
    use surrealdb::Surreal;

    #[derive(Debug, Serialize)]
    struct Name<'a> {
        first: &'a str,
        last: &'a str,
    }

    #[derive(Debug, Serialize)]
    struct Person<'a> {
        title: &'a str,
        name: Name<'a>,
        marketing: bool,
    }

    #[derive(Debug, Serialize)]
    struct Responsibility {
        marketing: bool,
    }

    #[derive(Debug, Deserialize)]
    struct Record {
        #[allow(dead_code)]
        id: Thing,
    }

    #[tokio::main]
    async fn main() -> surrealdb::Result<()> {
        // Create database connection
        let db = Surreal::new::<Mem>(()).await?;

        // Select a specific namespace / database
        db.use_ns("test").use_db("test").await?;

        // Create a new person with a random id
        let created: Vec<Record> = db
            .create("person")
            .content(Person {
                title: "Founder & CEO",
                name: Name {
                    first: "Tobie",
                    last: "Morgan Hitchcock",
                },
                marketing: true,
            })
            .await?;
        dbg!(created);

        // Update a person record with a specific id
        let updated: Option<Record> = db
            .update(("person", "jaime"))
            .merge(Responsibility { marketing: true })
            .await?;
        dbg!(updated);

        // Select all people records
        let people: Vec<Record> = db.select("person").await?;
        dbg!(people);

        // Perform a custom advanced query
        let groups = db
            .query("SELECT marketing, count() FROM type::table($table) GROUP BY marketing")
            .bind(("table", "person"))
            .await?;
        dbg!(groups);

        Ok(())
    }

    main().unwrap()
}
