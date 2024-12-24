mod common;

use ::std::{borrow::Cow, collections::BTreeMap};

use ::mockall::predicate;
use ::rucat_common::{
    database::UpdateEngineStateResponse,
    engine::{CreateEngineRequest, EngineId, EngineInfo, EngineState::*, EngineTime, EngineType},
    error::*,
    serde_json::json,
    tokio,
};
use common::{get_test_server, MockDB};
use http::StatusCode;

#[tokio::test]
async fn undefined_handler() -> Result<()> {
    let db = MockDB::new();
    let server = get_test_server(false, db).await?;

    let response = server.get("/any").await;

    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn root_get_request() -> Result<()> {
    let db = MockDB::new();
    let server = get_test_server(false, db).await?;

    let response = server.get("/").await;

    response.assert_status_ok();
    response.assert_text("welcome to rucat");
    Ok(())
}

#[tokio::test]
async fn get_engine_not_found() -> Result<()> {
    let mut db = MockDB::new();
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
    let db = MockDB::new();
    let server = get_test_server(false, db).await?;

    let response = server
        .post("/engine")
        .json(&json!({
            "version": "3.5.3",
            "engine_type": "Spark",
            "config": {}
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.text().contains("missing field `name`"));
    Ok(())
}

#[tokio::test]
async fn create_engine_with_unknown_field() -> Result<()> {
    let db = MockDB::new();
    let server = get_test_server(false, db).await?;

    let response = server
        .post("/engine")
        .json(&json!({
            "name": "test",
            "invalid": "invalid"
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.text().contains(
        "invalid: unknown field `invalid`, expected one of `name`, `engine_type`, `version`, `config`"
    ));
    Ok(())
}

#[tokio::test]
async fn create_engine_with_unsupported_engine_type() -> Result<()> {
    let db = MockDB::new();
    let server = get_test_server(false, db).await?;

    let response = server
        .post("/engine")
        .json(&json!({
            "name": "test",
            "engine_type": "foo",
            "config": {}
        }))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response
        .text()
        .contains("engine_type: unknown variant `foo`"));
    Ok(())
}

#[tokio::test]
async fn create_engine() -> Result<()> {
    let mut db = MockDB::new();
    db.expect_add_engine()
        .with(predicate::eq(CreateEngineRequest {
            name: "test".to_owned(),
            engine_type: EngineType::Spark,
            version: "3.5.3".to_owned(),
            config: Some(BTreeMap::from([(
                Cow::Borrowed("spark.executor.instances"),
                Cow::Borrowed("1"),
            )])),
        }))
        .times(1)
        .returning(|_| Ok(EngineId::new(Cow::Borrowed("123"))?));
    let server = get_test_server(false, db).await?;

    let response = server
        .post("/engine")
        .json(&json!({
            "name": "test",
            "engine_type": "Spark",
            "version": "3.5.3",
            "config": {
                "spark.executor.instances": "1"
            }
        }))
        .await;

    response.assert_json(&json!({
        "id": "123"
    }));

    Ok(())
}

#[tokio::test]
async fn get_engine() -> Result<()> {
    let mut db = MockDB::new();
    let engine_info = EngineInfo::new(
        "engine1".to_owned(),
        EngineType::Spark,
        "3.5.3".to_owned(),
        Running,
        BTreeMap::new(),
        EngineTime::now(),
    );
    let engine_info_cloned = engine_info.clone();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(move |_| Ok(Some(engine_info.clone())));
    let server = get_test_server(false, db).await?;

    let response: EngineInfo = server.get("/engine/123").await.json();
    assert_eq!(response, engine_info_cloned);

    Ok(())
}

#[tokio::test]
async fn delete_nonexistent_engine() -> Result<()> {
    let mut db = MockDB::new();
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
async fn delete_engine() -> Result<()> {
    let mut db = MockDB::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| {
            Ok(Some(EngineInfo::new(
                "engine1".to_owned(),
                EngineType::Spark,
                "3.5.3".to_owned(),
                WaitToStart,
                BTreeMap::new(),
                EngineTime::now(),
            )))
        });
    db.expect_delete_engine()
        .with(
            predicate::eq(EngineId::new(Cow::Borrowed("123"))?),
            predicate::eq(&WaitToStart),
        )
        .times(1)
        .returning(|_, _| {
            Ok(Some(UpdateEngineStateResponse {
                before_state: WaitToStart,
                update_success: true,
            }))
        });
    let server = get_test_server(false, db).await?;

    let response = server.delete("/engine/123").await;
    response.assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn stop_wait_to_start_engine() -> Result<()> {
    let mut db = MockDB::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| {
            Ok(Some(EngineInfo::new(
                "engine1".to_owned(),
                EngineType::Spark,
                "3.5.3".to_owned(),
                WaitToStart,
                BTreeMap::new(),
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
    let mut db = MockDB::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| Ok(None));
    let server = get_test_server(false, db).await?;
    let response = server.post("/engine/123/stop").await;
    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn restart_terminated_engine() -> Result<()> {
    let mut db = MockDB::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| {
            Ok(Some(EngineInfo::new(
                "engine1".to_owned(),
                EngineType::Spark,
                "3.5.3".to_owned(),
                Terminated,
                BTreeMap::new(),
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
    let mut db = MockDB::new();
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
    let mut db = MockDB::new();
    db.expect_get_engine()
        .with(predicate::eq(EngineId::new(Cow::Borrowed("123"))?))
        .times(1)
        .returning(|_| {
            Ok(Some(EngineInfo::new(
                "engine1".to_owned(),
                EngineType::Spark,
                "3.5.3".to_owned(),
                WaitToStart,
                BTreeMap::new(),
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
    let mut db = MockDB::new();
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
    let mut db = MockDB::new();
    db.expect_list_engines()
        .times(1)
        .returning(move || Ok(ids_cloned.to_vec()));
    let server = get_test_server(false, db).await?;

    let response = server.get("/engine").await;
    response.assert_status_ok();
    response.assert_json(&json!([ids[0], ids[1]]));

    Ok(())
}
