use crate::simulation::{attribute::Attribute, server::Server};

use std::sync::Arc;

#[allow(dead_code)]
pub struct ServerData {
    pub id: usize,
    pub attributes: Vec<Attribute>,
}

impl From<&Arc<Server>> for ServerData {
    fn from(server: &Arc<Server>) -> Self {
        Self {
            id: server.id(),
            attributes: server.attributes().clone(),
        }
    }
}
