use authentication::auth;
use axum::{extract::State, middleware, routing::get, Router};
use cluster_router::get_cluster_router;
use rucat_common::error::Result;
use state::{data_store::DataStore, AppState};
use surrealdb::{engine::local::Mem, Surreal};
use tower_http::trace::TraceLayer;

pub(crate) mod authentication;
pub(crate) mod cluster_router;
pub(crate) mod state;

/// This is the only entry for users to get the rucat server.
pub async fn get_server() -> Result<Router> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;

    let app_state = AppState::new(DataStore::connect_embedded_db(db));

    Ok(Router::new()
        .route("/", get(|_: State<AppState<'_>>| async {"welcome to rucat"}))
        .nest("/cluster", get_cluster_router())
        .layer(middleware::from_fn(auth))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state))
}
