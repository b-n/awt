use super::{Attribute, ClientProfile, TICKS_PER_SECOND};
use std::cmp::Ordering;
use std::sync::{atomic, atomic::AtomicUsize, Arc};

const ABANDON_TICKS: usize = 20 * TICKS_PER_SECOND;

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

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    id: usize,
    required_attributes: Vec<Attribute>,
    start: usize,
    abandon_tick: usize,
    established: Option<usize>,
    end: Option<usize>,
    status: Status,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, atomic::Ordering::SeqCst),
            required_attributes: vec![],
            start: 0,
            abandon_tick: 0,
            established: None,
            end: None,
            status: Status::default(),
        }
    }
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
    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn set_start(&mut self, tick: usize) -> usize {
        self.start = tick;
        self.abandon_tick = self.start + ABANDON_TICKS;
        self.abandon_tick
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

        if self.abandon_tick <= tick {
            println!("[CLIENT] {} abandoned at {}", self.id, tick);
            self.status = Status::Abandoned;
            self.end = Some(tick);
            false
        } else {
            true
        }
    }

    pub fn handle(&mut self, tick: usize, handling_time: usize) -> usize {
        assert!(
            tick >= self.start,
            "Cannot tick in the past. started: {}, current: {}",
            self.start,
            tick
        );

        println!("[CLIENT] {} handled at {}", self.id, tick);

        self.established = Some(tick);
        let end = tick + handling_time;
        self.end = Some(end);
        self.status = Status::Answered;

        end
    }

    #[allow(dead_code)]
    pub fn wait_time(self) -> Option<usize> {
        self.established.or(self.end).map(|t| t - self.start)
    }

    #[allow(dead_code)]
    pub fn handle_time(self) -> Option<usize> {
        if self.is_answered() {
            let established = self
                .established
                .expect("Client should have an established time if answered");
            self.end.map(|t| t - established)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_status() {
        let client = Client::default();
        assert!(client.is_unanswered());
    }

    #[test]
    fn abandons_past_abadonment_tick() {
        let mut client = Client::default();
        let abandon_tick = client.set_start(100);

        client.tick_wait(abandon_tick - 1);
        assert!(!client.is_abandoned());
        assert!(!client.tick_wait(abandon_tick));
        assert!(client.is_abandoned());
    }

    #[test]
    fn only_ticks_when_unanswered() {
        let mut client = Client::default();
        let abandon_tick = client.set_start(100);

        client.tick_wait(abandon_tick);

        assert!(!client.tick_wait(abandon_tick + 1));
    }

    #[should_panic]
    #[test]
    fn panics_ticking_in_past() {
        let mut client = Client::default();
        client.set_start(100);

        client.tick_wait(99);
    }

    #[test]
    fn handling_handles() {
        let mut client = Client::default();
        client.set_start(100);

        assert_eq!(200, client.handle(100, 100));
        assert!(client.is_answered());
        assert_eq!(Some(0), client.wait_time());
    }

    #[should_panic]
    #[test]
    fn handling_only_works_in_future() {
        let mut client = Client::default();
        client.set_start(100);

        client.handle(99, 1);
    }

    #[test]
    fn wait_time_abandoned() {
        let mut client = Client::default();
        let start_tick = 100;
        let abandon_tick = client.set_start(start_tick);
        client.tick_wait(abandon_tick);

        assert!(client.is_abandoned());
        assert_eq!(Some(abandon_tick - start_tick), client.wait_time());
    }

    #[test]
    fn wait_time_answered() {
        let mut client = Client::default();
        client.set_start(100);
        client.handle(200, 1);

        assert!(client.is_answered());
        assert_eq!(Some(100), client.wait_time());
    }

    #[test]
    fn wait_time_unanswered() {
        let mut client = Client::default();
        client.set_start(100);
        client.tick_wait(101);

        assert!(client.is_unanswered());
        assert_eq!(None, client.wait_time());
    }

    #[test]
    fn handle_time_unanswered() {
        let mut client = Client::default();
        client.set_start(100);
        client.tick_wait(101);

        assert!(client.is_unanswered());
        assert_eq!(None, client.handle_time());
    }

    #[test]
    fn handle_time_answered() {
        let mut client = Client::default();
        client.set_start(100);
        client.handle(200, 100);

        assert!(client.is_answered());
        assert_eq!(Some(100), client.handle_time());
    }

    #[test]
    fn handle_time_abandonend() {
        let mut client = Client::default();
        let abandon_tick = client.set_start(100);
        client.tick_wait(abandon_tick);

        assert!(client.is_abandoned());
        assert_eq!(None, client.handle_time());
    }
}
