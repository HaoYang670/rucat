use ::rucat_common::{config::DatabaseConfig, serde::Deserialize};

/// Configuration for rucat state monitor
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(crate = "rucat_common::serde")]
pub struct StateMonitorConfig {
    /// Time interval in second for checking engine state
    pub check_interval_secs: u8,
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
        serde_json::{from_value, json},
    };

    use super::*;

    #[test]
    fn missing_field_check_interval_secs() {
        let config = json!(
            {
                "database": {
                    "credentials": null,
                    "uri": ""
                }
            }
        );
        let result = from_value::<StateMonitorConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "missing field `check_interval_secs`"
        );
    }

    #[test]
    fn missing_field_database() {
        let config = json!(
            {
                "check_interval_secs": 1
            }
        );
        let result = from_value::<StateMonitorConfig>(config);
        assert_eq!(result.unwrap_err().to_string(), "missing field `database`");
    }

    #[test]
    fn deny_unknown_fields() {
        let config = json!(
            {
                "check_interval_secs": 1,
                "database": {
                    "credentials": null,
                    "uri": ""
                },
                "unknown_field": "unknown"
            }
        );
        let result = from_value::<StateMonitorConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "unknown field `unknown_field`, expected `check_interval_secs` or `database`"
        );
    }

    #[test]
    fn deserialize_state_monitor_config() -> Result<()> {
        let config = json!(
            {
                "check_interval_secs": 1,
                "database": {
                    "credentials": null,
                    "uri":""
                }
            }
        );
        let result = from_value::<StateMonitorConfig>(config)?;
        assert_eq!(
            result,
            StateMonitorConfig {
                check_interval_secs: 1,
                database: DatabaseConfig {
                    credentials: None,
                    uri: "".to_string()
                }
            }
        );
        Ok(())
    }
}
