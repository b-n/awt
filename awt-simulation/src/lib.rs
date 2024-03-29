#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(unknown_lints)]
#![warn(missing_debug_implementation)]
#![warn(missing_copy_implementation)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_difference)]

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

extern crate alloc;

pub mod attribute;
pub mod client;
pub mod error;
pub mod request;
pub mod server;

mod config;
mod routing;

use core::time::Duration;
use rand::{Rng, RngCore};

use alloc::{boxed::Box, vec::Vec};
use attribute::Attribute;
use client::Client;
use error::Error;
use request::{queue::Queue as RequestQueue, Data as RequestData, Request};
use routing::route_requests;
use server::{queue::Queue as ServerQueue, QueueableServer, Server};

pub use config::Config;

pub type Result<T> = core::result::Result<T, Error>;

pub struct Simulation {
    start: Duration,
    tick: Duration,
    tick_size: Duration,
    end: Duration,
    running: bool,
    clients: Vec<Client>,
    request_queue: RequestQueue,
    server_queue: ServerQueue,
    rng: Box<dyn RngCore>,
}

impl Simulation {
    /// Generate a new Simulation
    #[must_use]
    pub fn new(end: Duration, tick_size: Duration, rng: Box<dyn RngCore>) -> Self {
        Self {
            start: Duration::ZERO,
            tick: Duration::ZERO,
            tick_size,
            end,
            running: false,
            clients: Vec::new(),
            request_queue: RequestQueue::default(),
            server_queue: ServerQueue::default(),
            rng,
        }
    }
}

// Structure and setup
impl Simulation {
    /// Add a `Server` to the `Simulation`
    ///
    /// # Errors
    ///
    /// Will error when `Simulation` is already enabled.
    pub fn add_server(&mut self, server: Server) -> Result<()> {
        if self.running {
            return Err(Error::Enabled);
        }
        self.server_queue.push(QueueableServer::new(server));
        Ok(())
    }

    fn add_servers(&mut self, servers: Vec<Server>) {
        for server in servers {
            self.server_queue.push(QueueableServer::new(server));
        }
    }

    /// Add a `Client` to the `Simulation`
    ///
    /// # Errors
    ///
    /// Will error when `Simulation` is already enabled.
    pub fn add_client(&mut self, client: Client) -> Result<()> {
        if self.running {
            return Err(Error::Enabled);
        }

        self.clients.push(client);

        Ok(())
    }

    fn add_clients(&mut self, clients: &mut Vec<Client>) {
        self.clients.append(clients);
    }

    /// Enables the `Simulation` which will generate and schedule all simulation elements. The `Simulation` can then be
    /// advanced by calling the `tick()` until it returns false.
    ///
    /// # Errors
    ///
    /// Will error if already enabled.
    pub fn enable(&mut self) -> Result<bool> {
        if self.running {
            return Err(Error::Enabled);
        }

        self.running = true;
        self.generate_requests();

        self.server_queue.init();
        self.request_queue.init();
        Ok(self.running)
    }

    /// Returns a tuple which indicates of the whether the `Simulation` is still running along with
    /// it's current tick.
    #[must_use]
    pub fn running(&self) -> (bool, Duration) {
        (self.running, self.tick)
    }

    #[must_use]
    pub fn request_data(&self) -> Vec<RequestData> {
        self.request_queue
            .requests()
            .iter()
            .map(|request| request.borrow().data())
            .collect()
    }
}

// Generators and state modifiers
impl Simulation {
    fn generate_requests(&mut self) {
        let mut request_from_client = |c: &Client| -> Request {
            let start = self.rng.gen_range(self.start..=self.end);
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
            // TODO: It's possible that the request_id or server_id are not available in the queue,
            // which could lead to panics. This should "fail safely".
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
                (None, None) => self.end,
            }
        };

        if self.tick >= self.end {
            self.running = false;
            self.tick = self.end;
        }

        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::request::Status;
    use alloc::rc::Rc;
    use core::cell::RefCell;
    use rand::rngs::mock::StepRng;
    use std::collections::HashMap;

