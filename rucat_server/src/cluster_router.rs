use std::fmt::{Debug, Display};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use rucat_common::error::{Result, RucatError};
use serde::{Deserialize, Serialize};

use crate::state::{data_store::DataStore, AppState};
use ClusterState::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum ClusterState {
    /// Cluster is pending to be started.
    Pending,
    /// Cluster is running.
    Running,
    /// Cluster is stopped.
    Stopped,
}

impl Display for ClusterState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pending => write!(f, "Pending"),
            Running => write!(f, "Running"),
            Stopped => write!(f, "Stopped"),
        }
    }
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
pub(crate) struct ClusterInfo {
    name: String,
    cluster_type: ClusterType,
    state: ClusterState,
}

impl ClusterInfo {
    pub(crate) fn update_state(mut self, state: ClusterState) -> Self {
        self.state = state;
        self
    }
}

impl From<CreateClusterRequest> for ClusterInfo {
    fn from(value: CreateClusterRequest) -> Self {
        ClusterInfo {
            name: value.name,
            cluster_type: value.cluster_type,
            state: Pending,
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
        .ok_or_else(|| cluster_not_found(&id))
}

/// Stop a cluster to release resources. But cluster info is still kept in the data store.
/// TODO: make the state checking and updating atomic
async fn stop_cluster(Path(id): Path<ClusterId>, State(state): State<AppState<'_>>) -> Result<()> {
    let data_store = state.get_data_store();

    let cluster = get_cluster_helper(data_store, &id).await?;

    match cluster.state {
        Pending | Running => {
            data_store.update_cluster(&id, cluster.update_state(Stopped)).await
        },
        Stopped => {
            RucatError::NotAllowedError(format!("Cluster {} is already stopped", &id)).into()
        }
    }
}

/// Restart a stopped cluster with the same configuration.
/// TODO: make the state checking and updating atomic
async fn restart_cluster(Path(id): Path<ClusterId>, State(state): State<AppState<'_>>) -> Result<()> {
    let data_store = state.get_data_store();
    let cluster = get_cluster_helper(data_store, &id).await?;

    match cluster.state {
        Stopped => {
            data_store.update_cluster(&id, cluster.update_state(Pending)).await
        },
        other => {
            RucatError::NotAllowedError(format!("Cluster {} is in {} state, cannot be restart", &id, other)).into()
        }
    }
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
        .ok_or_else(|| cluster_not_found(&id))
}

async fn list_clusters(State(state): State<AppState<'_>>) -> Result<Json<Vec<ClusterId>>> {
    state.get_data_store().list_clusters().await.map(Json)
}

/// Pass the data store endpoint later
pub(crate) fn get_cluster_router() -> Router<AppState<'static>> {
    Router::new()
        .route("/", post(create_cluster).get(list_clusters))
        .route("/:id", get(get_cluster).delete(delete_cluster))
        .route("/:id/stop", post(stop_cluster))
        .route("/:id/restart", post(restart_cluster))
}



// ----------------- helper functions -----------------

/// helper function to create a NotFoundError
fn cluster_not_found(id: &ClusterId) -> RucatError {
    RucatError::NotFoundError(format!("Cluster {} not found", id))
}

async fn get_cluster_helper(data_store: &DataStore<'_>, id: &ClusterId) -> Result<ClusterInfo> {
    data_store
        .get_cluster(id)
        .await?
        .ok_or_else(|| cluster_not_found(id))
}
