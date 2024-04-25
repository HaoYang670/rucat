use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Clone)]
enum ClusterState {
    RUNNING,
    ERROR,
    SLEEPING,
}

/// Ballista first on k8s.
#[derive(Clone)]
enum ClusterType {
    Ballista,
    Rucat,
}

#[derive(Clone)]
pub(super) struct ClusterInfo {
    name: String,
    cluster_type: ClusterType,
    state: ClusterState,
}

pub(crate) type ClusterId = u8;

#[derive(Clone)]
pub struct Cluster {
    id: ClusterId,
    info: ClusterInfo,
}

impl Cluster {
    pub fn get_id(&self) -> ClusterId {
        self.id
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateClusterRequest {
    name: String,
}

/// create a cluster with cluster name in the request body
async fn create_cluster(
    State(state): State<AppState<'_>>,
    Json(body): Json<CreateClusterRequest>,
) -> String {
    format!("Create a cluster with name {}", body.name)
}

async fn delete_cluster(State(state): State<AppState<'_>>) -> () {
    todo!()
}

async fn stop_cluster(id: ClusterId, State(state): State<AppState<'_>>) -> () {
    todo!()
}

async fn start_cluster(id: ClusterId, State(state): State<AppState<'_>>) -> () {
    todo!()
}

async fn restart_cluster(id: ClusterId, State(state): State<AppState<'_>>) -> () {
    todo!()
}

async fn get_cluster(id: ClusterId, State(state): State<AppState<'_>>) -> () {
    todo!()
}

async fn list_clusters(State(state): State<AppState<'_>>) -> () {
    todo!()
}

/// Pass the data store endpoint later
pub fn get_cluster_router() -> Router<AppState<'static>> {
    Router::new().route("/", post(create_cluster).delete(delete_cluster))
}
