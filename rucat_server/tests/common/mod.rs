use ::mockall::mock;
use ::rucat_common::{
    config::Credentials,
    database::{DatabaseClient, UpdateEngineStateResponse},
    engine::{EngineId, EngineInfo, EngineState},
    error::*,
};
use ::rucat_server::get_server_internal;
use axum::async_trait;
use axum_test::TestServer;

mock! {
    pub DBClient{}
    #[async_trait]
    impl DatabaseClient for DBClient {
        fn get_uri(&self) -> &str;
        fn get_credentials<'a>(&'a self) -> Option<&'a Credentials>;
        async fn connect_local_db<'a>(credentials: Option<&'a Credentials>, uri: String) -> Result<Self>;
        async fn add_engine(&self, engine: EngineInfo) -> Result<EngineId>;
        async fn delete_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>>;
        async fn update_engine_state(
            &self,
            id: &EngineId,
            before: Vec<EngineState>,
            after: EngineState,
        ) -> Result<Option<UpdateEngineStateResponse>>;
        async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>>;
        async fn list_engines(&self) -> Result<Vec<EngineId>>;
    }
}

pub async fn get_test_server(auth_enable: bool, db: MockDBClient) -> Result<TestServer> {
    let app = get_server_internal::<MockDBClient>(auth_enable, db)?;
    TestServer::new(app).map_err(RucatError::fail_to_start_server)
}
