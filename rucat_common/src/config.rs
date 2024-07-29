//! Configuration for rucat server and engine.

use crate::{error::Result, EngineId};
use clap::Parser;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::from_reader;
use std::{fs::File, io::BufReader};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Command line arguments for rucat server and engine.
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

/// Variant for user to choose the database type when creating the server
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum DataBaseType {
    /// Embedded database runs in the same process as the rucat server
    Embedded,
    /// database runs in a separate process locally
    Local(String),
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub auth_enable: bool,
    pub engine_binary_path: String,
    pub db_type: DataBaseType,
}

#[derive(Deserialize, Serialize)]
pub struct EngineConfig {
    pub engine_id: EngineId,
    /// only support local mode now
    pub db_endpoint: String,
}

/// Parse [ServerConfig] or [EngineConfig] from the config file.
pub fn read_config<T: DeserializeOwned>(path: &str) -> Result<T> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = from_reader(reader)?;
    Ok(config)
}
