//! Functions for managing the engine.

use ::rucat_common::engine::{CreateEngineRequest, EngineId};

pub async fn create_engine(request: &CreateEngineRequest) -> Result<EngineId, reqwest::Error> {
    let client = reqwest::Client::new();
    let id: EngineId = client
        .post("http://localhost:3000/engine")
        .json(request)
        .send()
        .await?
        .json()
        .await?;

    Ok(id)
}
