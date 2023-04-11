mod queue;

pub use queue::Queue;

use super::Attribute;
use std::sync::{atomic, atomic::AtomicUsize};

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

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }
}
