use ::rucat_common::engine::{CreateEngineRequest, EngineId, EngineInfo};

use crate::Credentials;

type Result<T> = std::result::Result<T, reqwest::Error>;

/// Client for managing Rucat resources.
pub struct ResourceClient<'a> {
    /// Base URL of the Rucat server.
    base_url: &'a str,
    /// Credentials for authenticating with the Rucat server.
    credentials: Option<Credentials<'a>>,
    /// HTTP client for making requests to the Rucat server.
    client: reqwest::Client,
}

impl<'a> ResourceClient<'a> {
    /// Create a new `ResourceClient`.
    pub fn new(base_url: &'a str, credentials: Option<Credentials<'a>>) -> Self {
        Self {
            base_url,
            credentials,
            client: reqwest::Client::new(),
        }
    }

    pub async fn create_engine(&self, request: &CreateEngineRequest) -> Result<EngineId> {
        let url = self.build_url("/engine");
        let builder = self.client.post(url).json(request);
        let builder = self.enable_auth_for_request(builder);
        builder.send().await?.error_for_status()?.json().await
    }

    pub async fn get_engine_info(&self, engine_id: &EngineId) -> Result<EngineInfo> {
        let url = self.build_url(&format!("/engine/{}", engine_id));
        let builder = self.client.get(url);
        let builder = self.enable_auth_for_request(builder);
        builder.send().await?.error_for_status()?.json().await
    }

    pub async fn list_engines(&self) -> Result<Vec<EngineId>> {
        let url = self.build_url("/engine");
        let builder = self.client.get(url);
        let builder = self.enable_auth_for_request(builder);
        builder.send().await?.error_for_status()?.json().await
    }

    pub async fn stop_engine(&self, engine_id: &EngineId) -> Result<()> {
        let url = self.build_url(&format!("/engine/{}/stop", engine_id));
        let builder = self.client.post(url);
        let builder = self.enable_auth_for_request(builder);
        builder.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn restart_engine(&self, engine_id: &EngineId) -> Result<()> {
        let url = self.build_url(&format!("/engine/{}/restart", engine_id));
        let builder = self.client.post(url);
        let builder = self.enable_auth_for_request(builder);
        builder.send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn delete_engine(&self, engine_id: &EngineId) -> Result<()> {
        let url = self.build_url(&format!("/engine/{}", engine_id));
        let builder = self.client.delete(url);
        let builder = self.enable_auth_for_request(builder);
        builder.send().await?.error_for_status()?;
        Ok(())
    }

    /// Build a full URL from a path.
    fn build_url(&self, path: &str) -> String {
        self.base_url.to_owned() + path
    }

    /// Enable authentication for a request builder.
    fn enable_auth_for_request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.credentials {
            Some(Credentials::Basic { username, password }) => {
                builder.basic_auth(username, password)
            }
            Some(Credentials::Bearer { token }) => builder.bearer_auth(token),
            None => builder,
        }
    }
}
