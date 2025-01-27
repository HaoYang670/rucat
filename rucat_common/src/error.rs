use std::fmt::Display;

use ::anyhow::anyhow;

use RucatErrorType::*;

use crate::engine::EngineId;

pub type Result<T> = std::result::Result<T, RucatError>;

#[derive(Debug)]
pub enum RucatErrorType {
    NotFound,
    Unauthorized,
    NotAllowed,
    FailToStartServer,
    FailToStartStateMonitor,
    FailToStartEngine,
    FailToDeleteEngine,
    FailToConnectDatabase,
    FailToUpdateDatabase,
    FailToReadDatabase,
    FailToLoadConfig,
}

impl Display for RucatErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotFound => write!(f, "Not found"),
            Unauthorized => write!(f, "Unauthorized"),
            NotAllowed => write!(f, "Not allowed"),
            FailToStartServer => write!(f, "Fail to start server"),
            FailToStartStateMonitor => write!(f, "Fail to start state monitor"),
            FailToStartEngine => write!(f, "Fail to start engine"),
            FailToDeleteEngine => write!(f, "Fail to delete engine"),
            FailToConnectDatabase => write!(f, "Fail to connect to database"),
            FailToUpdateDatabase => write!(f, "Fail to update database"),
            FailToReadDatabase => write!(f, "Fail to read database"),
            FailToLoadConfig => write!(f, "Fail to load config"),
        }
    }
}

#[derive(Debug)]
pub struct RucatError {
    error_type: RucatErrorType,
    content: anyhow::Error,
}

impl RucatError {
    pub fn get_error_type(&self) -> &RucatErrorType {
        &self.error_type
    }
    pub fn unauthorized<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(Unauthorized, e)
    }

    pub fn not_allowed<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(NotAllowed, e)
    }

    pub fn engine_not_found(id: &EngineId) -> Self {
        Self::not_found(anyhow!("Engine {} not found", id))
    }

    pub fn not_found<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(NotFound, e)
    }

    pub fn fail_to_start_server<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(FailToStartServer, e)
    }

    pub fn fail_to_start_state_monitor<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(FailToStartStateMonitor, e)
    }
    pub fn fail_to_start_engine<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(FailToStartEngine, e)
    }

    pub fn fail_to_delete_engine<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(FailToDeleteEngine, e)
    }

    pub fn fail_to_load_config<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(FailToLoadConfig, e)
    }

    pub fn fail_to_connect_database<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(FailToConnectDatabase, e)
    }

    pub fn fail_to_update_database<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(FailToUpdateDatabase, e)
    }

    pub fn fail_to_read_database<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::new(FailToReadDatabase, e)
    }

    fn new<E: Into<anyhow::Error>>(error_type: RucatErrorType, content: E) -> Self {
        RucatError {
            error_type,
            content: content.into(),
        }
    }
}

impl Display for RucatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.error_type, self.content)
    }
}

impl std::error::Error for RucatError {}

impl<T> From<RucatError> for Result<T> {
    fn from(val: RucatError) -> Self {
        Result::Err(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized() {
        let error = RucatError::unauthorized(anyhow!("err_msg"));
        assert!(error.to_string().contains("Unauthorized: err_msg"));
    }

    #[test]
    fn not_allowed() {
        let error = RucatError::not_allowed(anyhow!("err_msg"));
        assert!(error.to_string().contains("Not allowed: err_msg"));
    }

    #[test]
    fn engine_not_found() -> Result<()> {
        let error = RucatError::engine_not_found(&EngineId::try_from("0")?);
        assert!(error.to_string().contains("Not found: Engine 0 not found"));
        Ok(())
    }

    #[test]
    fn not_found() {
        let error = RucatError::not_found(anyhow!("err_msg"));
        assert!(error.to_string().contains("Not found: err_msg"));
    }

    #[test]
    fn fail_to_start_server() {
        let error = RucatError::fail_to_start_server(anyhow!("err_msg"));
        assert!(error.to_string().contains("Fail to start server: err_msg"));
    }

    #[test]
    fn fail_to_start_engine() {
        let error = RucatError::fail_to_start_engine(anyhow!("err_msg"));
        assert!(error.to_string().contains("Fail to start engine: err_msg"));
    }

    #[test]
    fn fail_to_delete_engine() {
        let error = RucatError::fail_to_delete_engine(anyhow!("err_msg"));
        assert!(error.to_string().contains("Fail to delete engine: err_msg"));
    }

    #[test]
    fn fail_to_load_config() {
        let error = RucatError::fail_to_load_config(anyhow!("err_msg"));
        assert!(error.to_string().contains("Fail to load config: err_msg"));
    }

    #[test]
    fn fail_to_connect_database() {
        let error = RucatError::fail_to_connect_database(anyhow!("err_msg"));
        assert!(error
            .to_string()
            .contains("Fail to connect to database: err_msg"));
    }

    #[test]
    fn fail_to_update_database() {
        let error = RucatError::fail_to_update_database(anyhow!("err_msg"));
        assert!(error
            .to_string()
            .contains("Fail to update database: err_msg"));
    }

    #[test]
    fn fail_to_read_database() {
        let error = RucatError::fail_to_read_database(anyhow!("err_msg"));
        assert!(error.to_string().contains("Fail to read database: err_msg"));
    }

    #[test]
    fn nested_error() {
        let error = RucatError::fail_to_start_engine(RucatError::fail_to_update_database(anyhow!(
            "err_msg"
        )));
        assert!(error
            .to_string()
            .contains("Fail to start engine: Fail to update database: err_msg"));
    }
}
