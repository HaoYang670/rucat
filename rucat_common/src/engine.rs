use ::core::fmt::Display;
use ::std::{borrow::Cow, collections::BTreeMap, fmt};

use ::anyhow::anyhow;
use ::serde::{
    de::{self, Visitor},
    Deserializer,
};
use time::{
    format_description::BorrowedFormatItem, macros::format_description, Duration, OffsetDateTime,
};

use crate::{
    engine::EngineState::WaitToStart,
    error::{Result, RucatError},
};
use serde::{Deserialize, Serialize};

pub fn get_spark_app_id(id: &EngineId) -> Cow<'static, str> {
    Cow::Owned(format!("rucat-spark-{}", id))
}

pub fn get_spark_driver_name(id: &EngineId) -> Cow<'static, str> {
    Cow::Owned(format!("{}-driver", get_spark_app_id(id)))
}
pub fn get_spark_service_name(id: &EngineId) -> Cow<'static, str> {
    get_spark_app_id(id)
}

/// Request body to start an engine.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StartEngineRequest {
    // The name of the engine
    name: String,
    // Spark configurations
    configs: Option<EngineConfigs>,
}

/// Type of time in engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineTime(String);

impl EngineTime {
    /// The format description of the time in engine.
    const FORMAT_DESC: &'static [BorrowedFormatItem<'static>] = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]:[offset_second]"
    );

    /// Return a new [EngineTime] with the current time.
    pub fn now() -> Self {
        Self(
            // Use `unwrap` because the format is fixed.
            OffsetDateTime::now_utc().format(Self::FORMAT_DESC).unwrap(),
        )
    }

    /// Get the elapsed time from the time of this [EngineTime].
    /// TODO: remove this if not used.
    pub fn elapsed_time(&self) -> Duration {
        let now = OffsetDateTime::now_utc();
        // Use `unwrap` because the format is fixed.
        let time = OffsetDateTime::parse(&self.0, Self::FORMAT_DESC).unwrap();
        now - time
    }
}

pub type EngineConfigs = BTreeMap<Cow<'static, str>, Cow<'static, str>>;

/// States of Rucat engine
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EngineState {
    WaitToStart,
    TriggerStart,
    StartInProgress,
    Running,
    WaitToTerminate,
    TriggerTermination,
    TerminateInProgress,
    Terminated,
    // TODO: use COW<'static, str> instead of String
    ErrorWaitToClean(String),
    ErrorTriggerClean(String),
    ErrorCleanInProgress(String),
    ErrorClean(String),
}

/// Whole information of an engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub name: String,
    pub state: EngineState,
    pub config: EngineConfigs,
    /// time when the engine is created.
    ///
    /// Note, this is **not** the start time when the engine is RUNNING.
    create_time: EngineTime,
}

impl EngineInfo {
    pub fn new(
        name: String,
        state: EngineState,
        config: EngineConfigs,
        create_time: EngineTime,
    ) -> Self {
        Self {
            name,
            state,
            config,
            create_time,
        }
    }
}

impl TryFrom<StartEngineRequest> for EngineInfo {
    type Error = RucatError;

    fn try_from(value: StartEngineRequest) -> Result<Self> {
        Ok(EngineInfo::new(
            value.name,
            WaitToStart,
            value.configs.unwrap_or_default(),
            EngineTime::now(),
        ))
    }
}

/// Unique identifier for an engine.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Serialize)]
pub struct EngineId {
    id: Cow<'static, str>,
}

impl EngineId {
    pub fn new(id: Cow<'static, str>) -> Result<Self> {
        if id.is_empty() {
            Err(RucatError::not_allowed(anyhow!(
                "Engine id cannot be empty."
            )))
        } else {
            Ok(Self { id })
        }
    }
}

impl<'de> Deserialize<'de> for EngineId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(EngineIdVisitor)
    }
}

struct EngineIdVisitor;

impl Visitor<'_> for EngineIdVisitor {
    type Value = EngineId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a non-empty string representing an EngineId")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        EngineId::try_from(value.to_owned()).map_err(de::Error::custom)
    }
}

impl Display for EngineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl TryFrom<String> for EngineId {
    type Error = RucatError;
    fn try_from(id: String) -> Result<Self> {
        Self::new(Cow::Owned(id))
    }
}

impl TryFrom<&'static str> for EngineId {
    type Error = RucatError;
    fn try_from(id: &'static str) -> Result<Self> {
        Self::new(Cow::Borrowed(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_spark_app_id() -> Result<()> {
        let id = EngineId::try_from("abc")?;
        assert_eq!(get_spark_app_id(&id), "rucat-spark-abc");
        Ok(())
    }

    #[test]
    fn test_get_spark_driver_name() -> Result<()> {
        let id = EngineId::try_from("abc")?;
        assert_eq!(get_spark_driver_name(&id), "rucat-spark-abc-driver");
        Ok(())
    }

    #[test]
    fn test_get_spark_service_name() -> Result<()> {
        let id = EngineId::try_from("abc")?;
        assert_eq!(get_spark_service_name(&id), "rucat-spark-abc");
        Ok(())
    }

    mod engine_id {
        use ::serde_json::json;

        use super::*;

        #[test]
        fn engine_id_cannot_be_empty() {
            let result = EngineId::try_from("");
            assert!(result.is_err_and(|e| e
                .to_string()
                .starts_with("Not allowed: Engine id cannot be empty.")));
        }

        #[test]
        fn cannot_deserialize_empty_str_to_engine_id() {
            let result: std::result::Result<EngineId, _> = serde_json::from_value(json!(""));
            assert!(result.is_err_and(|e| e
                .to_string()
                .starts_with("Not allowed: Engine id cannot be empty.")));
        }

        #[test]
        fn deserialize_engine_id() -> anyhow::Result<()> {
            let result: EngineId = serde_json::from_value(json!("abc"))?;
            assert_eq!(result, EngineId::try_from("abc")?);
            Ok(())
        }
    }
}
