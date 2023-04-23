pub mod queue;

pub use queue::Queue;

use super::{Attribute, ClientProfile};
use std::cmp::Ordering;
use std::sync::{atomic, atomic::AtomicUsize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Pending,
    Enqueued,
    Abandoned,
    Answered,
}

impl Default for Status {
    fn default() -> Self {
        Self::Pending
    }
}

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    id: usize,
    required_attributes: Vec<Attribute>,
    start: usize,
    abandon_ticks: usize,
    handle_ticks: usize,
    established: Option<usize>,
    end: Option<usize>,
    status: Status,
    source: ClientProfile,
}

impl Ord for Request {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Request {
    /// Generate a `Request` based on the provided ticks and attributes
    pub fn new(
        start: usize,
        abandon_ticks: usize,
        handle_ticks: usize,
        required_attributes: Vec<Attribute>,
        source: &ClientProfile,
    ) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, atomic::Ordering::SeqCst),
            start,
            abandon_ticks,
            handle_ticks,
            required_attributes,
            established: None,
            end: None,
            status: Status::default(),
            source: source.clone(),
        }
    }

    /// Returns the `id` of this client.
    ///
    /// The `id` is autogenerated when the client is constructed, and is gauranteed to be unique
    /// for all `Client`s in this thread.
    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the `start` tick of this client (when it should be enqueud).
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

    /// Returns the `Status` of this `Client`.
    #[inline]
    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn enqueue(&mut self, tick: usize) {
        assert!(
            tick == self.start,
            "[REQUEST] {}. Unexpected Enqueue time. Expected: {}, Tried at: {}",
            self.id,
            self.start,
            tick
        );
        self.status = Status::Enqueued;
        println!("[REQUEST] {} enqueued at {}", self.id, tick);
    }

    // Returns whether the Request is continuing to wait
    pub fn tick_wait(&mut self, tick: usize) -> bool {
        if Status::Enqueued != self.status {
            return false;
        }

        assert!(
            tick >= self.start,
            "Cannot tick in the past. started: {}, current: {}",
            self.start,
            tick
        );

        if self.abandon_ticks <= tick {
            println!("[REQUEST] {} abandoned at {}", self.id, tick);
            self.status = Status::Abandoned;
            self.end = Some(tick);
            false
        } else {
            true
        }
    }

    pub fn handle(&mut self, tick: usize) -> usize {
        assert_eq!(
            Status::Enqueued,
            self.status,
            "Cannot tick Client when not enqueued"
        );

        assert!(
            tick >= self.start,
            "Cannot tick in the past. started: {}, current: {}",
            self.start,
            tick
        );

        println!("[REQUEST] {} handled at {}", self.id, tick);
        self.established = Some(tick);
        let end = tick + self.handle_ticks;
        self.end = Some(end);
        self.status = Status::Answered;

        end
    }

    pub fn wait_time(&self) -> Option<usize> {
        self.established.or(self.end).map(|t| t - self.start)
    }

    #[allow(dead_code)]
    pub fn handle_time(&self) -> Option<usize> {
        if Status::Answered == self.status {
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

    const ABANDON_TICKS: usize = 1000;
    const HANDLE_TICKS: usize = 300_000;

    fn default_request(start: usize) -> (Request, usize) {
        let client_profile = ClientProfile::default();

        let abandon_ticks = start + ABANDON_TICKS;
        let handle_ticks = HANDLE_TICKS;

        (
            Request::new(start, abandon_ticks, handle_ticks, vec![], &client_profile),
            abandon_ticks,
        )
    }

    fn enqueued_request(start: usize) -> (Request, usize) {
        let (mut request, abandon_ticks) = default_request(start);

        request.enqueue(start);
        (request, abandon_ticks)
    }

    #[test]
    fn default_status_is_pending() {
        let (request, _) = default_request(0);
        assert_eq!(&Status::Pending, request.status());
    }

    #[test]
    fn abandons_past_abandonment_tick() {
        let (mut request, abandon_tick) = enqueued_request(100);

        request.tick_wait(abandon_tick - 1);
        assert_eq!(&Status::Enqueued, request.status());
        assert!(!request.tick_wait(abandon_tick));
        assert_eq!(&Status::Abandoned, request.status());
    }

    #[test]
    fn only_ticks_when_unanswered() {
        let (mut request, abandon_tick) = enqueued_request(100);

        request.tick_wait(abandon_tick);

        assert!(!request.tick_wait(abandon_tick + 1));
    }

    #[should_panic]
    #[test]
    fn panics_ticking_in_past() {
        let (mut request, _) = enqueued_request(100);

        request.tick_wait(99);
    }

    #[test]
    fn handling_handles() {
        let (mut request, _) = enqueued_request(100);

        assert_eq!(100 + HANDLE_TICKS, request.handle(100));
        assert_eq!(&Status::Answered, request.status());
        assert_eq!(Some(0), request.wait_time());
    }

    #[should_panic]
    #[test]
    fn handle_only_when_enqueued() {
        let (mut request, _) = default_request(100);

        assert_eq!(&Status::Pending, request.status());

        request.handle(120);
    }

    #[should_panic]
    #[test]
    fn handling_only_works_in_future() {
        let (mut request, _) = enqueued_request(100);

        request.handle(99);
    }

    #[test]
    fn wait_time_abandoned() {
        let (mut request, abandon_tick) = enqueued_request(100);
        request.tick_wait(abandon_tick);

        assert_eq!(&Status::Abandoned, request.status());
        assert_eq!(Some(ABANDON_TICKS), request.wait_time());
    }

    #[test]
    fn wait_time_answered() {
        let (mut request, _) = enqueued_request(100);
        request.handle(200);

        assert_eq!(&Status::Answered, request.status());
        assert_eq!(Some(100), request.wait_time());
    }

    #[test]
    fn wait_time_unanswered() {
        let (mut request, _) = enqueued_request(100);
        request.tick_wait(101);

        assert_eq!(&Status::Enqueued, request.status());
        assert_eq!(None, request.wait_time());
    }

    #[test]
    fn handle_time_unanswered() {
        let (mut request, _) = enqueued_request(100);
        request.tick_wait(101);

        assert_eq!(&Status::Enqueued, request.status());
        assert_eq!(None, request.handle_time());
    }

    #[test]
    fn handle_time_answered() {
        let (mut request, _) = enqueued_request(100);
        request.handle(200);

        assert_eq!(&Status::Answered, request.status());
        assert_eq!(Some(HANDLE_TICKS), request.handle_time());
    }

    #[test]
    fn handle_time_abandonend() {
        let (mut request, abandon_tick) = enqueued_request(100);
        request.tick_wait(abandon_tick);

        assert_eq!(&Status::Abandoned, request.status());
        assert_eq!(None, request.handle_time());
    }
}
