use super::Attribute;
use std::cmp::Ordering;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Server {
    attributes: Vec<Attribute>,
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
