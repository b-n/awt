pub mod request_data;
use super::Server;

pub use request_data::RequestData;

use std::sync::Arc;

// TODO: Support rlua? (allow custom lua scripts to execute and return routes)
pub fn route_requests(
    requests: Vec<&RequestData>,
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
