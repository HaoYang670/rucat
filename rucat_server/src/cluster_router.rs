use std::fmt::{Debug, Display};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use rucat_common::error::Result;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ClusterInfo {
    name: String,
    cluster_type: ClusterType,
    state: ClusterState,
}

impl Display for ClusterInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl From<CreateClusterRequest> for ClusterInfo {
    fn from(value: CreateClusterRequest) -> Self {
        ClusterInfo {
            name: value.name,
            cluster_type: value.cluster_type,
            state: ClusterState::Pending,
        }
    }
}

impl From<ClusterInfo> for Bytes {
    fn from(value: ClusterInfo) -> Self {
        value.to_string().into()
    }
}

impl IntoResponse for ClusterInfo {
    fn into_response(self) -> axum::response::Response {
        Bytes::from(self).into_response()
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
    state.get_data_store().add_cluster(body.into()).await
}

async fn delete_cluster(State(state): State<AppState<'_>>) {
    todo!()
}

async fn stop_cluster(id: ClusterId, State(state): State<AppState<'_>>) {
    todo!()
}

async fn start_cluster(id: ClusterId, State(state): State<AppState<'_>>) {
    todo!()
}

async fn restart_cluster(id: ClusterId, State(state): State<AppState<'_>>) {
    todo!()
}

async fn get_cluster(
    Path(id): Path<ClusterId>,
    State(state): State<AppState<'_>>,
) -> Result<ClusterInfo> {
    state.get_data_store().get_cluster(&id).await
}

async fn list_clusters(State(state): State<AppState<'_>>) {
    todo!()
}

/// Pass the data store endpoint later
pub fn get_cluster_router() -> Router<AppState<'static>> {
    Router::new()
        .route("/", post(create_cluster))
        .route("/:id", get(get_cluster))
}
