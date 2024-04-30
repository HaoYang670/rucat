use rucat_common::error::Result;
use rucat_server::get_server;

#[tokio::main]
/// Start Rucat server
async fn main() -> Result<()> {
    static ENDPOINT: &str = "127.0.0.1:3000";
    let app = get_server().await?;

    // run it
    let listener = tokio::net::TcpListener::bind(ENDPOINT).await?;
    println!("Rucat server is listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
