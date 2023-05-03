use super::Attribute;
use core::time::Duration;
use std::cmp::Ordering;
use std::sync::{atomic, atomic::AtomicUsize};

mod queue;

pub use queue::Queue;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Server {
    pub id: usize,
    pub attributes: Vec<Attribute>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, atomic::Ordering::SeqCst),
            attributes: vec![],
        }
    }
}

impl Server {
    #[must_use]
    pub fn id(&self) -> usize {
        self.id
    }

    #[must_use]
    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Eq, PartialEq)]
pub struct QueueableServer {
    server: Server,
    pub tick: Duration,
}

impl Ord for QueueableServer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tick.cmp(&other.tick)
    }
}

impl PartialOrd for QueueableServer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl QueueableServer {
    #[must_use]
    pub fn new(server: Server) -> Self {
        Self {
            server,
            tick: Duration::ZERO,
        }
    }

    #[must_use]
    pub fn server(&self) -> &Server {
        &self.server
    }
}
