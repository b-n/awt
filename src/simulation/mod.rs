mod attribute;
mod client_profile;
mod request;
mod routing;
mod server;

pub use attribute::Attribute;
pub use client_profile::ClientProfile;
pub use server::{EnqueuedServer, Server};

use request::{Request, Status as RequestStatus};
use routing::{route_requests, RequestRoutingData};

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
    requests: Vec<Rc<RefCell<Request>>>,
    request_buffer: BinaryHeap<Reverse<Rc<RefCell<Request>>>>,
    request_queue: HashMap<usize, Rc<RefCell<Request>>>,
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
            requests: vec![],
            request_buffer: BinaryHeap::new(),
            request_queue: HashMap::new(),
            servers: vec![],
            server_buffer: BinaryHeap::new(),
            server_queue: HashMap::new(),
            rng: thread_rng(),
        }
    }
}

impl Debug for Simulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.requests.iter().fold(HashMap::new(), |mut acc, c| {
            let i = acc.entry(*c.borrow().status()).or_insert(0);
            *i += 1;
            acc
        });

        writeln!(f, "Simulation Tick: {}", self.tick)?;
        let _ = stats.iter().map(|(k, v)| writeln!(f, "{k:?} {v:>4}"));
        Ok(())
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
        self.generate_requests();
        self.generate_request_buffer();

        self.set_servers_available();
    }

    fn generate_requests(&mut self) {
        self.requests = self
            .client_profiles
            .iter()
            .flat_map(|cp| {
                let mut requests = vec![];
                for _ in 0..cp.quantity {
                    let mut request = Request::from(cp);

                    let start = self.rng.gen_range(0..=self.tick_until);
                    request.set_start(start);

                    requests.push(Rc::new(RefCell::new(request)));
                }
                requests
            })
            .collect();
    }

    fn generate_request_buffer(&mut self) {
        for request in &self.requests {
            self.request_buffer.push(Reverse(request.clone()));
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

        // release requests and servers
        self.enqueue_requests();
        self.enqueue_servers();

        // assign the relevant servers
        self.do_routing();

        // tick the main simulation
        self.increment_tick();
        // tick all the queued requests
        self.tick_queued();

        self.running
    }

    /// Find which requests haven't been added to the queue yet.
    fn enqueue_requests(&mut self) {
        while self
            .request_buffer
            .peek()
            .map_or(self.tick_until, |c| c.0.borrow().start())
            <= self.tick
        {
            let next_request = self
                .request_buffer
                .pop()
                .expect("Request was peeked and should have been popped")
                .0;

            let mut request = next_request.borrow_mut();
            request.enqueue(self.tick);

            self.request_queue
                .insert(request.id(), next_request.clone());
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
    /// 1. Each request in the queue (gauranteed to be in order of ticks) is passed to the router
    ///    along with a list of available servers.
    /// 2. If the router returns a server, then that server is assigned to the request
    /// 3. The server is pulled into a buffer for the expected number of ticks, and removed from
    ///    the pool
    fn do_routing(&mut self) {
        // TODO: Cache this data, it doesn't change cycle to cycle
        let request_data: Vec<RequestRoutingData> = self
            .request_queue
            .values()
            .map(RequestRoutingData::from)
            .collect();

        for (request_id, server_id) in
            route_requests(&request_data, self.server_queue.values().collect())
        {
            // TODO: handle request not in queue
            let mut request = self
                .request_queue
                .get(&request_id)
                .expect("Client Id does not exist")
                .borrow_mut();

            let release_tick = request.handle(self.tick, 300 * TICKS_PER_SECOND);

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
        // there are requests waiting for servers. If there are no requests waiting, then we can
        // directly advance the tick to the next request in the `request_buffer`, or the `server` in
        // the `server_buffer`.
        self.tick = if self.request_queue.is_empty() {
            let request_buffer_head = self.request_buffer.peek().map(|c| c.0.borrow().start());
            let server_buffer_head = self.server_buffer.peek().map(|c| c.0.tick);

            match (request_buffer_head, server_buffer_head) {
                (Some(t), Some(u)) if t <= u => t,
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
        self.request_queue.retain(|_, request| {
            let mut request = request.borrow_mut();
            request.tick_wait(self.tick);
            &RequestStatus::Enqueued == request.status()
        });
    }
}
