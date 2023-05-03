use std::cmp::{Ord, Reverse};
use std::collections::binary_heap::*;

#[derive(Clone, Default, core::fmt::Debug)]
pub struct MinQueue<T: Ord> {
    inner: BinaryHeap<Reverse<T>>,
}

impl<T: Ord> MinQueue<T> {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: BinaryHeap::new(),
        }
    }

    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: BinaryHeap::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop().map(|t| t.0)
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        self.inner.push(Reverse(item));
    }

    #[inline]
    pub fn peek(&self) -> Option<&T> {
        self.inner.peek().map(|t| &t.0)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }

    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn drain(&mut self) {
        self.inner.drain();
    }
}
