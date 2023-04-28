# AWT

[![CI](https://github.com/b-n/awt/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/b-n/awt/actions/workflows/ci.yml)

A small call center sim game written in Rust!

## Building

`cargo build --release`

## Running

Run directly from cargo with:

`cargo run --release -- <path/to/config.toml>`

## Configuration

This simulation runner is designed to run based on TOML configs. The path to the TOML configuration
can be supplied via command line argument.

The structure is broken up into the following areas:

- Simulation Setup
- Client Setup
- Server Setup
- Metric Setup
- Attributes

Base simulation setup is as follows:

`simulations` - **Integer** - The amount of simulations to run

`tick_size` - **Duration** - The size of a tick to tick

`tick_until` - **Duration** - The end point of the simulation

`clients` - **Array<Client>** - An array of `client` which supports many request of many patterns

`servers` - **Array<Server>** - An array of `server` which supports handling many requests of many
patterns

`metrics` - **Array<Metric>** - An array of `metric` which supports the creation of metrics to be
measured in the simulation

The maximum number of actual ticks can be represented by `tick_until` / `tick_size`.

Example:

- `tick_until` is 3600 seconds
- `tick_size` is 0secds, `10_000_000` nanos (10ms)

The total possible ticks is `3600` / `0.01` = `360000` ticks per simulation.

### Client

`handle_time` - **Duration** - The time a request will use of a server after answering

`abandon_time` - **Duration** - The time a request will wait until it abandons

`clean_up_time` - **Duration** - Future use

`quantity` - **Integer** - The number of requests to create that match the above parameters

`required_attributes` - **Attribute** - Future use

### Server

`quantity` - **Integer** - The amount of servers to create to handle the requests

`attributes` - **Attribute** - Future use

### Metric

`metric` - **MetricType** - The type of metric to create

`sla` - **Duration** - (Used only for ServiceLevel Metric) The amount of seconds for the SLA

`target` - **Variable** - The target for the metric

The following metric types (and their targets) are supported

| MetricType               | Target Type |
| ------------------------ | ----------- |
| `ServiceLevel(Duration)` | float64     |
| `AverageWorkTime`        | Duration    |
| `AverageSpeedAnswer`     | Duration    |
| `AverageTimeToAbandon`   | Duration    |
| `AverageTimeInQueue`     | Duration    |
| `AverageWorkTime`        | Duration    |
| `AbandonRate`            | float64     |
| `AnswerCount`            | Integer     |

## Attribute

Future use

## TODO

- [ ] Secure attribute routing
- [ ] Support for lua in routing

## License

`awt` is licenses with the MIT License (c) Ben Naylor.
