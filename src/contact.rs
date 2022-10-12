use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use rand::{thread_rng, Rng};

use crate::Agent;

// A contact has the following:
// - Expected handle time (100% which can be buffed up or down by agent stats)
// - Expected After handle time (100% which can be buffed up or down by agent stats)
// - Required training to answer the call

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metrics {
    wait: Option<usize>,
    talk: Option<usize>,
    after: Option<usize>,
    result: Option<ContactResult>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics { wait: None, talk: None, after: None, result: None }
    }
}

pub enum ContactType {
    Call,
    Email,
    Chat,
    SocialMedia,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ContactResult {
    ABANDONED,
    ANSWERED, 
}

/// start = the point the contact was answered
/// end = start + contact time + after contact work
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Contact {
    skill_preference: Option<usize>,
    base_talk_time: usize,
    base_acw: usize,
    time_to_abandon: usize,
    nps: Option<u8>,
    start: usize,
    end: Option<usize>,
    metrics: Metrics,
}

impl Contact {
    pub fn new(start: usize, time_to_abandon: usize) -> Self {
        Contact {
            skill_preference: None,
            base_talk_time: 300,
            base_acw: 300,
            nps: None,
            start,
            time_to_abandon,
            end: None,
            metrics: Metrics::new(),
        }
    }

    pub fn is_waiting(&self) -> bool {
        self.metrics.result.is_none()
    }

    /// Tick this contact based on a simulation time
    ///
    /// Contact will abandon if not answered in time
    pub fn tick(&mut self, current_tick: usize) {
        if current_tick < self.start {
            panic!("Cannot tick in the past for an already started call");
        }

        if self.start + self.time_to_abandon < current_tick {
            self.metrics.result = Some(ContactResult::ABANDONED);
            self.end = Some(current_tick);
        }
    }

    /// Answer this contact with the provided agent
    fn answer(&mut self, ticks: usize, _: Agent) {
        let start = self.start;
        if ticks < start {
            panic!("Contact cannot be picked up before being started");
        }
        self.metrics.wait = Some(ticks - start);
        self.metrics.talk = Some(self.base_talk_time);
        self.metrics.after = Some(self.base_acw);
        self.metrics.result = Some(ContactResult::ANSWERED);
     }
}

impl Ord for Contact {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Contact {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ContactQueue {
    inner: HashMap<usize, Contact>,
    waiting: HashSet<usize>,
}

impl ContactQueue {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            waiting: HashSet::new()
        }
    }

    /// Generates a random new id which is gauranteed to not collide with another item in the queue
    fn next_queue_id(&self) -> usize {
        let mut rng = thread_rng();
        let mut id = rng.gen();
        while self.inner.contains_key(&id) {
            id = rng.gen();
        }
        id
    }

    /// Push an owned contact onto this queue
    pub fn push(&mut self, contact: Contact) {
        let id = self.next_queue_id();
        self.inner.insert(id, contact);
        self.waiting.insert(id);
    }

    /// Returns a mutable list of contacts which are currently waiting in the contact queue
    pub fn waiting(&mut self) -> impl Iterator<Item = &mut Contact> {
        self.inner.iter_mut()
            .filter(|(id, _)| self.waiting.contains(id))
            .map(|(_, contact)| contact)
    }
}
