//! Configuration for rucat server

use std::{fs::File, io::BufReader};

use rucat_common::error::Result;
use serde::Deserialize;
use serde_json::from_reader;

/// Variant for user to choose the database type when creating the server
#[derive(Deserialize)]
#[serde(tag = "type", content = "content")]
pub(crate) enum DataBaseType {
    /// Embedded database runs in the same process as the server
    Embedded,
    /// Local database runs in a separate process locally
    Local(String),
}

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) auth_enable: bool,
    pub(crate) engine_binary_path: String,
    pub(crate) database: DataBaseType,
}

impl Config {
    pub(crate) fn read_config(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = from_reader(reader)?;
        Ok(config)
    }
}
