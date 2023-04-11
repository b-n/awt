mod attribute;
mod client_profile;
mod request;
mod routing;
mod server;

pub use attribute::Attribute;
pub use client_profile::ClientProfile;
pub use request::{Queue as RequestQueue, Request, Status as RequestStatus};
pub use server::{Queue as ServerQueue, QueueableServer, Server};

use routing::route_requests;

pub use core::fmt::Debug;
use rand::{Rng, RngCore};
use std::collections::HashMap;
use std::sync::Arc;

pub const TICKS_PER_SECOND: usize = 1000;
pub const ONE_HOUR: usize = TICKS_PER_SECOND * 60 * 60;

pub struct Simulation {
    tick: usize,
    tick_size: usize,
    tick_until: usize,
    running: bool,
    client_profiles: Vec<Arc<ClientProfile>>,
    request_queue: RequestQueue,
    server_queue: ServerQueue,
    rng: Box<dyn RngCore>,
}

impl Simulation {
    pub fn new(rng: Box<dyn RngCore>) -> Self {
        Self {
            tick: 0,
            tick_size: 1,
            tick_until: ONE_HOUR,
            running: false,
            client_profiles: vec![],
            request_queue: RequestQueue::new(),
            server_queue: ServerQueue::new(),
            rng,
        }
    }
}

impl Debug for Simulation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Simulation Tick: {}", self.tick)?;
        for (k, v) in self.statistics() {
            writeln!(f, "{:11} {v:>4}", format!("{k:?}"))?;
        }
        Ok(())
    }
}

// Structure and setup
impl Simulation {
    pub fn add_server(&mut self, server: Arc<Server>) {
        assert!(
            !self.running,
            "Servers can only be added whilst the simulation is stopped"
        );

        self.server_queue.push(QueueableServer::new(server));
    }

    pub fn add_client_profile(&mut self, client_profile: Arc<ClientProfile>) {
        assert!(
            !self.running,
            "Client Profiles can only be added whilst the simulation is stopped"
        );

        self.client_profiles.push(client_profile);
    }

    /// Enables the `Simulation`, generating all the internal state required for running. A
    /// `Simulation` can then be advanced by calling the `tick()` function until it returns
    /// `false`.
    pub fn enable(&mut self) {
        assert!(!self.running, "Cannot enable an already enabled simulation");

        self.running = true;
        self.tick_size = 1;
        self.generate_requests();

        self.server_queue.init();
        self.request_queue.init();
    }

    /// Returns a tuple which indicates of the whether the `Simulation` is still running along with
    /// it's current tick.
    #[allow(dead_code)]
    pub fn running(&self) -> (bool, usize) {
        (self.running, self.tick)
    }

    /// Returns a `HashMap` which contains the status of a `Request`, and the number of `Request`s
    /// which meet that state.
    pub fn statistics(&self) -> HashMap<RequestStatus, usize> {
        self.request_queue
            .requests()
            .iter()
            .fold(HashMap::new(), |mut acc, c| {
                let i = acc.entry(*c.borrow().status()).or_insert(0);
                *i += 1;
                acc
            })
    }
}

// Generators and state modifiers
impl Simulation {
    fn generate_requests(&mut self) {
        let mut request_from_client_profile = |cp: &Arc<ClientProfile>| -> Request {
            let start = self.rng.gen_range(0..=self.tick_until);
            let abandon_ticks = cp.abandon_time;
            let handle_ticks = cp.handle_time;

            Request::new(
                start,
                abandon_ticks,
                handle_ticks,
                cp.required_attributes.clone(),
                cp.clone(),
            )
        };

        for cp in &self.client_profiles {
            for _ in 0..cp.quantity {
                self.request_queue.push(request_from_client_profile(cp));
            }
        }
    }
}

