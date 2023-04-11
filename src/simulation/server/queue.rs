use super::Server;
use crate::simulation::routing::ServerData;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use crate::MinQueue;

pub struct Queue {
    inner: Vec<Arc<Server>>,
    enqueued: MinQueue<EnqueuedServer>,
    waiting: HashMap<usize, (Arc<Server>, ServerData)>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            enqueued: MinQueue::new(),
            waiting: HashMap::new(),
        }
    }

    pub fn push(&mut self, server: Arc<Server>) {
        self.inner.push(server);
    }

    pub fn set_all_available(&mut self) {
        for server in &self.inner {
            let routing_data = ServerData::from(server);
            self.waiting
                .insert(server.id(), (server.clone(), routing_data));
        }
    }

    pub fn release_servers(&mut self, tick: usize) {
        while self.enqueued.peek().map_or(usize::MAX, |s| s.tick) <= tick {
            let next_server = self
                .enqueued
                .pop()
                .expect("Server was peeked and should have popped")
                .server;

            let routing_data = ServerData::from(&next_server);

            self.waiting
                .insert(next_server.id(), (next_server.clone(), routing_data));
        }
    }

    pub fn enqueued_head(&self) -> Option<usize> {
        self.enqueued.peek().map(|c| c.tick)
    }

    pub fn routing_data(&self) -> Vec<&ServerData> {
        self.waiting.values().map(|(_, s)| s).collect()
    }

    pub fn assign_server(&mut self, server_id: usize, until: usize) {
        // TODO: Safely chceck that the server_queue has this server_id
        let server = self
            .waiting
            .remove(&server_id)
            .expect("Server Id should have been queued")
            .0;

        self.enqueued.push(EnqueuedServer::new(server, until));
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Clone, Eq, PartialEq)]
struct EnqueuedServer {
    pub server: Arc<Server>,
    pub tick: usize,
}

impl Ord for EnqueuedServer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tick.cmp(&other.tick)
    }
}

impl PartialOrd for EnqueuedServer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl EnqueuedServer {
    pub fn new(server: Arc<Server>, tick: usize) -> Self {
        Self { server, tick }
    }
}
