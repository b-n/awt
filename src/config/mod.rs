use core::time::Duration;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;

mod attribute;
mod client;
mod metric;
mod parsed;
mod server;

use attribute::Attribute;
use client::Client;
use metric::Metric;
pub use parsed::Parsed;
use server::Server;

#[derive(Default, Clone, Deserialize, Debug)]
pub struct Config {
    clients: Vec<Client>,
    servers: Vec<Server>,
    metrics: Vec<Metric>,
    #[serde(default)]
    pub simulations: usize,
    pub tick_size: Duration,
    pub tick_until: Duration,
    pub rng_seeds: Option<Vec<u64>>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IOError: Couldn't open file for reading - {0}")]
    IO(#[from] std::io::Error),
    #[error("DeserializationError: Invalid toml contents - {0}")]
    Deserialization(#[from] toml::de::Error),
    #[error("Metric: {0}")]
    Metric(metric::MetricError),
    #[error("There should be as many rng_seeds as simulations")]
    BadSeeds,
}

impl TryFrom<&PathBuf> for Config {
    type Error = ConfigError;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let mut file = File::open(path)?;
        let mut toml = String::new();

        file.read_to_string(&mut toml)?;

        let config = toml::from_str::<Config>(&toml)?;

        if let Some(seeds) = &config.rng_seeds {
            if seeds.len() != config.simulations {
                return Err(Self::Error::BadSeeds);
            }
        }

        Ok(config)
    }
}

impl Config {
    pub fn parsed(self) -> Result<Parsed, ConfigError> {
        Parsed::try_from(self)
    }
}