// Simulation logic
impl Simulation {
    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }

        // release requests and servers from queues
        self.request_queue.tick(self.tick);
        self.server_queue.tick(self.tick);

        // assign the relevant servers
        self.do_routing();

        // tick the main simulation
        self.increment_tick();

        self.running
    }

    /// Routing is fairly straight forward to orchestrate.
    ///
    /// 1. Each request in the queue (gauranteed to be in order of ticks) is passed to the router
    ///    along with a list of available servers.
    /// 2. If the router returns a server, then that server is assigned to the request
    /// 3. The server is pulled into a buffer for the expected number of ticks, and removed from
    ///    the pool
    fn do_routing(&mut self) {
        let request_data = self.request_queue.routing_data();
        let server_data = self.server_queue.routing_data();

        // Only attempt to route if there is actually something that could be done
        if request_data.is_empty() || server_data.is_empty() {
            return;
        }

        for (request_id, server_id) in route_requests(request_data, server_data) {
            let release_tick = self.request_queue.handle_request(request_id, self.tick);

            self.server_queue.enqueue(server_id, release_tick);
        }
    }

    fn increment_tick(&mut self) -> bool {
        // In order to allow custom routing options, we need to always tick with `tick_size` if
        // there are requests waiting for servers. If there are no requests waiting, then we can
        // directly advance the tick to the next request in the `request_buffer`, or the `server` in
        // the `server_buffer`.
        self.tick = if self.request_queue.has_waiting() {
            self.tick + self.tick_size
        } else {
            let request_buffer_head = self.request_queue.next_tick();
            let server_buffer_head = self.server_queue.next_tick();

            match (request_buffer_head, server_buffer_head) {
                (Some(t), Some(u)) if t <= u => t,
                (Some(_) | None, Some(t)) | (Some(t), None) => t,
                (None, None) => self.tick_until,
            }
        };

        if self.tick >= self.tick_until {
            self.running = false;
            self.tick = self.tick_until;
        }

        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::rngs::mock::StepRng;

    fn mock_rng() -> Box<dyn RngCore> {
        // Set the step size to be compatible with `gen_range`. `gen_range` restricts the domain by
        // giving multiple rolls per element in the range sequentially.
        let step = u64::MAX / ONE_HOUR as u64;

        Box::new(StepRng::new(1, step))
    }

    #[test]
    fn empty_sim() {
        let mut sim = Simulation::new(mock_rng());

        sim.enable();

        while sim.tick() {}

        let stats = sim.statistics();
        assert_eq!(None, stats.get(&RequestStatus::Pending));
        assert_eq!(None, stats.get(&RequestStatus::Enqueued));
        assert_eq!(None, stats.get(&RequestStatus::Answered));
        assert_eq!(None, stats.get(&RequestStatus::Abandoned));
        assert_eq!((false, ONE_HOUR), sim.running());
    }

    #[test]
    fn no_servers() {
        let mut sim = Simulation::new(mock_rng());

        let client_profile = Arc::new(ClientProfile::default());
        sim.add_client_profile(client_profile);

        sim.enable();

        while sim.tick() {}

        let stats = sim.statistics();
        assert_eq!(None, stats.get(&RequestStatus::Pending));
        assert_eq!(None, stats.get(&RequestStatus::Enqueued));
        assert_eq!(None, stats.get(&RequestStatus::Answered));
        assert_eq!(Some(&1), stats.get(&RequestStatus::Abandoned));
        assert_eq!((false, ONE_HOUR), sim.running());
    }

    #[test]
    fn can_handle_requests() {
        let mut sim = Simulation::new(mock_rng());

        let client_profile = Arc::new(ClientProfile::default());
        sim.add_client_profile(client_profile);

        let server = Arc::new(Server::default());
        sim.add_server(server);

        sim.enable();

        while sim.tick() {}

        let stats = sim.statistics();
        assert_eq!(None, stats.get(&RequestStatus::Pending));
        assert_eq!(None, stats.get(&RequestStatus::Enqueued));
        assert_eq!(Some(&1), stats.get(&RequestStatus::Answered));
        assert_eq!(None, stats.get(&RequestStatus::Abandoned));
        assert_eq!((false, ONE_HOUR), sim.running());
    }

    #[test]
    fn requests_can_abandon() {
        let mut sim = Simulation::new(mock_rng());

        // Ensure two requests are provided in a way that the second cannot be handled in time
        let client_profile = Arc::new(ClientProfile {
            handle_time: TICKS_PER_SECOND * 300,
            ..ClientProfile::default()
        });
        sim.add_client_profile(client_profile.clone());
        sim.add_client_profile(client_profile);

        let server = Arc::new(Server::default());
        sim.add_server(server);

        sim.enable();

        while sim.tick() {}

        let stats = sim.statistics();
        assert_eq!(None, stats.get(&RequestStatus::Pending));
        assert_eq!(None, stats.get(&RequestStatus::Enqueued));
        assert_eq!(Some(&1), stats.get(&RequestStatus::Answered));
        assert_eq!(Some(&1), stats.get(&RequestStatus::Abandoned));
        assert_eq!((false, ONE_HOUR), sim.running());
    }

    #[test]
    #[should_panic]
    fn cannot_add_profiles_whilst_running() {
        let mut sim = Simulation::new(mock_rng());
        sim.enable();

        // Cannot add profile to running sim
        let client_profile = Arc::new(ClientProfile::default());
        sim.add_client_profile(client_profile);
    }

    #[test]
    #[should_panic]
    fn cannot_enable_twice() {
        let mut sim = Simulation::new(mock_rng());
        sim.enable();

        // Cannot enable twice
        sim.enable();
    }

    #[test]
    #[should_panic]
    fn cannot_add_server_whilst_running() {
        let mut sim = Simulation::new(mock_rng());
        sim.enable();

        let server = Arc::new(Server::default());
        sim.add_server(server);
    }
}
