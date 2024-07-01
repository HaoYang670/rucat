use authentication::auth;
use axum::{extract::State, middleware, routing::get, Router};
use axum_extra::middleware::option_layer;
use config::{Config, DataBaseType};
use engine::router::get_engine_router;
use rucat_common::error::Result;
use state::{database::DataBase, AppState};
use tower_http::trace::TraceLayer;

pub(crate) mod authentication;
pub(crate) mod engine;
pub(crate) mod state;
mod config;

/// This is the only entry for users to get the rucat server.
pub async fn get_server(config_path: &str) -> Result<Router> {
    let Config {
        auth_enable,
        engine_binary_path,
        database: db_type,
    } = Config::read_config(config_path)?;

    let db = match db_type {
        DataBaseType::Embedded => DataBase::create_embedded_db().await?,
        DataBaseType::Local(path) => DataBase::connect_local_db(path).await?,
    };
    let app_state = AppState::new(db, engine_binary_path);

    // go through the router from outer to inner
    Ok(Router::new()
        .route("/", get(|_: State<AppState>| async { "welcome to rucat" }))
        .nest("/engine", get_engine_router())
        // TODO: use tower::ServiceBuilder to build the middleware stack
        // but need to be careful with the order of the middleware and the compatibility with axum::option_layer
        .layer(option_layer(auth_enable.then(|| middleware::from_fn(auth))))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state))
}
