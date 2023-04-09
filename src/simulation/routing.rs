use super::{Attribute, Request, Server};

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[allow(dead_code)]
pub struct RequestRoutingData {
    id: usize,
    start: usize,
    required_attributes: Vec<Attribute>,
}

impl From<&Rc<RefCell<Request>>> for RequestRoutingData {
    fn from(client: &Rc<RefCell<Request>>) -> Self {
        let client = client.borrow();
        Self {
            id: client.id(),
            start: client.start(),
            required_attributes: client.required_attributes().clone(),
        }
    }
}

// TODO: Support rlua? (allow custom lua scripts to execute and return routes)
pub fn route_requests(
    requests: &Vec<RequestRoutingData>,
    mut servers: Vec<&Arc<Server>>,
) -> Vec<(usize, usize)> {
    let mut routes = vec![];

    for request in requests {
        if let Some(server) = servers.pop() {
            routes.push((request.id, server.id()));
        }
    }

    routes
}
