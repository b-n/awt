use crate::{simulation::Server, Attribute};

#[allow(dead_code)]
pub struct ServerData {
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
