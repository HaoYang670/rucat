use ::rucat_client::{resource_client, Credentials};
use ::rucat_common::engine::{CreateEngineRequest, EngineType};
use rucat_common::tokio;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let request = CreateEngineRequest {
        name: "spark0".to_owned(),
        engine_type: EngineType::Spark,
        version: "3.0.0".to_owned(),
        config: None,
    };
    let credentials = Credentials::Bearer { token: "admin" };
    let client = resource_client::ResourceClient::new("http://localhost:3000", Some(credentials));
    let id = client.create_engine(&request).await?;
    println!("Engine created with id: {}", id);
    Ok(())
}
