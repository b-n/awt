use crate::simulation::{attribute::Attribute, request::Request};

#[allow(dead_code)]
pub struct RequestData {
    pub id: usize,
    pub start: usize,
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
