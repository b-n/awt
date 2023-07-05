use core::time::Duration;
use rand::{prelude::*, rngs::SmallRng, thread_rng, SeedableRng};
use rayon::iter::plumbing::UnindexedConsumer;
use rayon::prelude::*;

use super::ConfigError;

use awt_metrics::Metric;
use awt_simulation::{client::Client, server::Server, Config as SimulationConfig};

#[derive(Default, Clone, Debug)]
pub struct Parsed {
    simulations: usize,
    tick_size: Duration,
    tick_until: Duration,
    clients: Vec<Client>,
    servers: Vec<Server>,
    metrics: Vec<Metric>,
    rng_seeds: Vec<u64>,
}

impl TryFrom<super::Config> for Parsed {
    type Error = ConfigError;

    fn try_from(config: super::Config) -> Result<Self, Self::Error> {
        // Use the seeds if provided, otherwise ensure all seeds are generated
        let rng_seeds = if let Some(seeds) = &config.rng_seeds {
            seeds.clone()
        } else {
            let mut rng = thread_rng();
            (0..config.simulations).map(|_| rng.gen()).collect()
        };

        let parsed = Parsed {
            simulations: config.simulations,
            tick_size: config.tick_size,
            tick_until: config.tick_until,
            clients: config
                .clients
                .iter()
                .flat_map(|client_config| {
                    (0..client_config.quantity)
                        .map(|_| Client::from(client_config))
                        .collect::<Vec<Client>>()
                })
                .collect(),
            servers: config
                .servers
                .iter()
                .flat_map(|server_config| {
                    (0..server_config.quantity)
                        .map(|_| Server::from(server_config))
                        .collect::<Vec<Server>>()
                })
                .collect(),
            metrics: config
                .metrics
                .iter()
                .map(Metric::try_from)
                .collect::<Result<Vec<Metric>, super::metric::MetricError>>()
                .map_err(ConfigError::Metric)?,
            rng_seeds,
        };

        Ok(parsed)
    }
}

impl Parsed {
    /// Generates a `SimulationConfig` which can be passed into a `awt-simulation`.
    pub fn get(self, i: usize) -> SimulationConfig {
        // Unwrap since parsing should gaurantee the rng_seed exists
        let seed = self.rng_seeds.get(i).unwrap();
        let rng = Box::new(SmallRng::seed_from_u64(*seed));

        let mut simulation_config = SimulationConfig::new(self.tick_until, self.tick_size, rng);
        simulation_config.set_clients(self.clients);
        simulation_config.set_servers(self.servers);

        simulation_config
    }

    pub fn metrics(&self) -> Vec<Metric> {
        self.metrics.clone()
    }
}

impl ParallelIterator for Parsed {
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
