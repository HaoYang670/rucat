use ::rucat_common::{config::DatabaseVariant, serde::Deserialize};

/// Configuration for rucat state monitor
#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(crate = "rucat_common::serde")]
pub struct StateMonitorConfig {
    /// Time interval in second for checking engine state
    pub check_interval_secs: u8,
    /// Time interval in second for checking trigger state timeout
    /// *Trigger* states are expected to exist only for a very short time,
    /// and then be updated to *InProgress* or *Error* states. However, there is a possibility that
    /// the state monitor is down when the engine is in *Trigger* state, so we need to set a timeout
    /// to avoid the engine being stuck in *Trigger* state. State monitor will pick up those timed out engines
    /// and retrigger them.
    pub trigger_state_timeout_secs: u16,
    pub database: DatabaseVariant,
}

/// Load the configuration from the file
/// Unlike rucat server, we don't allow users to specify the config file path
/// because state monitor is a background service.
pub static CONFIG_FILE_PATH: &str = "/rucat_state_monitor/config.json";

#[cfg(test)]
mod tests {
    use super::*;
    use ::rucat_common::{
        anyhow::Result,
        serde_json::{from_value, json},
    };

    #[test]
    fn missing_field_check_interval_secs() {
        let config = json!(
            {
                "database": {
                    "Surreal": {
                        "credentials": null,
                        "uri": ""
                    }
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
                "check_interval_secs": 1,
                "trigger_state_timeout_secs": 60,
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
                    "Surreal": {
                        "credentials": null,
                        "uri": ""
                    }
                },
                "unknown_field": "unknown"
            }
        );
        let result = from_value::<StateMonitorConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "unknown field `unknown_field`, expected one of `check_interval_secs`, `trigger_state_timeout_secs`, `database`"
        );
    }

    #[test]
    fn deserialize_state_monitor_config() -> Result<()> {
        let config = json!(
            {
                "check_interval_secs": 1,
                "trigger_state_timeout_secs": 60,
                "database": {
                    "Surreal": {
                        "credentials": null,
                        "uri":""
                    }
                }
            }
        );
        let result = from_value::<StateMonitorConfig>(config)?;
        assert_eq!(
            result,
            StateMonitorConfig {
                check_interval_secs: 1,
                trigger_state_timeout_secs: 60,
                database: DatabaseVariant::Surreal {
                    credentials: None,
                    uri: "".to_string()
                }
            }
        );
        Ok(())
    }
}
