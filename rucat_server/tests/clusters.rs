use axum_test::TestServer;
use rucat_common::error::Result;
use rucat_server::get_server;
use serde_json::json;

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

#[tokio::test]
async fn get_cluster_not_found() -> Result<()> {
    let server = get_test_server().await?;

    let response = server.get("/cluster/any").await;

    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn create_and_get_cluster() -> Result<()> {
    let server = get_test_server().await?;
    let response = server
        .post("/cluster")
        //.json(r#"{"name": "test","cluster_type": "Ballista"}"#).await;
        .json(&json!({
            "name": "test",
            "cluster_type": "Ballista"
        }))
        .await;

    //response.assert_status_ok();

    let cluster_id = response.text();
    let response = server.get(&format!("/cluster/{}", cluster_id)).await;
    response.assert_status_ok();
    response.assert_text(r#"ClusterInfo { name: "test", cluster_type: Ballista, state: Pending }"#);

    Ok(())
}
