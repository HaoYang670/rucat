use ::httpmock::prelude::*;
use ::reqwest::StatusCode;
use ::rucat_client::{resource_client::ResourceClient, Credentials};
use ::rucat_common::{
    engine::{CreateEngineRequest, EngineType},
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
