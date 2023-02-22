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
#![feature(let_chains)]

mod attribute;
mod client;
mod client_profile;
mod server;
mod simulation;

use attribute::Attribute;
use client::Client;
use client_profile::ClientProfile;
use server::Server;
use simulation::{Simulation, TICKS_PER_SECOND};

use std::sync::Arc;

fn main() {
    let mut sim = Simulation::default();
    sim.add_server(Arc::new(Server::default()));

    let mut profiles = vec![];
    for _ in 0..100 {
        profiles.push(Arc::new(ClientProfile::default()));
    }
    
    for profile in profiles.iter() {
        sim.add_client_profile(profile);
    }

    sim.enable();

    while sim.tick() {}
}
