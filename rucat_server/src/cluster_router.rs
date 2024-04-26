use axum::{extract::State, routing::post, Json, Router};
use rucat_common::error::Result;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Clone, Serialize, Deserialize)]
enum ClusterState {
    Pending,
    Running,
}

/// Ballista first on k8s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ClusterType {
    Ballista,
    Rucat,
}

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct ClusterInfo {
    name: String,
    cluster_type: ClusterType,
    state: ClusterState,
}

impl From<CreateClusterRequest> for ClusterInfo {
    fn from(value: CreateClusterRequest) -> Self {
        ClusterInfo {
            name: value.name,
            cluster_type: value.cluster_type,
            state: ClusterState::Running,
        }
    }
}

pub(crate) type ClusterId = String;

#[derive(Clone, Deserialize)]
pub(crate) struct Cluster {
    id: ClusterId,
    info: ClusterInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CreateClusterRequest {
    name: String,
    cluster_type: ClusterType,
}

/// create a cluster with cluster name in the request body
async fn create_cluster(
    State(state): State<AppState<'_>>,
    Json(body): Json<CreateClusterRequest>,
) -> Result<ClusterId> {
    let data_store = state.get_data_store();
    data_store.add_cluster(body.into()).await
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
