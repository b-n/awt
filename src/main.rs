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
use log::{debug, error, info, trace};
use rayon::prelude::*;
use std::thread::available_parallelism;

mod args;
mod config;

use args::{log_level, Args};
use awt_metrics::{Aggregator, Metric};
use awt_simulation::{
    attribute::Attribute, client::Client, error::Error as SimulationError, server::Server,
    Config as SimulationConfig, Simulation,
};

use config::Config;

fn run_sim(
    counter: usize,
    config: SimulationConfig,
    metrics: &[Metric],
) -> Result<(), SimulationError> {
    let mut sim = Simulation::from(config);
    info!(target: "main", "sim {counter}: created");

    sim.enable()?;
    info!(target: "main", "sim {counter}: enabled");

    while sim.tick() {}
    info!(target: "main", "sim {counter}: finished ticking");

    let mut stats = Aggregator::with_metrics(metrics);
    stats.calculate(sim.requests());

    println!("Sim {counter} {:?}\n{}", sim.running(), stats);
    Ok(())
}

fn main() {
    match try_main() {
        Ok(_) => {
            std::process::exit(exitcode::OK);
        }
        Err(err) => {
            error!(target: "main", "{err}");
            std::process::exit(exitcode::USAGE);
        }
    }
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let log_level = log_level(args.log_level);
    simple_logger::init_with_level(log_level)?;

    let config_path = args.config_path.unwrap();
    let config = Config::try_from(&config_path)?.parsed()?;

    // We want to pin some cores, but not all the cores
    let sim_threads = available_parallelism()?.get() - 1;

    //Setup a fairly safe thread pool.
    rayon::ThreadPoolBuilder::new()
        .num_threads(sim_threads)
        .build_global()?;
    debug!(target: "main", "setting rayon to use {sim_threads} threads");

    trace!(target: "main", "config: {config:?}");
    let metrics = config.metrics();

    config
        .into_par_iter()
        .map(|(index, config)| {
            // Simulation config is cloned for each run since these are consumed by each simulation
            // which is done to ensure data encapsulation. Trade off is memory footprint, which is
            // rather small for these sims.
            run_sim(index, config.get(index), &metrics)
        })
        .collect::<Result<Vec<()>, SimulationError>>()?;

    Ok(())
}
