use super::QueueableServer;
use crate::simulation::routing::ServerData;
use crate::MinQueue;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Queue {
    inner: Vec<Rc<RefCell<QueueableServer>>>,
    enqueued: MinQueue<Rc<RefCell<QueueableServer>>>,
    waiting: HashMap<usize, (Rc<RefCell<QueueableServer>>, ServerData)>,
}

// Setup and cretion logic
impl Queue {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            enqueued: MinQueue::new(),
            waiting: HashMap::new(),
        }
    }

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
    pub fn tick(&mut self, tick: usize) {
        while self.enqueued.peek().map_or(usize::MAX, |s| s.borrow().tick) <= tick {
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

    pub fn next_tick(&self) -> Option<usize> {
        self.enqueued.peek().map(|c| c.borrow().tick)
    }
}

// Misc
impl Queue {
    pub fn routing_data(&self) -> Vec<&ServerData> {
        self.waiting.values().map(|(_, s)| s).collect()
    }

    pub fn enqueue(&mut self, id: usize, until: usize) {
        // TODO: Safely chceck that the server_queue has this server_id
        let (server, _) = self
            .waiting
            .remove(&id)
            .expect("Server Id should have been queued");

        server.borrow_mut().tick = until;

        self.enqueued.push(server);
    }
}