use axum_test::TestServer;
use http::StatusCode;
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
async fn create_cluster_with_missing_field() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .post("/cluster")
        .json(&json!({
            "name": "test"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    response.assert_text("Failed to deserialize the JSON body into the target type: missing field `cluster_type` at line 1 column 15");
    Ok(())
}

#[tokio::test]
async fn create_cluster_with_invalid_cluster_type() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .post("/cluster")
        .json(&json!({
            "name": "test",
            "cluster_type": "Invalid"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    response.assert_text("Failed to deserialize the JSON body into the target type: cluster_type: unknown variant `Invalid`, expected one of `BallistaLocal`, `BallistaRemote`, `Rucat` at line 1 column 39");
    Ok(())
}

#[tokio::test]
async fn create_cluster_with_unknown_field() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .post("/cluster")
        .json(&json!({
            "name": "test",
            "cluster_type": "BallistaLocal",
            "invalid": "invalid"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    response.assert_text("Failed to deserialize the JSON body into the target type: invalid: unknown field `invalid`, expected `name` or `cluster_type` at line 1 column 55");
    Ok(())
}

#[tokio::test]
async fn create_and_get_cluster() -> Result<()> {
    let server = get_test_server().await?;
    let response = server
        .post("/cluster")
        .json(&json!({
            "name": "test",
            "cluster_type": "BallistaLocal"
        }))
        .await;

    response.assert_status_ok();

    let cluster_id = response.text();
    let response = server.get(&format!("/cluster/{}", cluster_id)).await;
    response.assert_status_ok();
    response.assert_text(
        r#"ClusterInfo { name: "test", cluster_type: BallistaLocal, state: Pending }"#,
    );

    Ok(())
}
