use crate::{Attribute, Server};
use alloc::vec::Vec;

#[allow(dead_code)]
pub(crate) struct ServerData {
    pub id: usize,
    pub attributes: Vec<Attribute>,
}

impl From<&Server> for ServerData {
    fn from(server: &Server) -> Self {
        Self {
            id: server.id(),
            attributes: server.attributes().clone(),
        }
    }
}
