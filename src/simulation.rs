use crate::{Client, ClientProfile, Server};
use rand::seq::SliceRandom;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::cell::RefCell;
use std::sync::Arc;

pub const TICKS_PER_SECOND: usize = 1000;
pub const ONE_HOUR: usize = TICKS_PER_SECOND * 60 * 60;

#[derive(Debug, Clone)]
pub struct Simulation {
    tick: usize,
    tick_size: usize,
    tick_until: usize,
    running: bool,
    client_profiles: Vec<Arc<ClientProfile>>,
    servers: Vec<Arc<Server>>,
    clients: Vec<Client>,
    available_servers: Vec<Arc<Server>>,
    queued_clients: Vec<RefCell<Client>>,
    rng: ThreadRng,
}

impl Default for Simulation {
    fn default() -> Self {
        Self {
            tick: 0,
            tick_size: 1,
            tick_until: ONE_HOUR,
            running: false,
            client_profiles: vec![],
            servers: vec![],
            clients: vec![],
            available_servers: vec![],
            queued_clients: vec![],
            rng: thread_rng(),
        }
    }
}

// Structure and setup
impl Simulation {
    pub fn add_server(&mut self, server: Arc<Server>) {
        self.servers.push(server)
    }

    pub fn add_client_profile(&mut self, client_profile: &Arc<ClientProfile>) {
        self.client_profiles.push(client_profile.clone())
    }

    pub fn enable(&mut self) {
        self.running = true;
        self.tick_size = 1000;
        self.generate_clients();
        self.set_servers_available();
    }

    fn generate_clients(&mut self) {
        self.clients = self
            .client_profiles
            .iter()
            .enumerate()
            .map(|(i, cp)| {
                let mut clients = vec![];
                for _ in 0..cp.quantity {
                    let mut client = Client::from(cp);
                    client.set_id(i);
                    clients.push(client)
                }
                clients
            })
            .flatten()
            .collect();

        self.clients.shuffle(&mut self.rng);
    }

    fn set_servers_available(&mut self) {
        self.available_servers = self.servers.clone();
    }
}

// Simulation logic
impl Simulation {
    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }

        self.roll_client();

        // self.check_routing();

        self.increment_tick();

        self.tick_queued();

        self.running
    }

    /// Roll to see whether a new client should be generated from one of the
    /// client_profiles.
    /// 
    /// Returns whether a new client was enqueued or not
    fn roll_client(&mut self) -> bool {
        if self.clients.len() == 0 {
            return false;
        }

        let remaining_rolls = (self.tick_until - self.tick) / self.tick_size;
        
        let roll = self.rng.gen_range(0..=remaining_rolls);

        if roll <= self.clients.len() && let Some(mut next) = self.clients.pop() {
            next.enqueue(self.tick);
            self.queued_clients.push(RefCell::new(next));
            true
        } else {
            false
        }
    }

    fn increment_tick(&mut self) -> bool {
        if self.tick % (5 * TICKS_PER_SECOND) == 0 {
            //println!("Tick: {}", self.tick)
        }
        self.tick += self.tick_size;

        if self.tick >= self.tick_until {
            self.running = false;
            self.tick = self.tick_until;
        }

        self.running
    }

    fn tick_queued(&mut self) {
        self.queued_clients = self
            .queued_clients
            .iter_mut()
            .filter(|c| {
                let mut client = c.borrow_mut();
                client.tick_wait(self.tick)
            })
            .map(|c| c.clone())
            .collect();
    }
}
