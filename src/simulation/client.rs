use super::{Attribute, ClientProfile, TICKS_PER_SECOND};
use std::sync::Arc;
use std::cmp::Ordering;

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
    abandon_at: usize,
    established: Option<usize>,
    end: Option<usize>,
    status: Status,
}

impl Ord for Client {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Client {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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

    pub fn set_start(&mut self, tick: usize) {
        self.start = tick;
        self.abandon_at = 20 * TICKS_PER_SECOND;
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn add_required_attribute(&mut self, attr: &Attribute) {
        self.required_attributes.push(attr.clone());
    }

    pub fn is_waiting(&self) -> bool {
        Status::Unanswered == self.status
    }

    pub fn enqueue(&mut self, tick: usize) {
        println!("[CLIENT] {} enqueued at {}", self.id, tick);
    }

    // Returns whether the Client is continuing to wait
    pub fn tick_wait(&mut self, tick: usize) -> bool {
        if tick < self.start {
            panic!(
                "Cannot tick in the past. started: {}, current: {}",
                self.start, tick
            );
        }

        if self.start + self.abandon_at < tick {
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
