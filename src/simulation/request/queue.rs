use super::{Request, Status};
use crate::simulation::routing::RequestData;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::MinQueue;

pub struct Queue {
    inner: Vec<Rc<RefCell<Request>>>,
    enqueued: MinQueue<Rc<RefCell<Request>>>,
    waiting: HashMap<usize, (Rc<RefCell<Request>>, RequestData)>,
}

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

    pub fn generate_queued(&mut self) {
        for req in &self.inner {
            self.enqueued.push(req.clone());
        }
    }

    pub fn requests(&self) -> &Vec<Rc<RefCell<Request>>> {
        &self.inner
    }

    pub fn enqueue(&mut self, tick: usize) {
        while self
            .enqueued
            .peek()
            .map_or(usize::MAX, |c| c.borrow().start())
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

    pub fn has_waiting(&self) -> bool {
        !self.waiting.is_empty()
    }

    pub fn enqueued_head(&self) -> Option<usize> {
        self.enqueued.peek().map(|c| c.borrow().start())
    }

    pub fn routing_data(&self) -> Vec<&RequestData> {
        self.waiting.values().map(|(_, r)| r).collect()
    }

    pub fn handle_request(&mut self, id: usize, tick: usize) -> usize {
        // TODO: This is not safe since id might not exist
        let mut request = self
            .waiting
            .get(&id)
            .expect("Client Id does not exist")
            .0
            .borrow_mut();

        request.handle(tick)
    }

    pub fn tick_waiting(&mut self, tick: usize) {
        self.waiting.retain(|_, (request, _)| {
            let mut request = request.borrow_mut();
            request.tick_wait(tick);
            &Status::Enqueued == request.status()
        });
    }
}
