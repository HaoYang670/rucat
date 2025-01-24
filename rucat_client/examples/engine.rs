use ::rucat_client::{engine::create_engine, Credentials};
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
    let id = create_engine(&request, Some(&credentials)).await?;
    println!("Engine created with id: {}", id);
    Ok(())
}
