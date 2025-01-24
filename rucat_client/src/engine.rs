//! Functions for managing the engine.

use ::rucat_common::engine::{CreateEngineRequest, EngineId};

use crate::Credentials;

pub async fn create_engine(
    request: &CreateEngineRequest,
    credentials: Option<&Credentials<'_>>,
) -> Result<EngineId, reqwest::Error> {
    let client = reqwest::Client::new();
    let builder = client.post("http://localhost:3000/engine").json(request);
    let builder = match credentials {
        Some(Credentials::Basic { username, password }) => {
            builder.basic_auth(username, Some(password))
        }
        Some(Credentials::Bearer { token }) => builder.bearer_auth(token),
        None => builder,
    };
    builder.send().await?.error_for_status()?.json().await
}