    const TICK_SIZE: Duration = Duration::new(0, 50_000_000);
    const ONE_HOUR: Duration = Duration::new(60 * 60, 0);

    fn request_stats(requests: &[Rc<RefCell<Request>>]) -> HashMap<Status, usize> {
        requests.iter().fold(HashMap::new(), |mut acc, r| {
            let status = *r.borrow().status();
            if let Some(v) = acc.get_mut(&status) {
                *v += 1;
            } else {
                acc.insert(status, 1);
            }
            acc
        })
    }

    fn mock_rng() -> Box<dyn RngCore> {
        // Set the step size to be compatible with `gen_range`. `gen_range` restricts the domain by
        // giving multiple rolls per element in the range sequentially.
        let step = u64::MAX / 3600;
        Box::new(StepRng::new(1, step))
    }

    fn simulation() -> Simulation {
        Simulation::new(ONE_HOUR, TICK_SIZE, mock_rng())
    }

    #[test]
    fn empty_sim() -> Result<()> {
        let mut sim = simulation();

        sim.enable()?;

        while sim.tick() {}

        let stats = request_stats(sim.request_queue.requests());
        assert_eq!(None, stats.get(&Status::Answered));
        assert_eq!(None, stats.get(&Status::Abandoned));
        Ok(assert_eq!((false, ONE_HOUR), sim.running()))
    }

    #[test]
    fn no_servers() -> Result<()> {
        let mut sim = simulation();

        let client = Client::default();
        sim.add_client(client)?;

        sim.enable()?;

        while sim.tick() {}

        let stats = request_stats(sim.request_queue.requests());
        assert_eq!(None, stats.get(&Status::Answered));
        assert_eq!(Some(&1), stats.get(&Status::Abandoned));
        Ok(assert_eq!((false, ONE_HOUR), sim.running()))
    }

    #[test]
    fn can_handle_requests() -> Result<()> {
        let mut sim = simulation();

        let client = Client::default();
        sim.add_client(client)?;

        let server = Server::default();
        sim.add_server(server)?;

        sim.enable()?;

        while sim.tick() {}

        let stats = request_stats(sim.request_queue.requests());
        assert_eq!(Some(&1), stats.get(&Status::Answered));
        assert_eq!(None, stats.get(&Status::Abandoned));
        Ok(assert_eq!((false, ONE_HOUR), sim.running()))
    }

    #[test]
    fn requests_can_abandon() -> Result<()> {
        let mut sim = simulation();

        // Ensure two requests are provided in a way that the second cannot be handled in time
        let client = Client {
            handle_time: Duration::new(300, 0),
            ..Client::default()
        };
        sim.add_client(client.clone())?;
        sim.add_client(client)?;

        let server = Server::default();
        sim.add_server(server)?;

        sim.enable()?;

        while sim.tick() {}

        let stats = request_stats(sim.request_queue.requests());
        assert_eq!(Some(&1), stats.get(&Status::Answered));
        assert_eq!(Some(&1), stats.get(&Status::Abandoned));
        Ok(assert_eq!((false, ONE_HOUR), sim.running()))
    }

    #[test]
    fn cannot_add_profiles_whilst_running() -> Result<()> {
        let mut sim = Simulation::new(ONE_HOUR, TICK_SIZE, mock_rng());
        sim.enable()?;

        // Cannot add profile to running sim
        let client = Client::default();
        Ok(assert!(sim.add_client(client).is_err()))
    }

    #[test]
    fn cannot_enable_twice() -> Result<()> {
        let mut sim = Simulation::new(ONE_HOUR, TICK_SIZE, mock_rng());
        sim.enable()?;

        // Cannot enable twice
        Ok(assert!(sim.enable().is_err()))
    }

    #[test]
    fn cannot_add_server_whilst_running() -> Result<()> {
        let mut sim = Simulation::new(ONE_HOUR, TICK_SIZE, mock_rng());
        sim.enable()?;

        let server = Server::default();
        Ok(assert!(sim.add_server(server).is_err()))
    }
}
