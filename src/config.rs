use toml;

#[macro_use]
use serde_derive::Deserialize;

use std::collections::HashMap;

use std::fs::File;
use std::io::prelude::*;

use std::error::Error;
pub use toml::value::*;

#[derive(Debug, Deserialize)]
pub struct Config {
    //// Server address
    pub(crate) address: String,
    //// Server port
    pub(crate) port: u16,
    //// User defined configuration values
    pub(crate) config: Option<HashMap<String, Value>>,
}

impl Config {
    pub fn parse_config() -> Result<Config, Box<Error>> {
        let mut config_file = File::open("Catalyst.toml")?;
        let mut config_string = String::new();
        config_file.read_to_string(&mut config_string)?;

        let config: Config = toml::from_str(&config_string[..]).unwrap();
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: String::from("127.0.0.1"),
            port: 3000,
            config: None,
        }
    }
}
