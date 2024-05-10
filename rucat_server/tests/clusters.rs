use axum_test::TestServer;
use rucat_common::error::Result;
use rucat_server::get_server;

/// server with embedded datastore and authentication disabled
async fn get_test_server() -> Result<TestServer> {
    let app = get_server(false).await?;
    TestServer::new(app).map_err(|e| e.into())
}

#[tokio::test]
async fn undefined_handler() -> Result<()> {
    let server = get_test_server().await?;

    let response = server.get("/any").await;

    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn root_get_request() -> Result<()> {
    let server = get_test_server().await?;

    let response = server.get("/").await;

    response.assert_status_ok();
    response.assert_text("welcome to rucat");
    Ok(())
}
