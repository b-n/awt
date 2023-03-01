mod attribute;
mod client;
mod client_profile;
mod routing;
mod server;

pub use attribute::Attribute;
pub use client_profile::ClientProfile;
pub use server::{EnqueuedServer, Server};

use client::Client;
use routing::{route_clients, ClientRoutingData};

pub use core::fmt::Debug;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;
use std::sync::Arc;

pub const TICKS_PER_SECOND: usize = 1000;
pub const ONE_HOUR: usize = TICKS_PER_SECOND * 60 * 60;

#[derive(Clone)]
pub struct Simulation {
    tick: usize,
    tick_size: usize,
    tick_until: usize,
    running: bool,
    client_profiles: Vec<Arc<ClientProfile>>,
    clients: Vec<Rc<RefCell<Client>>>,
    client_buffer: BinaryHeap<Reverse<Rc<RefCell<Client>>>>,
    client_queue: HashMap<usize, Rc<RefCell<Client>>>,
    servers: Vec<Arc<Server>>,
    server_buffer: BinaryHeap<Reverse<EnqueuedServer>>,
    server_queue: HashMap<usize, Arc<Server>>,
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
            clients: vec![],
            client_buffer: BinaryHeap::new(),
            client_queue: HashMap::new(),
            servers: vec![],
            server_buffer: BinaryHeap::new(),
            server_queue: HashMap::new(),
            rng: thread_rng(),
        }
    }
}

impl Debug for Simulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unanswered = self
            .clients
            .iter()
            .filter(|c| c.borrow().is_unanswered())
            .count();
        let answered = self
            .clients
            .iter()
            .filter(|c| c.borrow().is_answered())
            .count();
        let abandonend = self
            .clients
            .iter()
            .filter(|c| c.borrow().is_abandoned())
            .count();

        writeln!(
            f,
            "Simulation Tick: {}
Unanswered {:>4}
Answered   {:>4}
Abandoned  {:>4}",
            self.tick, unanswered, answered, abandonend
        )
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

                    clients.push(Rc::new(RefCell::new(client)));
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
        self.server_queue = self.servers.iter().map(|s| (s.id(), s.clone())).collect();
    }
}

// Simulation logic
impl Simulation {
    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }

        // release clients and servers
        self.enqueue_clients();
        self.enqueue_servers();

        // assign the relevant servers
        self.do_routing();

        // tick the main simulation
        self.increment_tick();
        // tick all the queued clients
        self.tick_queued();

        self.running
    }

    /// Find which clients haven't been added to the queue yet.
    fn enqueue_clients(&mut self) {
        while self
            .client_buffer
            .peek()
            .map_or(self.tick_until, |c| c.0.borrow().start())
            <= self.tick
        {
            let next_client = self
                .client_buffer
                .pop()
                .expect("Client was peeked and should have been popped")
                .0;

            let mut client = next_client.borrow_mut();
            client.enqueue(self.tick);

            self.client_queue.insert(client.id(), next_client.clone());
        }
    }

    fn enqueue_servers(&mut self) {
        while self
            .server_buffer
            .peek()
            .map_or(self.tick_until, |s| s.0.tick)
            <= self.tick
        {
            let next_server = self
                .server_buffer
                .pop()
                .expect("Server was peeked and should have popped")
                .0;

            self.server_queue
                .insert(next_server.server.id(), next_server.server);
        }
    }

    /// Routing is fairly straight forward to orchestrate.
    ///
    /// 1. Each client in the queue (gauranteed to be in order of ticks) is passed to the router
    ///    along with a list of available servers.
    /// 2. If the router returns a server, then that server is assigned to the client
    /// 3. The server is pulled into a buffer for the expected number of ticks, and removed from
    ///    the pool
    fn do_routing(&mut self) {
        // TODO: Cache this data, it doesn't change cycle to cycle
        let client_data: Vec<ClientRoutingData> = self
            .client_queue
            .values()
            .map(ClientRoutingData::from)
            .collect();

        for (client_id, server_id) in
            route_clients(&client_data, self.server_queue.values().collect())
        {
            // TODO: handle client not in queue
            let mut client = self
                .client_queue
                .get(&client_id)
                .expect("Client Id does not exist")
                .borrow_mut();

            let release_tick = client.handle(self.tick, 300 * TICKS_PER_SECOND);

            // TODO: Safely chceck that the server_queue has this server_id
            let server = self
                .server_queue
                .remove(&server_id)
                .expect("Server Id should have been queued");

            self.server_buffer
                .push(Reverse(EnqueuedServer::new(server, release_tick)));
        }
    }

    fn increment_tick(&mut self) -> bool {
        // In order to allow custom routing options, we need to always tick with `tick_size` if
        // there are clients waiting for servers. If there are no clients waiting, then we can
        // directly advance the tick to the next client in the `client_buffer`, or the `server` in
        // the `server_buffer`.
        self.tick = if self.client_queue.is_empty() {
            let client_buffer_head = self.client_buffer.peek().map(|c| c.0.borrow().start());
            let server_buffer_head = self.server_buffer.peek().map(|c| c.0.tick);

            match (client_buffer_head, server_buffer_head) {
                (Some(t), Some(u)) if t >= u => t,
                (Some(_) | None, Some(t)) | (Some(t), None) => t,
                (None, None) => self.tick_until,
            }
        } else {
            self.tick + self.tick_size
        };

        if self.tick >= self.tick_until {
            self.running = false;
            self.tick = self.tick_until;
        }

        self.running
    }

    fn tick_queued(&mut self) {
        self.client_queue.retain(|_, client| {
            let mut client = client.borrow_mut();
            client.tick_wait(self.tick);
            client.is_unanswered()
        });
    }
}
