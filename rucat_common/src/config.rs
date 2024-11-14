//! Configuration for rucat server and engine.

use crate::error::{Result, RucatError};
use clap::Parser;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::from_reader;
use std::{fs::File, io::BufReader};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Command line arguments for rucat server.
pub struct Args {
    /// path to the config file
    #[arg(long)]
    pub config_path: String,
}

impl Args {
    /// helper function for exporting the `clap::Parser::parse` function
    pub fn parse_args() -> Self {
        Args::parse()
    }
}

/// Credentials for the database
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

/// Variant for user to choose the database type when creating the server
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum DatabaseVariant {
    /// Embedded database has the same lifetime as the server
    /// and cannot be shared between servers
    Embedded,
    /// database runs in a separate process locally
    Local { uri: String },
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    pub credentials: Option<Credentials>,
    pub variant: DatabaseVariant,
}

/// Parse config from file.
pub fn load_config<T: DeserializeOwned>(path: &str) -> Result<T> {
    let file = File::open(path).map_err(RucatError::fail_to_load_config)?;
    let reader = BufReader::new(file);
    let config = from_reader(reader).map_err(RucatError::fail_to_load_config)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use ::anyhow::Result;
    use ::serde_json::{from_value, json};

    use super::*;

    #[test]
    fn allow_missing_field_credentials() -> Result<()> {
        let config = json!(
            {
                "variant": {
                    "type": "Embedded"
                }
            }
        );
        let result = from_value::<DatabaseConfig>(config)?;
        assert_eq!(
            result,
            DatabaseConfig {
                credentials: None,
                variant: DatabaseVariant::Embedded
            }
        );
        Ok(())
    }

    #[test]
    fn missing_field_variant() {
        let config = json!(
            {
                "credentials": null
            }
        );
        let result = from_value::<DatabaseConfig>(config);
        assert_eq!(result.unwrap_err().to_string(), "missing field `variant`");
    }

    #[test]
    fn deny_unknown_fields() {
        let config = json!(
            {
                "credentials": null,
                "variant": {
                    "type": "Embedded"
                },
                "unknown_field": "unknown"
            }
        );
        let result = from_value::<DatabaseConfig>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "unknown field `unknown_field`, expected `credentials` or `variant`"
        );
    }

    #[test]
    fn deserialize_embedded_database_config() -> Result<()> {
        let config = json!(
            {
                "credentials": null,
                "variant": {
                    "type": "Embedded"
                }
            }
        );
        let result = from_value::<DatabaseConfig>(config)?;
        assert_eq!(
            result,
            DatabaseConfig {
                credentials: None,
                variant: DatabaseVariant::Embedded
            }
        );
        Ok(())
    }

    #[test]
    fn deserialize_local_database_config() -> Result<()> {
        let config = json!(
            {
                "credentials": {
                    "username": "admin",
                    "password": "pwd"
                },
                "variant": {
                    "type": "Local",
                    "uri": "localhost:27017"
                }
            }
        );
        let result = from_value::<DatabaseConfig>(config)?;
        assert_eq!(
            result,
            DatabaseConfig {
                credentials: Some(Credentials {
                    username: "admin".to_string(),
                    password: "pwd".to_string()
                }),
                variant: DatabaseVariant::Local {
                    uri: "localhost:27017".to_string()
                }
            }
        );
        Ok(())
    }
}
