# Overall design goals

Simulate requests being made to servers, but do it real quick.


Requests follow poisson distribution rules - e.g. they are all random events that are indenpendant
from each other.

## Terminology

- [Simulation] Period - The total length of the period being simulated.
- Client = The thing that makes the request
- Server = The thing that handles the request
- Request = The actual unit of work that is being performed
- Attribute = Conditions that the Server must meet that the Client provides.

## Simulation

Simulate some real world equivilant time through splitting it down into `tick`s. Ticks are just a
subdevision of a real world unit. For most purposes, a tick is 1ms (e.g. 1000 ticks per second).

There is a trade off in terms of how many ticks there are per simulation:

- The more tick subdivisions, the "more accurate" a real life simulation
- The more ticks, the slower the overall simulation (a lot more time to check)

### Routing

Not all requests can be made to all servers. The client has certain attributes which must be met by
the server in order to facilitate the connection.

Routing can be provided by matching the current state of the servers with the incoming request.

Knowns:
- The attributes the client request requires
- The available attributes on all servers
- The current wait time of the request

Unknowns:
- The length of time left for each request on each server (probing should not be possible, because
  requests only end when they disconnect)

### Optimizations

- Request start times can be rolled in the beginning, instead of every tick
- Ticks can be fast forwarded when there are no requests waiting for servers. Ticks can be fast
  forwarded to:
  - The next request start (rolled at the start of the simulation)
  - The next request end (when the server releases back into the server pool)

Ticks can't be fast forwarded whilst waiting for servers because routing logic may choose to hold
off making a connection until a more applicable server is available. e.g. Given a Server with
attributes of AB, and a client of attributes B, we may choose to leave a request waiting until a
server with only attributes of B becomes available or until a certain amount of time has elapsed.
This gives the router slightly more flexibility in terms of routing.

## Metrics

For a given simulation period, we want to measure certain statistics. These statistics are
aggregated values which show some level of "health" of a certain simulation. Simulations can have
many of these metrics (even multiples of the same metric, SLA for example).

Each metric cares about a specific value from each request. e.g. AverageConnectionTime is only concerned
with the connection time of a request, but is not concerned with the answer time.

Reporting on each metric is possible as soon as specific events happen - specifically,
time to answer/abandon metrics are available as soon as a request is connected/abandoned, however
the value does not change when the connection is finished.
