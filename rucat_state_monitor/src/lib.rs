use ::core::num::NonZeroU64;

use ::rucat_common::{config::DatabaseConfig, serde::Deserialize};

/// Configuration for rucat state monitor
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(crate = "rucat_common::serde")]
pub struct StateMonitorConfig {
    /// Time interval in millisecond for checking each Spark state
    pub check_interval_millis: NonZeroU64,
    pub database: DatabaseConfig,
}

/// Load the configuration from the file
/// Unlike rucat server, we don't allow users to specify the config file path
/// because state monitor is a background service.
pub static CONFIG_FILE_PATH: &str = "/rucat_state_monitor/config.json";

#[cfg(test)]
mod tests {
    use ::rucat_common::{
        anyhow::Result,
        config::DatabaseVariant,
        serde_json::{from_value, json},
    };

    use super::*;

    #[test]
    fn check_interval_millis_cannot_be_zero() {
        let config = json!(
            {
                "check_interval_millis": 0,
                "database": {
                    "credentials": null,
                    "variant": {
                        "type": "Embedded"
                    }
                }
            }
        );
        let result = from_value::<StateMonitorConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid value: integer `0`, expected a nonzero u64"
        );
    }

    #[test]
    fn missing_field_check_interval_millis() {
        let config = json!(
            {
                "database": {
                    "credentials": null,
                    "variant": {
                        "type": "Embedded"
                    }
                }
            }
        );
        let result = from_value::<StateMonitorConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "missing field `check_interval_millis`"
        );
    }

    #[test]
    fn missing_field_database() {
        let config = json!(
            {
                "check_interval_millis": 1000
            }
        );
        let result = from_value::<StateMonitorConfig>(config);
        assert_eq!(result.unwrap_err().to_string(), "missing field `database`");
    }

    #[test]
    fn deny_unknown_fields() {
        let config = json!(
            {
                "check_interval_millis": 1000,
                "database": {
                    "credentials": null,
                    "variant": {
                        "type": "Embedded"
                    }
                },
                "unknown_field": "unknown"
            }
        );
        let result = from_value::<StateMonitorConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "unknown field `unknown_field`, expected `check_interval_millis` or `database`"
        );
    }

    #[test]
    fn deserialize_state_monitor_config() -> Result<()> {
        let config = json!(
            {
                "check_interval_millis": 1000,
                "database": {
                    "credentials": null,
                    "variant": {
                        "type": "Embedded"
                    }
                }
            }
        );
        let result = from_value::<StateMonitorConfig>(config)?;
        assert_eq!(
            result,
            StateMonitorConfig {
                check_interval_millis: NonZeroU64::new(1000).unwrap(),
                database: DatabaseConfig {
                    credentials: None,
                    variant: DatabaseVariant::Embedded
                }
            }
        );
        Ok(())
    }
}
