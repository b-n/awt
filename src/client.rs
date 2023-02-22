use crate::Attribute;
use crate::ClientProfile;
use std::sync::Arc;
use crate::{TICKS_PER_SECOND};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Unanswered,
    Abandoned,
    Answered,
}

impl Default for Status {
    fn default() -> Self {
        Self::Unanswered
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Client {
    id: usize,
    required_attributes: Vec<Attribute>,
    start: usize,
    max_wait_time: usize,
    established: Option<usize>,
    end: Option<usize>,
    status: Status,
}

impl From<&Arc<ClientProfile>> for Client { 
    fn from(client_profile: &Arc<ClientProfile>) -> Self {
        let mut client = Self::default();
        client.required_attributes = client_profile.required_attributes.clone(); 
        client
    }
}

impl Client {
    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    pub fn add_required_attribute(&mut self, attr: &Attribute) {
        self.required_attributes.push(attr.clone());
    }

    pub fn is_waiting(&self) -> bool {
        Status::Unanswered == self.status
    }

    pub fn enqueue(&mut self, tick: usize) {
        self.status = Status::Unanswered;
        self.start = tick;
        self.max_wait_time = 20 * TICKS_PER_SECOND;
        println!("[CLIENT] {} enqueued at {}", self.id, tick);
    }

    // Returns whether the Client is continuing to wait
    pub fn tick_wait(&mut self, tick: usize) -> bool {
        if tick < self.start {
            panic!("Cannot tick in the past. started: {}, current: {}", self.start, tick);
        }

        if self.start + self.max_wait_time < tick {
            println!("[CLIENT] {} abandoned at {}", self.id, tick);
            self.status = Status::Abandoned;
            self.end = Some(tick);
            false
        } else {
            true
        } 
    }

    // Handle this client
    pub fn handle(&mut self, current_tick: usize, handling_time: usize) {
        self.established = Some(current_tick);
        self.end = Some(current_tick + handling_time);
        self.status = Status::Answered;
    }
}
