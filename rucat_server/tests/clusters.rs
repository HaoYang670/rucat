use axum_test::TestServer;
use rucat_common::error::Result;
use rucat_server::get_server;


async fn get_test_server() -> Result<TestServer> {
  let app = get_server().await?;
  TestServer::new(app).map_err(|e| e.into())
}

#[tokio::test]
async fn unauthorized() -> Result<()> {
  let server = get_test_server().await?;

  // Get the request.
  let response = server
      .get("/ping")
      .await;

  response.assert_status_unauthorized();
  Ok(())
}
