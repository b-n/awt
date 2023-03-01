use super::{Attribute, ClientProfile, TICKS_PER_SECOND};
use std::cmp::Ordering;
use std::sync::Arc;

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
    abandon_tick: usize,
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
        Self {
            required_attributes: client_profile.required_attributes.clone(),
            ..Self::default()
        }
    }
}

impl Client {
    pub fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn set_start(&mut self, tick: usize) {
        self.start = tick;
        self.abandon_tick = self.start + 20 * TICKS_PER_SECOND;
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[allow(dead_code)]
    pub fn add_required_attribute(&mut self, attr: &Attribute) {
        self.required_attributes.push(attr.clone());
    }

    #[inline]
    pub fn required_attributes(&self) -> &Vec<Attribute> {
        &self.required_attributes
    }

    #[inline]
    pub fn is_unanswered(&self) -> bool {
        Status::Unanswered == self.status
    }

    #[inline]
    pub fn is_answered(&self) -> bool {
        Status::Answered == self.status
    }

    #[inline]
    pub fn is_abandoned(&self) -> bool {
        Status::Abandoned == self.status
    }

    pub fn enqueue(&mut self, tick: usize) {
        println!("[CLIENT] {} enqueued at {}", self.id, tick);
    }

    // Returns whether the Client is continuing to wait
    pub fn tick_wait(&mut self, tick: usize) -> bool {
        if !self.is_unanswered() {
            return false;
        }

        assert!(
            tick >= self.start,
            "Cannot tick in the past. started: {}, current: {}",
            self.start,
            tick
        );

        if self.abandon_tick < tick {
            println!("[CLIENT] {} abandoned at {}", self.id, tick);
            self.status = Status::Abandoned;
            self.end = Some(tick);
            false
        } else {
            true
        }
    }

    pub fn handle(&mut self, tick: usize, handling_time: usize) -> usize {
        println!("[CLIENT] {} handled at {}", self.id, tick);
        self.established = Some(tick);
        let end = tick + handling_time;
        self.end = Some(end);
        self.status = Status::Answered;

        end
    }
}
