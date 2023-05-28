use alloc::boxed::Box;
use core::time::Duration;
use rand::RngCore;

use crate::client::Client;
use crate::server::Server;
use crate::Simulation;

pub struct Config {
    pub(crate) end: Duration,
    pub(crate) tick_size: Duration,
    pub(crate) clients: Vec<Client>,
    pub(crate) servers: Vec<Server>,
    pub(crate) rng: Box<dyn RngCore>,
}

impl alloc::fmt::Debug for Config {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Config")
            .field("end", &self.end)
            .field("tick_size", &self.tick_size)
            .field("clients", &self.clients)
            .field("servers", &self.servers)
            .finish()
    }
}

// Constructors
impl Config {
    #[must_use]
    pub fn new(end: Duration, tick_size: Duration, rng: Box<dyn RngCore>) -> Self {
        Self {
            end,
            tick_size,
            clients: vec![],
            servers: vec![],
            rng,
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
}

impl From<Config> for Simulation {
    /// Generate a Simulation from a `Config`
    ///
    /// This function consumes the provided config so ensure the config is cloned before trying to
    /// use in other simulations.
    fn from(mut config: Config) -> Self {
        let mut sim = Self::new(config.end, config.tick_size, config.rng);
        sim.add_servers(config.servers);
        sim.add_clients(&mut config.clients);
        sim
    }
}
