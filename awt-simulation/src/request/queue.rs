use core::time::Duration;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{Request, Status};
use crate::routing::RequestData;
use min_queue::MinQueue;

pub struct Queue {
    inner: Vec<Rc<RefCell<Request>>>,
    enqueued: MinQueue<Rc<RefCell<Request>>>,
    waiting: HashMap<usize, (Rc<RefCell<Request>>, RequestData)>,
}

// Setup logic
impl Queue {
    pub fn new() -> Self {
        Self {
            inner: vec![],
            enqueued: MinQueue::new(),
            waiting: HashMap::new(),
        }
    }
    pub fn push(&mut self, req: Request) {
        let req = Rc::new(RefCell::new(req));

        self.inner.push(req);
    }

    // Assign all of the requests into the queue to be released
    pub fn init(&mut self) {
        for req in &self.inner {
            self.enqueued.push(req.clone());
        }
    }
}

// Ticking logic
impl Queue {
    pub fn tick(&mut self, tick: Duration) {
        // First tick the already waiting items, they cannot be assigned if they are already over
        // their waiting limit.
        self.tick_queued(tick);
        // ...then release new items to the waiting queue to be assigned.
        self.tick_release_to_queue(tick);
    }

    fn tick_queued(&mut self, tick: Duration) {
        self.waiting.retain(|_, (request, _)| {
            let mut request = request.borrow_mut();
            request.tick_wait(tick);
            &Status::Enqueued == request.status()
        });
    }

    fn tick_release_to_queue(&mut self, tick: Duration) {
        while self
            .enqueued
            .peek()
            .map_or(Duration::MAX, |c| c.borrow().start())
            <= tick
        {
            let next_request = self
                .enqueued
                .pop()
                .expect("Request was peeked and should have been popped");

            let mut request = next_request.borrow_mut();
            request.enqueue(tick);

            let routing_data = RequestData::from(&*request);

            self.waiting
                .insert(request.id(), (next_request.clone(), routing_data));
        }
    }

    pub fn next_tick(&self) -> Option<Duration> {
        self.enqueued.peek().map(|c| c.borrow().start())
    }
}

// Misc
impl Queue {
    pub fn requests(&self) -> &Vec<Rc<RefCell<Request>>> {
        &self.inner
    }

    pub fn has_waiting(&self) -> bool {
        !self.waiting.is_empty()
    }

    pub fn routing_data(&self) -> Vec<&RequestData> {
        self.waiting.values().map(|(_, r)| r).collect()
    }

    pub fn handle_request(&mut self, id: usize, tick: Duration) -> Duration {
        // TODO: This is not safe since id might not exist
        let mut request = self
            .waiting
            .get(&id)
            .expect("Client Id does not exist")
            .0
            .borrow_mut();

        request.handle(tick)
    }
}
