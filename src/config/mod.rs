use awt_simulation::Config as SimulationConfig;
use core::time::Duration;
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use rayon::iter::plumbing::UnindexedConsumer;
use rayon::prelude::*;
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
    pub fn get(&self, i: usize) -> SimulationConfig {
        let rng: Box<SmallRng> = if let Some(seeds) = &self.rng_seeds {
            let seed = seeds.get(i).expect("oof");
            Box::new(SmallRng::seed_from_u64(*seed))
        } else {
            Box::new(SmallRng::from_rng(thread_rng()).unwrap())
        };

        let mut simulation_config = SimulationConfig::new(self.tick_until, self.tick_size, rng);
        for client in self.clients() {
            simulation_config.add_client(client);
        }
        for server in self.servers() {
            simulation_config.add_server(server);
        }

        simulation_config
    }
}

impl ParallelIterator for Config {
    type Item = (usize, Self);

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        (0..self.simulations)
            .into_par_iter()
            .map(|sim| (sim, self.clone()))
            .drive_unindexed(consumer)
    }
}

impl Config {
    pub fn clients(&self) -> Vec<awt_simulation::client::Client> {
        self.clients
            .iter()
            .flat_map(|client_config| {
                (0..client_config.quantity)
                    .map(|_| crate::Client::from(client_config))
                    .collect::<Vec<crate::Client>>()
            })
            .collect()
    }

    pub fn servers(&self) -> Vec<awt_simulation::server::Server> {
        self.servers
            .iter()
            .flat_map(|server_config| {
                (0..server_config.quantity)
                    .map(|_| crate::Server::from(server_config))
                    .collect::<Vec<crate::Server>>()
            })
            .collect()
    }

    pub fn metrics(&self) -> Result<Vec<awt_metrics::Metric>, ConfigError> {
        self.metrics
            .iter()
            .map(crate::Metric::try_from)
            .collect::<Result<Vec<crate::Metric>, metric::MetricError>>()
            .map_err(ConfigError::Metric)
    }
}
