use std::fmt::{Debug, Display};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use bytes::Bytes;
use rucat_common::error::{Result, RucatError};
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
    /// Ballista in local mode
    BallistaLocal,
    /// Ballista in remote mode, e.g. on k8s.
    BallistaRemote,
    Rucat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            state: ClusterState::Pending,
        }
    }
}

pub(crate) type ClusterId = String;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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

async fn delete_cluster(
    Path(id): Path<ClusterId>,
    State(state): State<AppState<'_>>,
) -> Result<()> {
    state
        .get_data_store()
        .delete_cluster(&id)
        .await?
        .map(|_| ())
        .ok_or_else(|| RucatError::NotFoundError(format!("Cluster {} not found", id)))
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
) -> Result<Json<ClusterInfo>> {
    state
        .get_data_store()
        .get_cluster(&id)
        .await?
        .map(Json)
        .ok_or_else(|| RucatError::NotFoundError(format!("Cluster {} not found", id)))
}

async fn list_clusters(State(state): State<AppState<'_>>) -> Result<Json<Vec<ClusterId>>> {
    state.get_data_store().get_all_clusters().await.map(Json)
}

/// Pass the data store endpoint later
pub(crate) fn get_cluster_router() -> Router<AppState<'static>> {
    Router::new()
        .route("/", post(create_cluster).get(list_clusters))
        .route("/:id", get(get_cluster).delete(delete_cluster))
}
