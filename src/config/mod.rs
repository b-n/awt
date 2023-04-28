use core::time::Duration;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;

mod client;

pub use client::Client;

#[derive(Deserialize)]
pub struct Config {
    pub clients: Vec<Client>,
    #[serde(default)]
    pub simulations: usize,
    pub tick_size: Duration,
    pub tick_until: Duration,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IOError: Couldn't open file for reading - {0}")]
    IOError(#[from] std::io::Error),
    #[error("DeserializeError: Invalid toml contents - {0}")]
    DeserializeError(#[from] toml::de::Error),
}

impl TryFrom<&PathBuf> for Config {
    type Error = ConfigError;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let mut file = File::open(path)?;
        let mut toml = String::new();

        file.read_to_string(&mut toml)?;

        Ok(toml::from_str::<Config>(&toml)?)
    }
}
