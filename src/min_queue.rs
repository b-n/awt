use std::cmp::{Ord, Reverse};
use std::collections::BinaryHeap;

#[derive(Clone)]
pub struct MinQueue<T> {
    inner: BinaryHeap<Reverse<T>>,
}

impl<T> MinQueue<T>
where
    T: Ord,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: BinaryHeap::new(),
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        self.inner.push(Reverse(item));
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop().map(|t| t.0)
    }

    #[inline]
    pub fn peek(&self) -> Option<&T> {
        self.inner.peek().map(|t| &t.0)
    }
}
