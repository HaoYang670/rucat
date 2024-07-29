use std::process::Child;

use authentication::auth;
use axum::{extract::State, middleware, routing::get, Router};
use axum_extra::middleware::option_layer;
use engine::router::get_engine_router;
use rucat_common::config::{read_config, DataBaseType};
use rucat_common::database::DataBase;
use rucat_common::{config::ServerConfig, error::Result};
use state::AppState;
use tower_http::trace::TraceLayer;

pub(crate) mod authentication;
pub(crate) mod engine;
pub(crate) mod state;

/// This is the only entry for users to get the rucat server.
/// # Return
/// - The router for the server
/// - The process of the embedded database if the database is embedded
pub async fn get_server(config_path: &str) -> Result<(Router, Option<Child>)> {
    let ServerConfig {
        auth_enable,
        engine_binary_path,
        db_type,
    } = read_config(config_path)?;

    let (db, embedded_db_ps) = match db_type {
        DataBaseType::Embedded => {
            let (db, ps) = DataBase::create_embedded_db().await?;
            (db, Some(ps))
        }
        DataBaseType::Local(path) => (DataBase::connect_local_db(path).await?, None),
    };
    let app_state = AppState::new(db, engine_binary_path);

    // go through the router from outer to inner
    let router = Router::new()
        .route("/", get(|_: State<AppState>| async { "welcome to rucat" }))
        .nest("/engine", get_engine_router())
        // TODO: use tower::ServiceBuilder to build the middleware stack
        // but need to be careful with the order of the middleware and the compatibility with axum::option_layer
        .layer(option_layer(auth_enable.then(|| middleware::from_fn(auth))))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    Ok((router, embedded_db_ps))
}
