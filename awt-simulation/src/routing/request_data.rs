use core::time::Duration;

use crate::{Attribute, Request};

#[allow(dead_code)]
pub struct RequestData {
    pub id: usize,
    pub start: Duration,
    pub required_attributes: Vec<Attribute>,
}

impl From<&Request> for RequestData {
    fn from(client: &Request) -> Self {
        Self {
            id: client.id(),
            start: client.start(),
            required_attributes: client.required_attributes().clone(),
        }
    }
}
