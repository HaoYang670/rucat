use axum_extra::headers::authorization::Credentials;
use axum_test::TestServer;
use headers::Authorization;
use http::{header::AUTHORIZATION, HeaderValue};
use rucat_common::error::Result;
use rucat_server::get_server;

/// server with embedded datastore and authentication enabled
async fn get_test_server() -> Result<TestServer> {
    let app = get_server(true).await?;
    TestServer::new(app).map_err(|e| e.into())
}

static USERNAME: &str = "remzi";
static PWD: &str = "yang";
static TOKEN: &str = "Bearer remziy"; // Bearer token

#[tokio::test]
async fn without_auth_header() -> Result<()> {
    let server = get_test_server().await?;

    let response = server.get("/any").await;

    response.assert_status_unauthorized();
    Ok(())
}

#[tokio::test]
async fn with_wrong_basic_auth() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .get("/")
        .add_header(
            AUTHORIZATION,
            Authorization::basic("wrong", "wrong").0.encode(),
        )
        .await;

    response.assert_status_unauthorized();
    response.assert_text("Unauthorized error: wrong credentials");
    Ok(())
}

#[tokio::test]
async fn with_wrong_bearer_auth() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .get("/any")
        .add_header(
            AUTHORIZATION,
            Authorization::bearer("wrong").unwrap().0.encode(),
        )
        .await;

    response.assert_status_unauthorized();
    response.assert_text("Unauthorized error: wrong credentials");
    Ok(())
}

#[tokio::test]
async fn with_unsupported_auth() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
      .get("/")
      // AWS signature
      .add_header(
        AUTHORIZATION,
        HeaderValue::from_static(
          "AWS4-HMAC-SHA256 Credential=access/20240509/us-west-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-security-token, Signature=f5ac1720dc52cb85d150acfd743202b99ac316470427f29b58e41b237c756929"
        ))
      .await;

    response.assert_status_unauthorized();
    response.assert_text("Unauthorized error: Unsupported credentials type");
    Ok(())
}

#[tokio::test]
async fn undefined_handler() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .get("/any")
        .add_header(
            AUTHORIZATION,
            Authorization::basic(USERNAME, PWD).0.encode(),
        )
        .await;

    response.assert_status_not_found();
    Ok(())
}

#[tokio::test]
async fn root_get_request() -> Result<()> {
    let server = get_test_server().await?;

    let response = server
        .get("/")
        .add_header(
            AUTHORIZATION,
            Authorization::basic(USERNAME, PWD).0.encode(),
        )
        .await;

    response.assert_status_ok();
    response.assert_text("welcome to rucat");
    Ok(())
}