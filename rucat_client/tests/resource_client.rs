use ::httpmock::prelude::*;
use ::reqwest::StatusCode;
use ::rucat_client::{resource_client::ResourceClient, Credentials};
use ::rucat_common::{
    engine::{CreateEngineRequest, EngineInfo, EngineState, EngineTime, EngineType},
    serde_json::json,
    tokio,
};
use ::std::{borrow::Cow, collections::BTreeMap};

#[tokio::test]
async fn create_engine_success() {
    let server = MockServer::start();
    let request_body = CreateEngineRequest {
        name: "engine1".to_owned(),
        engine_type: EngineType::Spark,
        version: "3.5.4".to_owned(),
        config: Some(BTreeMap::from([(
            Cow::Borrowed("spark.executor.memory"),
            Cow::Borrowed("2g"),
        )])),
    };
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/engine")
            .header_exists("Authorization")
            .json_body_obj(&request_body);
        then.status(200)
            .header("content-type", "text/html; charset=UTF-8")
            .json_body(json!({
                "id": "abc",
            }));
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let engine_id = client.create_engine(&request_body).await.unwrap();

    mock.assert();
    assert_eq!(engine_id.to_string(), "abc");
}

#[tokio::test]
async fn create_engine_error() {
    let server = MockServer::start();
    let request_body = CreateEngineRequest {
        name: "engine1".to_owned(),
        engine_type: EngineType::Spark,
        version: "3.5.4".to_owned(),
        config: Some(BTreeMap::from([(
            Cow::Borrowed("spark.executor.memory"),
            Cow::Borrowed("2g"),
        )])),
    };
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/engine")
            .header_exists("Authorization")
            .json_body_obj(&request_body);
        then.status(500);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let err = client.create_engine(&request_body).await.unwrap_err();

    mock.assert();
    assert_eq!(err.status(), Some(StatusCode::INTERNAL_SERVER_ERROR));
}

#[tokio::test]
async fn get_engine_info_error() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/engine/abc")
            .header_exists("Authorization");
        then.status(500);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let err = client
        .get_engine_info(&"abc".try_into().unwrap())
        .await
        .unwrap_err();

    mock.assert();
    assert_eq!(err.status(), Some(StatusCode::INTERNAL_SERVER_ERROR));
}

#[tokio::test]
async fn get_engine_info_success() {
    let server = MockServer::start();
    let engine_info = EngineInfo::new(
        "engine1".to_owned(),
        EngineType::Spark,
        "3.5.4".to_owned(),
        EngineState::Running,
        BTreeMap::from([(Cow::Borrowed("spark.executor.memory"), Cow::Borrowed("2g"))]),
        EngineTime::now(),
    );
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/engine/abc")
            .header_exists("Authorization");
        then.status(200)
            .header("content-type", "text/html; charset=UTF-8")
            .json_body_obj(&engine_info);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let response = client
        .get_engine_info(&"abc".try_into().unwrap())
        .await
        .unwrap();

    mock.assert();
    assert_eq!(response, engine_info);
}

#[tokio::test]
async fn list_engines_error() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/engine")
            .header_exists("Authorization");
        then.status(500);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let err = client.list_engines().await.unwrap_err();

    mock.assert();
    assert_eq!(err.status(), Some(StatusCode::INTERNAL_SERVER_ERROR));
}

#[tokio::test]
async fn list_engines_success() {
    let server = MockServer::start();
    let engine_ids = vec!["abc".try_into().unwrap(), "def".try_into().unwrap()];
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/engine")
            .header_exists("Authorization");
        then.status(200)
            .header("content-type", "text/html; charset=UTF-8")
            .json_body_obj(&engine_ids);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let response = client.list_engines().await.unwrap();

    mock.assert();
    assert_eq!(response, engine_ids);
}

#[tokio::test]
async fn stop_engine_error() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/engine/abc/stop")
            .header_exists("Authorization");
        then.status(500);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let err = client
        .stop_engine(&"abc".try_into().unwrap())
        .await
        .unwrap_err();

    mock.assert();
    assert_eq!(err.status(), Some(StatusCode::INTERNAL_SERVER_ERROR));
}

#[tokio::test]
async fn stop_engine_success() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/engine/abc/stop")
            .header_exists("Authorization");
        then.status(200);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    client
        .stop_engine(&"abc".try_into().unwrap())
        .await
        .unwrap();

    mock.assert();
}

#[tokio::test]
async fn restart_engine_error() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/engine/abc/restart")
            .header_exists("Authorization");
        then.status(500);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let err = client
        .restart_engine(&"abc".try_into().unwrap())
        .await
        .unwrap_err();

    mock.assert();
    assert_eq!(err.status(), Some(StatusCode::INTERNAL_SERVER_ERROR));
}

#[tokio::test]
async fn restart_engine_success() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/engine/abc/restart")
            .header_exists("Authorization");
        then.status(200);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    client
        .restart_engine(&"abc".try_into().unwrap())
        .await
        .unwrap();

    mock.assert();
}

#[tokio::test]
async fn delete_engine_error() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(DELETE)
            .path("/engine/abc")
            .header_exists("Authorization");
        then.status(500);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    let err = client
        .delete_engine(&"abc".try_into().unwrap())
        .await
        .unwrap_err();

    mock.assert();
    assert_eq!(err.status(), Some(StatusCode::INTERNAL_SERVER_ERROR));
}

#[tokio::test]
async fn delete_engine_success() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(DELETE)
            .path("/engine/abc")
            .header_exists("Authorization");
        then.status(200);
    });
    let base_url = server.base_url();
    let client = ResourceClient::new(&base_url, Some(Credentials::Bearer { token: "admin" }));
    client
        .delete_engine(&"abc".try_into().unwrap())
        .await
        .unwrap();

    mock.assert();
}
