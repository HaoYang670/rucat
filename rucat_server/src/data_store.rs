//! Datastore to record clusters' infomation

use crate::cluster_router::{Cluster, ClusterId};

/// Functions implementation should be atomic
trait Datastore {
    fn add_cluster(self, cluster: Cluster) -> Self;
    fn check_cluster(&self, id: ClusterId) -> bool;
    fn get_cluster(&self, id: ClusterId) -> Option<Cluster>;
    fn delete_cluster(self, id: ClusterId) -> Self;
    fn get_all_clusters(&self) -> impl IntoIterator<Item = Cluster>;
}
