use std::cmp::Ordering;

// A contact has the following:
// - Expected handle time (100% which can be buffed up or down by agent stats)
// - Expected After handle time (100% which can be buffed up or down by agent stats)
// - Required training to answer the call

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
    expected_talk_time: usize,
    expected_acw: usize,
    nps: Option<u8>,
    start: Option<usize>,
    end: Option<usize>,
    result: Option<ContactResult>,
}

impl Contact {
    pub fn new() -> Self {
        Contact {
            skill_preference: None,
            expected_talk_time: 300,
            expected_acw: 300,
            nps: None,
            start: None,
            end: None,
            result: None,
        }
    }

    pub fn start_contact(&mut self, ticks: usize) {
        self.start = Some(ticks)
    } 

    pub fn end_contact(&mut self, ticks: usize) {
        self.end = Some(ticks)
    }

    pub fn call_length(&mut self) -> Option<usize> {
        match (self.start, self.end) {
            (None, _) => None,
            (_, None) => None,
            (Some(start), Some(end)) => Some(end - start),
        }
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
    queue: Vec<Contact>
}

impl ContactQueue {
    pub fn new() -> Self {
        Self { queue: vec![] }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Contact> {
        self.queue.iter_mut()
            .filter(|contact| contact.result == None)
    }
}















