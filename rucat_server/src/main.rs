use axum::{middleware, routing::get, Router};
use rucat_common::error::Result;
use rucat_server::{authentication::auth, cluster_router::get_cluster_router};
use tower_http::trace::TraceLayer;

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    static ENDPOINT: &str = "127.0.0.1:3000";
    // setup tracing
    tracing_subscriber::fmt::init();
    let app = get_app();

    // run it
    let listener = tokio::net::TcpListener::bind(ENDPOINT).await?;
    println!("Rucat server is listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}

/// build our application with a route
fn get_app() -> Router {
    async fn root_handler() -> &'static str {
        "Hello, Rucat!"
    }

    Router::new()
        .route("/", get(root_handler))
        .nest("/cluster", get_cluster_router())
        .layer(middleware::from_fn(auth))
        .layer(TraceLayer::new_for_http())
}
