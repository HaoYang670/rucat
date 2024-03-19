use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CreateClusterRequest {
    name: String,
}

/// create a cluster with cluster name in the request body
async fn create_cluster(Json(body): Json<CreateClusterRequest>) -> String {
    format!("Create a cluster with name {}", body.name)
}

pub fn get_cluster_router() -> Router {
    Router::new().route("/", post(create_cluster))
}
