use std::net::SocketAddr;

use time::{
    format_description::BorrowedFormatItem, macros::format_description, Duration, OffsetDateTime,
};

use serde::{Deserialize, Serialize};

/// Type of time in engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineTime(String);

impl EngineTime {
    /// The format description of the time in engine.
    const FORMAT_DESC: &'static [BorrowedFormatItem<'static>] = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
    );

    /// Create a new [EngineTime] with the current time.
    pub fn now() -> Self {
        Self(
            // Use `unwrap` because the format is fixed.
            OffsetDateTime::now_utc().format(Self::FORMAT_DESC).unwrap(),
        )
    }

    /// Get the elapsed time from the time of this [EngineTime].
    pub fn elapsed_time(&self) -> Duration {
        let now = OffsetDateTime::now_utc();
        // Use `unwrap` because the format is fixed.
        let time = OffsetDateTime::parse(&self.0, Self::FORMAT_DESC).unwrap();
        now - time
    }
}

/// States of Rucat engine
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EngineState {
    /// Engine is pending to be started.
    Pending,
    /// Engine is running.
    Running,
    /// Engine is stopped.
    Stopped,
}

/// Types of Rucat engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngineType {
    /// Ballista in local mode
    BallistaLocal,
    /// Ballista in remote mode, e.g. on k8s.
    BallistaRemote,
    Rucat,
}

/// Connection information of an engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConnection {
    endpoint: String,
    update_time: EngineTime,
}

impl From<SocketAddr> for EngineConnection {
    fn from(addr: SocketAddr) -> Self {
        Self {
            endpoint: addr.to_string(),
            update_time: EngineTime::now(),
        }
    }
}

impl EngineConnection {
    pub fn renew(&mut self) {
        self.update_time = EngineTime::now();
    }
}

/// Whole information of an engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    name: String,
    engine_type: EngineType,
    /// The address of the engine.
    // We don't embed `endpoint` in `EngineState` because SurrealQL doesn't support pattern matching.`
    connection: Option<EngineConnection>,
    state: EngineState,
    created_time: EngineTime,
}

impl EngineInfo {
    pub fn new(
        name: String,
        engine_type: EngineType,
        state: EngineState,
        connection: Option<EngineConnection>,
    ) -> Self {
        Self {
            name,
            engine_type,
            state,
            connection,
            created_time: EngineTime::now(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_connection(&self) -> Option<&EngineConnection> {
        self.connection.as_ref()
    }

    pub fn get_engine_type(&self) -> &EngineType {
        &self.engine_type
    }

    pub fn get_state(&self) -> &EngineState {
        &self.state
    }
}
