// The game: Optimize a call center
//
// We have a contact center, that call center has a people we hire to take calls
// The call center will receive an hour a number of contacts. Those contacts have some statistics that need to be
// met
//
// Statistics (per contact type):
// - SLA (% answered in X time)
// - Call abandon rate < x%
// - Average wait time
// - Client happiness score
//
// An hour is simulated X times. The statistics for each of those X times is averaged
// - Over achieving a statistic doesn't do anything
// - Meeting the statistic gives money
// - Missing a statistic will reduce won money
//
// Money can be spent on:
// - Training agents - reduce call time
// - Improve the tools - reduce after call work
// - Bonuses to agents - makes them work more effectively
// - Upskilling those agents to handle calls faster
// - Self service (e.g. reduce number of calls) - inverse log effectiveness
// - Self service help (e.g reduce expected call time) - inverse log effectiveness
// - Marketing to use different channels (that can be async and thus faster)

// Setup Clippy
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(unknown_lints)]
#![warn(missing_debug_implementation)]
#![warn(missing_copy_implementation)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_difference)]
// Caused by hermit-abi dependency in rayon and clap
#![allow(clippy::multiple_crate_versions)]

use clap::Parser;
use core::time::Duration;
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use rayon::prelude::*;
use std::thread::available_parallelism;

mod args;
mod attribute;
mod config;
mod metric;
mod min_queue;
mod simulation;

use args::Args;
use attribute::Attribute;
use config::Config;
use metric::{Metric, MetricType};
use min_queue::MinQueue;
use simulation::{Client, Server, Simulation};

fn run_sim(
    counter: usize,
    tick_size: Duration,
    tick_until: Duration,
    servers: Vec<Server>,
    clients: Vec<Client>,
    metrics: Vec<Metric>,
) {
    // Rust docs says we can trust this won't fail 🤞
    // Ref: https://docs.rs/rand/latest/rand/rngs/struct.SmallRng.html#examples
    let rng = Box::new(SmallRng::from_rng(thread_rng()).unwrap());
    let mut sim = Simulation::new(rng, tick_size, tick_until);

    for server in servers {
        sim.add_server(server);
    }

    for client in clients {
        sim.add_client(client);
    }

    for metric in metrics {
        sim.add_metric(metric);
    }

    sim.enable();

    while sim.tick() {}

    println!("Sim {counter} {:?}\n{}", sim.running(), sim.statistics());
}

fn main() {
    match try_main() {
        Ok(_) => {
            std::process::exit(exitcode::OK);
        }
        Err(err) => {
            println!("{err}");
            std::process::exit(exitcode::USAGE);
        }
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config_path = args.config_path.unwrap();

    let config = Config::try_from(&config_path)?;

    // We want to pin some cores, but not all the cores
    let sim_threads = available_parallelism()?.get() - 1;

    //Setup a fairly safe thread pool.
    rayon::ThreadPoolBuilder::new()
        .num_threads(sim_threads)
        .build_global()?;

    // Retrieve all the required values from the config prior to starting the simulation run('s)
    let clients = config.clients();
    let servers = config.servers();
    let metrics = config.metrics()?;
    let simulations = config.simulations;
    let tick_size = config.tick_size;
    let tick_until = config.tick_until;

    (0..simulations).into_par_iter().for_each(|sim| {
        // All values are cloned from the above instead of generating each round. All simulation
        // data is owned to ensure encapsulation of the data, and for speed at the trade off of
        // memory usage (which is very small per sim).
        run_sim(
            sim,
            tick_size,
            tick_until,
            servers.clone(),
            clients.clone(),
            metrics.clone(),
        );
    });

    Ok(())
}
