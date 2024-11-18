mod common;

use ::std::{borrow::Cow, collections::HashMap};

use ::mockall::predicate;
use ::rucat_common::{
    database::UpdateEngineStateResponse,
    engine::{EngineConfigs, EngineId, EngineInfo, EngineState::*},
    error::*,
    serde_json::json,
    tokio,
};
use common::{get_test_server, MockDBClient};
use http::StatusCode;

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
async fn create_engine_with_missing_field() -> Result<()> {
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
async fn create_engine_with_unknown_field() -> Result<()> {
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
async fn create_engine_with_forbidden_configs() -> Result<()> {
    async fn helper(key: &str) -> Result<()> {
        let db = MockDBClient::new();
        let server = get_test_server(false, db).await?;
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
    db.expect_delete_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("any"))?))
        .times(1)
        .returning(|_| Ok(None));
    let server = get_test_server(false, db).await?;
    let response = server.delete("/engine/any").await;
    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn stop_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_update_engine_state()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("123"))?),
            predicate::eq(vec![Pending, Running]),
            predicate::eq(Stopped),
        )
        .times(1)
        .returning(|_, _, _| {
            Ok(Some(UpdateEngineStateResponse {
                before_state: Pending,
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
    db.expect_update_engine_state()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("any"))?),
            predicate::eq(vec![Pending, Running]),
            predicate::eq(Stopped),
        )
        .times(1)
        .returning(|_, _, _| Ok(None));
    let server = get_test_server(false, db).await?;
    let response = server.post("/engine/any/stop").await;
    response.assert_status_not_found();
    Ok(())
}

/*/
#[tokio::test]
async fn delete_engine() -> Result<()> {
    let server = get_test_server().await?;
    let id: EngineId = create_engine_helper(&server).await.json();

    let response = server.delete(&format!("/engine/{}", id)).await;
    response.assert_status_ok();
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

*/

#[tokio::test]
async fn restart_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_update_engine_state()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("123"))?),
            predicate::eq(vec![Stopped]),
            predicate::eq(Pending),
        )
        .times(1)
        .returning(|_, _, _| {
            Ok(Some(UpdateEngineStateResponse {
                before_state: Pending,
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
    db.expect_update_engine_state()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("any"))?),
            predicate::eq(vec![Stopped]),
            predicate::eq(Pending),
        )
        .times(1)
        .returning(|_, _, _| Ok(None));
    let server = get_test_server(false, db).await?;
    let response = server.post("/engine/any/restart").await;
    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn cannot_restart_pending_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_update_engine_state()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("123"))?),
            predicate::eq(vec![Stopped]),
            predicate::eq(Pending),
        )
        .times(1)
        .returning(|_, _, _| {
            Ok(Some(UpdateEngineStateResponse {
                before_state: Pending,
                update_success: false,
            }))
        });
    let server = get_test_server(false, db).await?;

    let response = server.post("/engine/123/restart").await;
    response.assert_status_forbidden();
    response
        .text()
        .contains("Not allowed: Engine 123 is in Pending state, cannot be restarted");

    Ok(())
}

#[tokio::test]
async fn cannot_restart_running_engine() -> Result<()> {
    let mut db = MockDBClient::new();
    db.expect_update_engine_state()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("123"))?),
            predicate::eq(vec![Stopped]),
            predicate::eq(Pending),
        )
        .times(1)
        .returning(|_, _, _| {
            Ok(Some(UpdateEngineStateResponse {
                before_state: Running,
                update_success: false,
            }))
        });
    let server = get_test_server(false, db).await?;

    let response = server.post("/engine/123/restart").await;
    response.assert_status_forbidden();
    response
        .text()
        .contains("Not allowed: Engine 123 is in Running state, cannot be restarted");

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
