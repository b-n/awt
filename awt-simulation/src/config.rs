use alloc::boxed::Box;
use core::time::Duration;
use rand::{rngs::SmallRng, thread_rng, RngCore, SeedableRng};

use crate::client::Client;
use crate::metric::Metric;
use crate::server::Server;

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) end: Duration,
    pub(crate) tick_size: Duration,
    pub(crate) clients: Vec<Client>,
    pub(crate) servers: Vec<Server>,
    pub(crate) metrics: Vec<Metric>,
    /// If None provided, the use thread_rng() to generate a seed
    rng_seed: Option<u64>,
}

// Constructors
impl Config {
    #[must_use]
    pub fn new(end: Duration, tick_size: Duration) -> Self {
        Self {
            end,
            tick_size,
            clients: vec![],
            servers: vec![],
            metrics: vec![],
            rng_seed: None,
        }
    }

    #[must_use]
    pub fn with_seed(end: Duration, tick_size: Duration, rng_seed: u64) -> Self {
        Self {
            end,
            tick_size,
            clients: vec![],
            servers: vec![],
            metrics: vec![],
            rng_seed: Some(rng_seed),
        }
    }
}

impl Config {
    pub fn add_client(&mut self, client: Client) {
        self.clients.push(client);
    }

    pub fn add_server(&mut self, server: Server) {
        self.servers.push(server);
    }

    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.push(metric);
    }
}

impl Config {
    pub(crate) fn rng(&self) -> Box<dyn RngCore> {
        if let Some(seed) = self.rng_seed {
            Box::new(SmallRng::seed_from_u64(seed))
        } else {
            Box::new(SmallRng::from_rng(thread_rng()).unwrap())
        }
    }
}
