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

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// Database config
pub enum DatabaseVariant {
    Surreal {
        credentials: Option<Credentials>,
        uri: String,
    },
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
    fn allow_missing_credentials() -> Result<()> {
        let config = json!(
            {
                "Surreal": { "uri": "" }
            }
        );
        let result = from_value::<DatabaseVariant>(config)?;
        assert_eq!(
            result,
            DatabaseVariant::Surreal {
                credentials: None,
                uri: "".to_string()
            }
        );
        Ok(())
    }

    #[test]
    fn missing_uri() {
        let config = json!(
            {
                "Surreal": { "credentials": null }
            }
        );
        let result = from_value::<DatabaseVariant>(config);
        assert_eq!(result.unwrap_err().to_string(), "missing field `uri`");
    }

    #[test]
    fn deny_unknown_fields() {
        let config = json!(
            {
                "Surreal": {
                    "credentials": null,
                    "uri": "",
                    "unknown_field": "unknown"
                }
            }
        );
        let result = from_value::<DatabaseVariant>(config);
        assert_eq!(
            result.unwrap_err().to_string(),
            "unknown field `unknown_field`, expected `credentials` or `uri`"
        );
    }

    #[test]
    fn deserialize_database_config() -> Result<()> {
        let config = json!(
            {
                "Surreal": {
                    "credentials": {
                        "username": "admin",
                        "password": "pwd"
                    },
                    "uri": "localhost:27017"
                }
            }
        );
        let result = from_value::<DatabaseVariant>(config)?;
        assert_eq!(
            result,
            DatabaseVariant::Surreal {
                credentials: Some(Credentials {
                    username: "admin".to_string(),
                    password: "pwd".to_string()
                }),
                uri: "localhost:27017".to_string()
            }
        );
        Ok(())
    }
}
