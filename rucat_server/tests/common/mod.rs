use ::mockall::mock;
use ::rucat_common::{
    database_client::{DatabaseClient, UpdateEngineStateResponse},
    engine::{EngineId, EngineInfo, EngineState, StartEngineRequest},
    error::*,
};
use ::rucat_server::get_server;
use axum::async_trait;
use axum_test::TestServer;

mock! {
    pub DBClient{}
    #[async_trait]
    impl DatabaseClient for DBClient {
        async fn add_engine(&self, engine: StartEngineRequest) -> Result<EngineId>;
        async fn delete_engine(&self, id: &EngineId, current_state: &EngineState) -> Result<Option<UpdateEngineStateResponse>>;
        async fn update_engine_state(
            &self,
            id: &EngineId,
            before: &EngineState,
            after: &EngineState,
        ) -> Result<Option<UpdateEngineStateResponse>>;
        async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>>;
        async fn list_engines(&self) -> Result<Vec<EngineId>>;
        async fn list_engines_need_update(&self) -> Result<Vec<(EngineId, EngineInfo)>>;
    }
}

pub async fn get_test_server(auth_enable: bool, db: MockDBClient) -> Result<TestServer> {
    let app = get_server(auth_enable, db)?;
    TestServer::new(app).map_err(RucatError::fail_to_start_server)
}
