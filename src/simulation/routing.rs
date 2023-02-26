use super::{Client, Server};

use std::cell::RefCell;
use std::sync::Arc;

// TODO: Support rlua? (allow custom lua scripts to execute and return routes)
pub fn route_client(client: &RefCell<Client>, servers: &[Arc<Server>]) -> Option<Arc<Server>> {
    let _ = client;
    servers.first().map(Clone::clone)
}

// `route_client` should eventually support user provided routing. If/when that happens, then we
// need to ensure that the route is validated. A false result should result in a "BadRoute" or
// equiviliant `Client` Status.
#[allow(dead_code)]
pub fn validate_route(_client: &RefCell<Client>, _server: &Arc<Server>) -> bool {
    true
}
