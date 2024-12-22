mod common;

use ::rucat_common::{error::Result, tokio};
use axum_extra::headers::authorization::Credentials as _;
use common::{get_test_server, MockDBClient};
use headers::Authorization;
use http::{header::AUTHORIZATION, HeaderValue};

static USERNAME: &str = "admin";
static PWD: &str = "admin";
static TOKEN: &str = "admin"; // Bearer token

#[tokio::test]
async fn without_auth_header() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(true, db).await?;

    let response = server.get("/any").await;

    response.assert_status_unauthorized();
    Ok(())
}

#[tokio::test]
async fn with_wrong_basic_auth() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(true, db).await?;

    let response = server
        .get("/")
        .add_header(
            AUTHORIZATION,
            Authorization::basic("wrong", "wrong").0.encode(),
        )
        .await;

    response.assert_status_unauthorized();
    response.assert_text("Unauthorized: wrong credentials");
    Ok(())
}

#[tokio::test]
async fn with_wrong_bearer_auth() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(true, db).await?;

    let response = server
        .get("/any")
        .add_header(
            AUTHORIZATION,
            Authorization::bearer("wrong").unwrap().0.encode(),
        )
        .await;

    response.assert_status_unauthorized();
    response.assert_text("Unauthorized: wrong credentials");
    Ok(())
}

#[tokio::test]
async fn with_unsupported_auth() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(true, db).await?;

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
    response.assert_text("Unauthorized: Unsupported credentials type");
    Ok(())
}

#[tokio::test]
async fn basic_auth_successful() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(true, db).await?;

    let response = server
        .get("/")
        .add_header(
            AUTHORIZATION,
            Authorization::basic(USERNAME, PWD).0.encode(),
        )
        .await;

    response.assert_status_ok();
    Ok(())
}

#[tokio::test]
async fn bearer_auth_successful() -> Result<()> {
    let db = MockDBClient::new();
    let server = get_test_server(true, db).await?;

    let response = server
        .get("/")
        .add_header(
            AUTHORIZATION,
            Authorization::bearer(TOKEN).unwrap().0.encode(),
        )
        .await;

    response.assert_status_ok();
    Ok(())
}
