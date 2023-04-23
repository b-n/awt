mod client;
mod request;
mod routing;
mod server;
mod statistics;

use rand::{Rng, RngCore};

pub use client::Client;
pub use request::{Queue as RequestQueue, Request, Status as RequestStatus};
pub use server::{Queue as ServerQueue, QueueableServer, Server};
pub use statistics::Statistics;

use crate::{Attribute, ClientProfile, Metric};
use routing::route_requests;

pub const TICKS_PER_SECOND: usize = 1000;
pub const ONE_HOUR: usize = TICKS_PER_SECOND * 60 * 60;

pub struct Simulation {
    tick: usize,
    tick_size: usize,
    tick_until: usize,
    running: bool,
    clients: Vec<Client>,
    request_queue: RequestQueue,
    server_queue: ServerQueue,
    statistics: Statistics,
    rng: Box<dyn RngCore>,
}

impl Simulation {
    pub fn new(rng: Box<dyn RngCore>) -> Self {
        Self {
            tick: 0,
            tick_size: 1,
            tick_until: ONE_HOUR,
            running: false,
            clients: vec![],
            request_queue: RequestQueue::new(),
            server_queue: ServerQueue::new(),
            statistics: Statistics::default(),
            rng,
        }
    }
}

// Structure and setup
impl Simulation {
    pub fn add_server(&mut self, server: Server) {
        assert!(
            !self.running,
            "Servers can only be added whilst the simulation is stopped"
        );

        self.server_queue.push(QueueableServer::new(server));
    }

    pub fn add_client(&mut self, client: Client) {
        assert!(
            !self.running,
            "Client Profiles can only be added whilst the simulation is stopped"
        );

        self.clients.push(client);
    }

    pub fn add_metric(&mut self, metric: Metric) {
        assert!(
            !self.running,
            "Cannot add metric whilst simulation is in progress"
        );

        self.statistics.push(metric);
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

    /// Returns the `Statistics` object for this simulation.
    pub fn statistics(&mut self) -> &Statistics {
        assert!(
            !self.running,
            "Cannot get statistics for inprogress simulation"
        );

        let requests = self.request_queue.requests();
        self.statistics.calculate(requests);
        &self.statistics
    }
}

// Generators and state modifiers
impl Simulation {
    fn generate_requests(&mut self) {
        let mut request_from_client = |c: &Client| -> Request {
            let start = self.rng.gen_range(0..=self.tick_until);
            let abandon_ticks = start + c.abandon_time;
            let handle_ticks = c.handle_time;

            Request::new(
                start,
                abandon_ticks,
                handle_ticks,
                c.required_attributes.clone(),
                c,
            )
        };

        for client in &self.clients {
            self.request_queue.push(request_from_client(client));
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
    use crate::MetricType;

    use rand::rngs::mock::StepRng;

    fn mock_rng() -> Box<dyn RngCore> {
        // Set the step size to be compatible with `gen_range`. `gen_range` restricts the domain by
        // giving multiple rolls per element in the range sequentially.
        let step = u64::MAX / ONE_HOUR as u64;

        Box::new(StepRng::new(1, step))
    }

    fn simulation() -> Simulation {
        let mut sim = Simulation::new(mock_rng());

        sim.add_metric(Metric::with_target_f64(MetricType::AbandonRate, 0.0).unwrap());
        sim.add_metric(Metric::with_target_usize(MetricType::AnswerCount, 0).unwrap());

        sim
    }

    #[test]
    fn empty_sim() {
        let mut sim = simulation();

        sim.enable();

        while sim.tick() {}

        let stats = sim.statistics();
        assert_eq!(
            None,
            stats.get(&MetricType::AbandonRate).and_then(Metric::value)
        );
        assert_eq!(
            Some(0.0),
            stats.get(&MetricType::AnswerCount).and_then(Metric::value)
        );
        assert_eq!((false, ONE_HOUR), sim.running());
    }

    #[test]
    fn no_servers() {
        let mut sim = simulation();

        let client = Client::default();
        sim.add_client(client);

        sim.enable();

        while sim.tick() {}

        let stats = sim.statistics();
        assert_eq!(
            Some(1.0),
            stats.get(&MetricType::AbandonRate).and_then(Metric::value)
        );
        assert_eq!(
            Some(0.0),
            stats.get(&MetricType::AnswerCount).and_then(Metric::value)
        );
        assert_eq!((false, ONE_HOUR), sim.running());
    }

    #[test]
    fn can_handle_requests() {
        let mut sim = simulation();

        let client = Client::default();
        sim.add_client(client);

        let server = Server::default();
        sim.add_server(server);

        sim.enable();

        while sim.tick() {}

        let stats = sim.statistics();
        assert_eq!(
            Some(0.0),
            stats.get(&MetricType::AbandonRate).and_then(Metric::value)
        );
        assert_eq!(
            Some(1.0),
            stats.get(&MetricType::AnswerCount).and_then(Metric::value)
        );
        assert_eq!((false, ONE_HOUR), sim.running());
    }

    #[test]
    fn requests_can_abandon() {
        let mut sim = simulation();

        // Ensure two requests are provided in a way that the second cannot be handled in time
        let client = Client {
            handle_time: TICKS_PER_SECOND * 300,
            ..Client::default()
        };
        sim.add_client(client.clone());
        sim.add_client(client);

        let server = Server::default();
        sim.add_server(server);

        sim.enable();

        while sim.tick() {}

        let stats = sim.statistics();
        assert_eq!(
            Some(0.5),
            stats.get(&MetricType::AbandonRate).and_then(Metric::value)
        );
        assert_eq!(
            Some(1.0),
            stats.get(&MetricType::AnswerCount).and_then(Metric::value)
        );

        assert_eq!((false, ONE_HOUR), sim.running());
    }

    #[test]
    #[should_panic]
    fn cannot_add_profiles_whilst_running() {
        let mut sim = Simulation::new(mock_rng());
        sim.enable();

        // Cannot add profile to running sim
        let client = Client::default();
        sim.add_client(client);
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

        let server = Server::default();
        sim.add_server(server);
    }
}
