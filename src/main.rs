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

mod min_queue;
mod simulation;

use min_queue::MinQueue;
use simulation::{ClientProfile, Server, Simulation};

use std::sync::Arc;

use std::thread;

const N_THREADS: usize = 10_000;

fn main() {
    let server = Arc::new(Server::default());
    let mut profiles = vec![];
    for _ in 0..100 {
        profiles.push(Arc::new(ClientProfile::default()));
    }

    let mut children = vec![];
    for i in 0..N_THREADS {
        let server = server.clone();
        let profiles = profiles.clone();
        children.push(thread::spawn(move || {
            let mut sim = Simulation::default();
            sim.add_server(server);

            for profile in profiles {
                sim.add_client_profile(profile);
            }

            sim.enable();

            while sim.tick() {}

            println!("Thread {i}\n{sim:?}");
        }));
    }

    for child in children {
        let _ = child.join();
    }
}
