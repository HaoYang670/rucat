use ::std::time::SystemTime;

use ::mockall::mock;
use ::rucat_common::{
    database::{Database, EngineIdAndInfo, UpdateEngineStateResponse},
    engine::{CreateEngineRequest, EngineId, EngineInfo, EngineState},
    error::*,
};
use ::rucat_server::get_server;
use axum_test::TestServer;

mock! {
    pub DB{}
    impl Database for DB {
        async fn add_engine(&self, engine: CreateEngineRequest, next_update_time: Option<SystemTime>) -> Result<EngineId>;
        async fn remove_engine(&self, id: &EngineId, current_state: &EngineState) -> Result<Option<UpdateEngineStateResponse>>;
        async fn update_engine_state(
            &self,
            id: &EngineId,
            before: &EngineState,
            after: &EngineState,
            next_update_time: Option<SystemTime>,
        ) -> Result<Option<UpdateEngineStateResponse>>;
        async fn get_engine(&self, id: &EngineId) -> Result<Option<EngineInfo>>;
        async fn list_engines(&self) -> Result<Vec<EngineId>>;
        async fn list_engines_need_update(&self) -> Result<Vec<EngineIdAndInfo>>;
    }
}

pub async fn get_test_server(auth_enable: bool, db: MockDB) -> Result<TestServer> {
    let app = get_server(auth_enable, db)?;
    TestServer::new(app).map_err(RucatError::fail_to_start_server)
}
