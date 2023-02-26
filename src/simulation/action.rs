use crate::Client;
use std::sync::Arc;

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Action {
    ReleaseServer(Arc<Server>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct TimedAction {
    tick: usize,
    action: Action,
}

impl Ord for TimedAction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tick.cmp(&other.tick)
    }
}

impl PartialOrd for TimedAction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ActionQueue {
    queue: BinaryHeap<TimedAction>,
}

impl ActionQueue {
    pub fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    /// Push a new action to the queue at a given time (tick)
    pub fn push(&mut self, tick: usize, action: Action) {
        let timed_action = TimedAction { tick, action };
        self.queue.push(timed_action)
    }

    /// Pop the next available action that is before the provided tick
    pub fn pop(&mut self, before: usize) -> Option<Action> {
        let next = self.queue.peek()?.tick;

        if next > before {
            None
        } else {
            let timed = self.queue.pop()?;
            Some(timed.action)
        }
    }
}
