//! Configuration for rucat server and engine.

use crate::error::Result;
use clap::Parser;
use serde::Deserialize;
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
#[derive(Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum DataBaseType {
    /// Embedded database runs in the same process as the rucat server
    Embedded,
    /// database runs in a separate process locally
    Local(String),
}

#[derive(Deserialize)]
pub struct Config {
    pub auth_enable: bool,
    pub engine_binary_path: String,
    pub database: DataBaseType,
}

impl Config {
    pub fn read_config(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = from_reader(reader)?;
        Ok(config)
    }
}
