mod request_data;
mod server_data;

pub(crate) use request_data::RequestData;
pub(crate) use server_data::ServerData;

// TODO: Support rlua? (allow custom lua scripts to execute and return routes)
pub(crate) fn route_requests(
    requests: Vec<&RequestData>,
    mut servers: Vec<&ServerData>,
) -> Vec<(usize, usize)> {
    let mut routes = vec![];

    for request in requests {
        if let Some(server) = servers.pop() {
            routes.push((request.id, server.id));
        }
    }

    routes
}
