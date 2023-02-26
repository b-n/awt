mod attribute;
mod client;
mod client_profile;
mod server;

pub use attribute::Attribute;
pub use client_profile::ClientProfile;
pub use server::Server;

use client::Client;

use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
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
    clients: Vec<RefCell<Client>>,
    client_buffer: BinaryHeap<Reverse<RefCell<Client>>>,
    client_queue: BinaryHeap<Reverse<RefCell<Client>>>,
    available_servers: Vec<Arc<Server>>,
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
            client_buffer: BinaryHeap::new(),
            client_queue: BinaryHeap::new(),
            available_servers: vec![],
            rng: thread_rng(),
        }
    }
}

// Structure and setup
impl Simulation {
    pub fn add_server(&mut self, server: Arc<Server>) {
        self.servers.push(server);
    }

    pub fn add_client_profile(&mut self, client_profile: &Arc<ClientProfile>) {
        self.client_profiles.push(client_profile.clone());
    }

    pub fn enable(&mut self) {
        self.running = true;
        self.tick_size = 1;
        self.generate_clients();
        self.generate_client_buffer();

        self.set_servers_available();
    }

    fn generate_clients(&mut self) {
        self.clients = self
            .client_profiles
            .iter()
            .enumerate()
            .flat_map(|(i, cp)| {
                let mut clients = vec![];
                for _ in 0..cp.quantity {
                    let mut client = Client::from(cp);
                    client.set_id(i);

                    let start = self.rng.gen_range(0..=self.tick_until);
                    client.set_start(start);

                    clients.push(RefCell::new(client));
                }
                clients
            })
            .collect();
    }

    fn generate_client_buffer(&mut self) {
        for client in &self.clients {
            self.client_buffer.push(Reverse(client.clone()));
        }
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

        self.enqueue_clients();

        // self.check_routing();

        self.increment_tick();

        self.tick_queued();

        self.running
    }

    /// Find which clients haven't been added to the queue yet.
    fn enqueue_clients(&mut self) {
        while let Some(client) = self.client_buffer.peek() && (client.0.borrow()).start() <= self.tick {
            let next_client = self.client_buffer.pop().expect("Client should have been popped");
            
            next_client.0.borrow_mut().enqueue(self.tick);

            self.client_queue.push(next_client);
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
        for client in &self.client_queue {
            let mut client = client.0.borrow_mut();
            client.tick_wait(self.tick);
        }

        self.client_queue
            .retain(|client| client.0.borrow().is_waiting());
    }
}
