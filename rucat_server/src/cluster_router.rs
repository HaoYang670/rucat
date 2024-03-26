use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

enum ClusterState {
    RUNNING,
    SLEEPING,
}

struct ClusterId(u8);

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

async fn stop_cluster() -> () {
    todo!()
}

async fn start_cluster() -> () {
    todo!()
}

async fn restart_cluster() -> () {
    todo!()
}

async fn get_cluster() -> () {
    todo!()
}

async fn list_clusters() -> () {
    todo!()
}

pub fn get_cluster_router() -> Router {
    Router::new().route("/", post(create_cluster).delete(delete_cluster))
}
