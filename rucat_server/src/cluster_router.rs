use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

enum ClusterState {
    RUNNING,
    ERROR,
    SLEEPING,
}

/// For future
enum ClusterType {
    Datafusion,
    Rucat,
}

pub(crate) type ClusterId = u8;

pub(crate) struct Cluster<'a> {
    name: &'a str,
    id: ClusterId,
    state: ClusterState,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateClusterRequest {
    name: String,
}

/// create a cluster with cluster name in the request body
async fn create_cluster(Json(body): Json<CreateClusterRequest>) -> String {
    format!("Create a cluster with name {}", body.name)
}

async fn delete_cluster() -> () {
    todo!()
}

async fn stop_cluster(id: ClusterId) -> () {
    todo!()
}

async fn start_cluster(id: ClusterId) -> () {
    todo!()
}

async fn restart_cluster(id: ClusterId) -> () {
    todo!()
}

async fn get_cluster(id: ClusterId) -> () {
    todo!()
}

async fn list_clusters() -> () {
    todo!()
}

pub fn get_cluster_router() -> Router {
    Router::new().route("/", post(create_cluster).delete(delete_cluster))
}
