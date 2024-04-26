use axum::{middleware, Router};
use rucat_common::error::Result;
use rucat_server::{
    authentication::auth,
    cluster_router::get_cluster_router,
    state::{data_store::DataStore, AppState},
};
use surrealdb::{engine::local::Mem, Surreal};
use tower_http::trace::TraceLayer;

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    static ENDPOINT: &str = "127.0.0.1:3000";
    // setup tracing
    tracing_subscriber::fmt::init();

    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;

    let app_state = AppState::new(DataStore::connect_embedded_db(db));

    let app = Router::new()
        .nest("/cluster", get_cluster_router())
        .layer(middleware::from_fn(auth))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // run it
    let listener = tokio::net::TcpListener::bind(ENDPOINT).await?;
    println!("Rucat server is listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
