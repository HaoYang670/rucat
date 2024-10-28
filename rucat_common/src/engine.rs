use ::core::fmt::Display;
use ::std::{collections::HashMap, fmt};

use ::anyhow::anyhow;
use ::serde::{
    de::{self, Visitor},
    Deserializer,
};
use time::{
    format_description::BorrowedFormatItem, macros::format_description, Duration, OffsetDateTime,
};

use crate::error::{Result, RucatError};
use serde::{Deserialize, Serialize};

/// Preset configurations that are not allowed to be set.
type PresetConfig = (&'static str, fn(&EngineId) -> String);
const PRESET_CONFIGS: [PresetConfig; 6] = [
    ("spark.app.id", get_spark_app_id),
    ("spark.kubernetes.container.image", |_| {
        "apache/spark:3.5.3".to_owned()
    }),
    ("spark.driver.host", get_spark_service_name),
    ("spark.kubernetes.driver.pod.name", get_spark_driver_name),
    ("spark.kubernetes.executor.podNamePrefix", get_spark_app_id),
    ("spark.driver.extraJavaOptions", |_| {
        "-Divy.cache.dir=/tmp -Divy.home=/tmp".to_owned()
    }),
];

pub fn get_spark_app_id(id: &EngineId) -> String {
    format!("rucat-spark-{}", id)
}

pub fn get_spark_driver_name(id: &EngineId) -> String {
    format!("{}-driver", get_spark_app_id(id))
}
pub fn get_spark_service_name(id: &EngineId) -> String {
    get_spark_app_id(id)
}

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

/// User-defined configuration for creating an Spark app.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EngineConfigs(HashMap<String, String>);

impl EngineConfigs {
    /// Convert the configuration to the format of `spark-submit`.
    /// The preset configurations are set with the given engine id.
    pub fn to_spark_submit_format_with_preset_configs(&self, id: &EngineId) -> Vec<String> {
        // Safety: keys in `PRESET_CONFIG` and `self.0` are not overlapping.
        PRESET_CONFIGS
            .iter()
            .map(|(k, v)| format!("{}={}", k, v(id)))
            .chain(self.0.iter().map(|(k, v)| format!("{}={}", k, v)))
            .flat_map(|conf| ["--conf".to_owned(), conf])
            .collect()
    }
}

impl TryFrom<HashMap<String, String>> for EngineConfigs {
    type Error = RucatError;

    fn try_from(config: HashMap<String, String>) -> Result<Self> {
        PRESET_CONFIGS
            .iter()
            .map(|(key, _)| key)
            .find(|&&key| config.contains_key(key))
            .map_or(Ok(Self(config)), |key| {
                Err(RucatError::not_allowed(anyhow!(
                    "The config {} is not allowed as it is reserved.",
                    key
                )))
            })
    }
}

/// States of Rucat engine
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum EngineState {
    Pending,
    Running,
    Stopped,
}

/// Whole information of an engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub name: String,
    pub state: EngineState,
    pub config: EngineConfigs,
    created_time: EngineTime,
}

impl EngineInfo {
    pub fn new(name: String, state: EngineState, config: EngineConfigs) -> Self {
        Self {
            name,
            state,
            config,
            created_time: EngineTime::now(),
        }
    }
}

/// Unique identifier for an engine.
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Serialize)]
pub struct EngineId {
    id: String,
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

impl<'de> Visitor<'de> for EngineIdVisitor {
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
        if id.is_empty() {
            Err(RucatError::not_allowed(anyhow!(
                "Engine id cannot be empty."
            )))
        } else {
            Ok(Self { id })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_spark_app_id() -> Result<()> {
        let id = EngineId::try_from("abc".to_owned())?;
        assert_eq!(get_spark_app_id(&id), "rucat-spark-abc");
        Ok(())
    }

    #[test]
    fn test_get_spark_driver_name() -> Result<()> {
        let id = EngineId::try_from("abc".to_owned())?;
        assert_eq!(get_spark_driver_name(&id), "rucat-spark-abc-driver");
        Ok(())
    }

    #[test]
    fn test_get_spark_service_name() -> Result<()> {
        let id = EngineId::try_from("abc".to_owned())?;
        assert_eq!(get_spark_service_name(&id), "rucat-spark-abc");
        Ok(())
    }

    mod engine_id {
        use ::serde_json::json;

        use super::*;

        #[test]
        fn engine_id_cannot_be_empty() {
            let result = EngineId::try_from("".to_owned());
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
            assert_eq!(result, EngineId::try_from("abc".to_owned())?);
            Ok(())
        }
    }

    mod engine_config {
        use super::*;

        fn check_preset_config(key: &str) {
            let config = HashMap::from([(key.to_owned(), "".to_owned())]);
            let result = EngineConfigs::try_from(config);
            assert!(result.is_err_and(|e| e.to_string().starts_with(&format!(
                "Not allowed: The config {} is not allowed as it is reserved.",
                key
            ))));
        }

        #[test]
        fn preset_configs_are_not_allowed_to_be_set() {
            check_preset_config("spark.app.id");
            check_preset_config("spark.kubernetes.container.image");
            check_preset_config("spark.driver.host");
            check_preset_config("spark.kubernetes.driver.pod.name");
            check_preset_config("spark.kubernetes.executor.podNamePrefix");
            check_preset_config("spark.driver.extraJavaOptions");
        }

        #[test]
        fn empty_engine_config() -> Result<()> {
            let result = EngineConfigs::try_from(HashMap::new())?;
            assert!(result.0 == HashMap::new());

            let spark_submit_format = result
                .to_spark_submit_format_with_preset_configs(&EngineId::try_from("abc".to_owned())?);
            assert_eq!(
                spark_submit_format,
                vec![
                    "--conf",
                    "spark.app.id=rucat-spark-abc",
                    "--conf",
                    "spark.kubernetes.container.image=apache/spark:3.5.3",
                    "--conf",
                    "spark.driver.host=rucat-spark-abc",
                    "--conf",
                    "spark.kubernetes.driver.pod.name=rucat-spark-abc-driver",
                    "--conf",
                    "spark.kubernetes.executor.podNamePrefix=rucat-spark-abc",
                    "--conf",
                    "spark.driver.extraJavaOptions=-Divy.cache.dir=/tmp -Divy.home=/tmp",
                ]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
            );
            Ok(())
        }

        #[test]
        fn engine_config_with_1_item() -> Result<()> {
            let config = HashMap::from([("spark.executor.instances".to_owned(), "2".to_owned())]);
            let result = EngineConfigs::try_from(config.clone())?;
            assert!(result.0 == config);

            let spark_submit_format = result
                .to_spark_submit_format_with_preset_configs(&EngineId::try_from("abc".to_owned())?);
            assert_eq!(
                spark_submit_format,
                vec![
                    "--conf",
                    "spark.app.id=rucat-spark-abc",
                    "--conf",
                    "spark.kubernetes.container.image=apache/spark:3.5.3",
                    "--conf",
                    "spark.driver.host=rucat-spark-abc",
                    "--conf",
                    "spark.kubernetes.driver.pod.name=rucat-spark-abc-driver",
                    "--conf",
                    "spark.kubernetes.executor.podNamePrefix=rucat-spark-abc",
                    "--conf",
                    "spark.driver.extraJavaOptions=-Divy.cache.dir=/tmp -Divy.home=/tmp",
                    "--conf",
                    "spark.executor.instances=2",
                ]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
            );
            Ok(())
        }
    }
}
