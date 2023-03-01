use super::Attribute;
use std::cmp::Ordering;
use std::sync::{atomic, atomic::AtomicUsize, Arc};

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Server {
    id: usize,
    attributes: Vec<Attribute>,
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
    pub fn id(&self) -> usize {
        self.id
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct EnqueuedServer {
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
