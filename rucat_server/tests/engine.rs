use ::rucat_common::error::*;
use axum_test::TestServer;
use http::StatusCode;
use rucat_server::get_server;
use serde_json::json;

/// server with embedded datastore and authentication disabled
async fn get_test_server() -> Result<TestServer> {
    let (app, _) = get_server("./tests/configs/engine_test_config.json").await?;
    TestServer::new(app).map_err(RucatError::fail_to_start_server)
}

/// This is a helper function to create an engine.
///
/// **DO NOT** use this function when testing failed cases in create_engine
/*
async fn create_engine_helper(server: &TestServer) -> TestResponse {
    server
        .post("/engine")
        .json(&json!({
            "name": "test",
            "configs": {},
        }))
        .await
}
*/

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
async fn get_engine_not_found() -> Result<()> {
    let server = get_test_server().await?;

    let response = server.get("/engine/any").await;

    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn create_engine_with_missing_field() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .post("/engine")
        .json(&json!({
            "configs": {}
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.text().contains("missing field `name`"));
    Ok(())
}

#[tokio::test]
async fn create_engine_with_unknown_field() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .post("/engine")
        .json(&json!({
            "name": "test",
            "invalid": "invalid"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response
        .text()
        .contains("invalid: unknown field `invalid`, expected `name` or `configs`"));
    Ok(())
}

#[tokio::test]
async fn create_engine_with_forbidden_configs() -> Result<()> {
    async fn helper(key: &str) -> Result<()> {
        let server = get_test_server().await?;
        let response = server
            .post("/engine")
            .json(&json!({
                "name": "test",
                "configs": {
                    key: "123"
                }
            }))
            .await;

        response.assert_status(StatusCode::FORBIDDEN);
        assert!(response.text().contains(
            format!(
                "Not allowed: The config {} is not allowed as it is reserved.",
                key
            )
            .as_str()
        ));
        Ok(())
    }

    helper("spark.app.id").await?;
    helper("spark.kubernetes.container.image").await?;
    helper("spark.driver.host").await?;
    helper("spark.kubernetes.driver.pod.name").await?;
    helper("spark.kubernetes.executor.podNamePrefix").await?;
    helper("spark.driver.extraJavaOptions").await
}

/*/
#[tokio::test]
async fn get_engine() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = create_engine_helper(&server).await.json();

    let response: EngineInfo = server.get(&format!("/engine/{}", id)).await.json();
    assert_eq!(response.name, "test");
    assert_eq!(response.state, Running);

    Ok(())
}

#[tokio::test]
async fn delete_nonexistent_engine() -> Result<()> {
    let server = get_test_server().await?;
    let response = server.delete("/engine/any").await;
    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn delete_engine() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = create_engine_helper(&server).await.json();

    let response = server.delete(&format!("/engine/{}", id)).await;
    response.assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn stop_engine() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = create_engine_helper(&server).await.json();

    let response = server.post(&format!("/engine/{}/stop", id)).await;
    response.assert_status_ok();

    let response: EngineInfo = server.get(&format!("/engine/{}", id)).await.json();
    assert_eq!(response.name, "test");
    assert_eq!(response.state, Stopped);

    Ok(())
}

#[tokio::test]
async fn stop_nonexistent_engine() -> Result<()> {
    let server = get_test_server().await?;
    let response = server.post("/engine/any/stop").await;
    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn stop_engine_twice() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = create_engine_helper(&server).await.json();

    server.post(&format!("/engine/{}/stop", id)).await;

    let response = server.post(&format!("/engine/{}/stop", id)).await;
    response.assert_status_forbidden();
    response.assert_text(format!(
        "Not allowed error: Engine {} is in Stopped state, cannot be stopped",
        id
    ));

    Ok(())
}

#[tokio::test]
async fn restart_engine() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = create_engine_helper(&server).await.json();

    server.post(&format!("/engine/{}/stop", id)).await;
    let response = server.post(&format!("/engine/{}/restart", id)).await;
    response.assert_status_ok();

    let response: EngineInfo = server.get(&format!("/engine/{}", id)).await.json();
    assert_eq!(response.name, "test");
    // we haven't implemented reconnection yet
    assert_eq!(response.state, Pending);

    Ok(())
}

#[tokio::test]
async fn restart_nonexistent_engine() -> Result<()> {
    let server = get_test_server().await?;
    let response = server.post("/engine/any/restart").await;
    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn cannot_restart_pending_engine() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = create_engine_helper(&server).await.json();
    let response = server.post(&format!("/engine/{}/restart", id)).await;
    response.assert_status_forbidden();
    response.assert_text(format!(
        "Not allowed error: Engine {} is in Pending state, cannot be restarted",
        id
    ));

    Ok(())
}

#[tokio::test]
async fn cannot_restart_running_engine() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = create_engine_helper(&server).await.json();

    let response = server.post(&format!("/engine/{}/restart", id)).await;
    response.assert_status_forbidden();
    response.assert_text(format!(
        "Not allowed error: Engine {} is in Running state, cannot be restarted",
        id
    ));

    Ok(())
}
*/

#[tokio::test]
async fn list_engines_empty() -> Result<()> {
    let server = get_test_server().await?;
    let response = server.get("/engine").await;
    response.assert_status_ok();
    response.assert_text("[]");
    Ok(())
}

/*
#[tokio::test]
async fn list_2_engines() -> Result<()> {
    let server = get_test_server().await?;

    let mut ids = [
        create_engine_helper(&server).await.text(),
        create_engine_helper(&server).await.text(),
    ];
    ids.sort();

    let response = server.get("/engine").await;
    response.assert_status_ok();
    response.assert_text(format!("[{},{}]", ids[0], ids[1]));

    Ok(())
}
*/
