mod common;

use ::std::{borrow::Cow, collections::HashMap};

use ::mockall::predicate;
use ::rucat_common::{
    database_client::UpdateEngineStateResponse,
    engine::{EngineConfigs, EngineId, EngineInfo, EngineState::*, EngineTime},
    error::*,
    serde_json::json,
    tokio,
};
use common::{get_test_server, MockDBClient};
use http::StatusCode;

/// This is a helper function to start an engine.
///
/// **DO NOT** use this function when testing failed cases in start_engine
/*
async fn start_engine_helper(server: &TestServer) -> TestResponse {
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
    let db = MockDBClient::new();
    let server = get_test_server(false, db).await?;

    let response = server.get("/any").await;

    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn root_get_request() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(false, db).await?;

    let response = server.get("/").await;

    response.assert_status_ok();
    response.assert_text("welcome to rucat");
    Ok(())
}

#[tokio::test]
async fn get_engine_not_found() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("any"))?))
        .times(1)
        .returning(|_| Ok(None));
    let server = get_test_server(false, db).await?;

    let response = server.get("/engine/any").await;

    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn start_engine_with_missing_field() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(false, db).await?;

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
async fn start_engine_with_unknown_field() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(false, db).await?;

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
async fn get_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| {
            Ok(Some(EngineInfo::new(
                "engine1".to_owned(),
                Running,
                EngineConfigs::try_from(HashMap::new())?,
                EngineTime::now(),
            )))
        });
    let server = get_test_server(false, db).await?;

    let response: EngineInfo = server.get("/engine/123").await.json();
    assert_eq!(response.name, "engine1");
    assert_eq!(response.state, Running);

    Ok(())
}

#[tokio::test]
async fn delete_nonexistent_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| Ok(None));
    let server = get_test_server(false, db).await?;
    let response = server.delete("/engine/123").await;
    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn stop_wait_to_start_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| {
            Ok(Some(EngineInfo::new(
                "engine1".to_owned(),
                WaitToStart,
                EngineConfigs::try_from(HashMap::new())?,
                EngineTime::now(),
            )))
        });
    db.expect_update_engine_state()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("123"))?),
            predicate::eq(&WaitToStart),
            predicate::eq(&Terminated),
        )
        .times(1)
        .returning(|_, _, _| {
            Ok(Some(UpdateEngineStateResponse {
                before_state: WaitToStart,
                update_success: true,
            }))
        });
    let server = get_test_server(false, db).await?;

    let response = server.post("/engine/123/stop").await;
    response.assert_status_ok();

    Ok(())
}

#[tokio::test]
async fn stop_nonexistent_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| Ok(None));
    let server = get_test_server(false, db).await?;
    let response = server.post("/engine/123/stop").await;
    response.assert_status_not_found();
    Ok(())
}

/*/
#[tokio::test]
async fn delete_engine() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = start_engine_helper(&server).await.json();

    let response = server.delete(&format!("/engine/{}", id)).await;
    response.assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn stop_engine_twice() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = start_engine_helper(&server).await.json();

    server.post(&format!("/engine/{}/stop", id)).await;

    let response = server.post(&format!("/engine/{}/stop", id)).await;
    response.assert_status_forbidden();
    response.assert_text(format!(
        "Not allowed error: Engine {} is in Stopped state, cannot be stopped",
        id
    ));

    Ok(())
}

*/

#[tokio::test]
async fn restart_terminated_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| {
            Ok(Some(EngineInfo::new(
                "engine1".to_owned(),
                Terminated,
                EngineConfigs::try_from(HashMap::new())?,
                EngineTime::now(),
            )))
        });
    db.expect_update_engine_state()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("123"))?),
            predicate::eq(&Terminated),
            predicate::eq(&WaitToStart),
        )
        .times(1)
        .returning(|_, _, _| {
            Ok(Some(UpdateEngineStateResponse {
                before_state: Terminated,
                update_success: true,
            }))
        });
    let server = get_test_server(false, db).await?;

    let response = server.post("/engine/123/restart").await;
    response.assert_status_ok();

    Ok(())
}

#[tokio::test]
async fn restart_nonexistent_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| Ok(None));
    let server = get_test_server(false, db).await?;
    let response = server.post("/engine/123/restart").await;
    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn cannot_restart_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| {
            Ok(Some(EngineInfo::new(
                "engine1".to_owned(),
                WaitToStart,
                EngineConfigs::try_from(HashMap::new())?,
                EngineTime::now(),
            )))
        });
    let server = get_test_server(false, db).await?;

    let response = server.post("/engine/123/restart").await;
    response.assert_status_forbidden();
    response
        .text()
        .contains("Not allowed: Engine 123 is in WaitToStart state, cannot be restarted");

    Ok(())
}

#[tokio::test]
async fn list_engines_empty() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_list_engines().times(1).returning(|| Ok(vec![]));
    let server = get_test_server(false, db).await?;
    let response = server.get("/engine").await;
    response.assert_status_ok();
    response.assert_text("[]");
    Ok(())
}

#[tokio::test]
async fn list_2_engines() -> Result<()> {
    let ids = [
        EngineId::new(Cow::Borrowed("1"))?,
        EngineId::new(Cow::Borrowed("2"))?,
    ];
    let ids_cloned = ids.clone();
    let mut db = MockDBClient::new();
    db.expect_list_engines()
        .times(1)
        .returning(move || Ok(ids_cloned.to_vec()));
    let server = get_test_server(false, db).await?;

    let response = server.get("/engine").await;
    response.assert_status_ok();
    response.assert_json(&json!([ids[0], ids[1]]));

    Ok(())
}
