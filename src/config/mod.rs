use awt_simulation::Config as SimulationConfig;
use core::time::Duration;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;

mod client;
mod metric;
mod server;

use client::Client;
use metric::Metric;
use server::Server;

#[derive(Deserialize)]
pub struct Config {
    clients: Vec<Client>,
    servers: Vec<Server>,
    metrics: Vec<Metric>,
    #[serde(default)]
    pub simulations: usize,
    pub tick_size: Duration,
    pub tick_until: Duration,
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

impl Config {
    pub fn simulation_config(&self) -> SimulationConfig {
        let mut simulation_config = SimulationConfig::new(self.tick_until, self.tick_size);
        for client in self.clients() {
            simulation_config.add_client(client);
        }
        for server in self.servers() {
            simulation_config.add_server(server);
        }
        simulation_config
    }
}

impl Config {
    pub fn clients(&self) -> Vec<crate::Client> {
        self.clients
            .iter()
            .flat_map(|client_config| {
                (0..client_config.quantity)
                    .map(|_| crate::Client::from(client_config))
                    .collect::<Vec<crate::Client>>()
            })
            .collect()
    }

    pub fn servers(&self) -> Vec<crate::Server> {
        self.servers
            .iter()
            .flat_map(|server_config| {
                (0..server_config.quantity)
                    .map(|_| crate::Server::from(server_config))
                    .collect::<Vec<crate::Server>>()
            })
            .collect()
    }

    pub fn metrics(&self) -> Result<Vec<crate::Metric>, ConfigError> {
        self.metrics
            .iter()
            .map(crate::Metric::try_from)
            .collect::<Result<Vec<crate::Metric>, metric::MetricError>>()
            .map_err(ConfigError::Metric)
    }
}
