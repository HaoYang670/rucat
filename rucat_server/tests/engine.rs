use axum_test::{TestResponse, TestServer};
use http::StatusCode;
use rucat_common::error::Result;
use rucat_server::get_server;
use serde_json::json;

/// server with embedded datastore and authentication disabled
async fn get_test_server() -> Result<TestServer> {
    let (app, _) = get_server("./tests/configs/engine_test_config.json").await?;
    TestServer::new(app).map_err(|e| e.into())
}

/// This is a helper function to create an engine.
///
/// **DO NOT** use this function when testing failed cases in create_engine
async fn create_engine_helper(server: &TestServer) -> TestResponse {
    server
        .post("/engine")
        .json(&json!({
            "name": "test",
            "engine_type": "BallistaLocal"
        }))
        .await
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
            "name": "test"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.text().contains("missing field `engine_type`"));
    Ok(())
}

#[tokio::test]
async fn create_engine_with_invalid_engine_type() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .post("/engine")
        .json(&json!({
            "name": "test",
            "engine_type": "Invalid"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response
        .text()
        .contains("engine_type: unknown variant `Invalid`"));
    Ok(())
}

#[tokio::test]
async fn create_engine_with_unknown_field() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .post("/engine")
        .json(&json!({
            "name": "test",
            "engine_type": "BallistaLocal",
            "invalid": "invalid"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response
        .text()
        .contains("invalid: unknown field `invalid`, expected `name` or `engine_type`"));
    Ok(())
}

#[tokio::test]
async fn get_engine() -> Result<()> {
    let server = get_test_server().await?;
    let engine_id = create_engine_helper(&server).await.text();

    let response = server.get(&format!("/engine/{}", engine_id)).await;
    response.assert_status_ok();
    response.assert_text_contains(
        r#"{"name":"test","engine_type":"BallistaLocal","endpoint":null,"state":"Pending","created_time":"#,
    );

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
    let engine_id = create_engine_helper(&server).await.text();

    let response = server.delete(&format!("/engine/{}", engine_id)).await;
    response.assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn stop_engine() -> Result<()> {
    let server = get_test_server().await?;
    let engine_id = create_engine_helper(&server).await.text();

    let response = server.post(&format!("/engine/{}/stop", engine_id)).await;
    response.assert_status_ok();

    let response = server.get(&format!("/engine/{}", engine_id)).await;
    response.assert_text_contains(
        r#"{"name":"test","engine_type":"BallistaLocal","endpoint":null,"state":"Stopped","created_time":"#,
    );

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
    let engine_id = create_engine_helper(&server).await.text();

    server.post(&format!("/engine/{}/stop", engine_id)).await;

    let response = server.post(&format!("/engine/{}/stop", engine_id)).await;
    response.assert_status_forbidden();
    response.assert_text(format!(
        "Not allowed error: Engine {} is in Stopped state, cannot be stopped",
        engine_id
    ));

    Ok(())
}

#[tokio::test]
async fn restart_engine() -> Result<()> {
    let server = get_test_server().await?;
    let engine_id = create_engine_helper(&server).await.text();

    server.post(&format!("/engine/{}/stop", engine_id)).await;
    let response = server.post(&format!("/engine/{}/restart", engine_id)).await;
    response.assert_status_ok();

    let response = server.get(&format!("/engine/{}", engine_id)).await;
    response.assert_text_contains(
        r#"{"name":"test","engine_type":"BallistaLocal","endpoint":null,"state":"Pending","created_time":"#,
    );

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
    let engine_id = create_engine_helper(&server).await.text();

    let response = server.post(&format!("/engine/{}/restart", engine_id)).await;
    response.assert_status_forbidden();
    response.assert_text(format!(
        "Not allowed error: Engine {} is in Pending state, cannot be restarted",
        engine_id
    ));

    Ok(())
}

#[tokio::test]
#[should_panic(expected = "not yet implemented")]
async fn cannot_restart_running_engine() {
    todo!("not yet implemented")
}

#[tokio::test]
async fn list_engines_empty() -> Result<()> {
    let server = get_test_server().await?;
    let response = server.get("/engine").await;
    response.assert_status_ok();
    response.assert_text("[]");
    Ok(())
}

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
    response.assert_text(format!("[\"{}\",\"{}\"]", ids[0], ids[1]));

    Ok(())
}
