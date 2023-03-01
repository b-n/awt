use super::{Attribute, Client, Server};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct ClientRoutingData {
    id: usize,
    start: usize,
    required_attributes: Vec<Attribute>,
}

impl From<&Rc<RefCell<Client>>> for ClientRoutingData {
    fn from(client: &Rc<RefCell<Client>>) -> Self {
        let client = client.borrow();
        Self {
            id: client.id(),
            start: client.start(),
            required_attributes: client.required_attributes().clone(),
        }
    }
}

// TODO: Support rlua? (allow custom lua scripts to execute and return routes)
pub fn route_clients(
    clients: &Vec<ClientRoutingData>,
    mut servers: Vec<&Arc<Server>>,
) -> Vec<(usize, usize)> {
    let mut routes = vec![];

    for client in clients {
        if let Some(server) = servers.pop() {
            routes.push((client.id, server.id()));
        }
    }

    routes
}
