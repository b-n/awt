use binary_heap_plus::{BinaryHeap, MinComparator};
use core::time::Duration;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::QueueableServer;
use crate::routing::ServerData;

pub struct Queue {
    inner: Vec<Rc<RefCell<QueueableServer>>>,
    enqueued: BinaryHeap<Rc<RefCell<QueueableServer>>, MinComparator>,
    waiting: HashMap<usize, (Rc<RefCell<QueueableServer>>, ServerData)>,
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            inner: vec![],
            enqueued: BinaryHeap::new_min(),
            waiting: HashMap::new(),
        }
    }
}

// Setup and cretion logic
impl Queue {
    pub fn push(&mut self, server: QueueableServer) {
        let server = Rc::new(RefCell::new(server));

        self.inner.push(server);
    }

    pub fn init(&mut self) {
        for server in &self.inner {
            let routing_data = ServerData::from(server.borrow().server());
            self.waiting.insert(
                server.borrow().server().id(),
                (server.clone(), routing_data),
            );
        }
    }
}

// Logic relevant for progressing and selecting items from the queue
impl Queue {
    pub fn tick(&mut self, tick: Duration) {
        while self
            .enqueued
            .peek()
            .map_or(Duration::MAX, |s| s.borrow().tick)
            <= tick
        {
            let next_server = self
                .enqueued
                .pop()
                .expect("Server was peeked and should have popped");

            let routing_data = ServerData::from(next_server.borrow().server());

            self.waiting.insert(
                next_server.borrow().server().id(),
                (next_server.clone(), routing_data),
            );
        }
    }

    #[must_use]
    pub fn next_tick(&self) -> Option<Duration> {
        self.enqueued.peek().map(|c| c.borrow().tick)
    }
}

// Misc
impl Queue {
    #[must_use]
    pub fn routing_data(&self) -> Vec<&ServerData> {
        self.waiting.values().map(|(_, s)| s).collect()
    }

    pub fn enqueue(&mut self, id: usize, until: Duration) {
        // TODO: Safely chceck that the server_queue has this server_id
        let (server, _) = self
            .waiting
            .remove(&id)
            .expect("Server Id should have been queued");

        server.borrow_mut().tick = until;

        self.enqueued.push(server);
    }
}
