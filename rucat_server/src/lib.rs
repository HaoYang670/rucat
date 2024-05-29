use authentication::auth;
use axum::{extract::State, middleware, routing::get, Router};
use axum_extra::middleware::option_layer;
use engine_router::get_engine_router;
use rucat_common::error::Result;
use state::{data_store::DataStore, AppState};
use surrealdb::{engine::local::Mem, Surreal};
use tower_http::trace::TraceLayer;

pub(crate) mod authentication;
pub(crate) mod engine_router;
pub(crate) mod state;

/// This is the only entry for users to get the rucat server.
pub async fn get_server(auth_enable: bool) -> Result<Router> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;

    let app_state = AppState::new(DataStore::connect_embedded_db(db));

    // go through the router from outer to inner
    Ok(Router::new()
        .route(
            "/",
            get(|_: State<AppState<'_>>| async { "welcome to rucat" }),
        )
        .nest("/engine", get_engine_router())
        // TODO: use tower::ServiceBuilder to build the middleware stack
        // but need to be careful with the order of the middleware and the compatibility with axum::option_layer
        .layer(option_layer(auth_enable.then(|| middleware::from_fn(auth))))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state))
}
