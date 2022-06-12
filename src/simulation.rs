//use contact::Contact;
use crate::ContactCenter;

use crate::{Action, ActionQueue, Contact, ContactQueue};

const TICKS_PER_SECOND: usize = 1000;
const ONE_HOUR: usize = TICKS_PER_SECOND * 60 * 60;

//tickSize = number of milliseconds
pub struct Simulation {
    run_until: usize,
    ticks: usize,
    tick_size: usize,
    running: bool,
    contact_center: ContactCenter,
    contacts: Vec<Contact>,
    queued_actions: ActionQueue,
    queued_contacts: ContactQueue,
    available_agents: Vec<usize>,
}

// constructors
impl Simulation {
    pub fn new(contact_center: ContactCenter) -> Self {
        let available_agents = contact_center
            .agents()
            .map(|agent| agent.id)
            .collect::<Vec<usize>>();

        Simulation {
            run_until: ONE_HOUR,
            ticks: 0,
            tick_size: 1000,
            running: true,
            contact_center,
            contacts: vec![],
            queued_actions: ActionQueue::new(),
            queued_contacts: ContactQueue::new(),
            available_agents,
        }
    }
}

// impls
impl Simulation {
    /// This is the main logic of the simulation
    ///
    /// Returns a true when the simulation is still ticking
    ///
    /// Actions:
    ///   - Process tick related actions
    ///     - Release the agents that have finished their last contact
    ///   - Increment the tick on the existing waiting contacts
    ///   - Roll the dice to see if a new contact should be added to the queue
    ///   - Assign the waiting calls to available agents
    ///
    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }

        self.process_actions();
        // pop new call?
        // allocate_agents

        self.increment_tick()
    }

    fn process_actions(&mut self) {
        while let Some(action) = self.queued_actions.pop(self.ticks) {
            match action {
                Action::ReleaseAgent(id) => { self.available_agents.push(id); }
            }
        }
    }

    fn increment_tick(&mut self) -> bool {
        self.ticks += self.tick_size;

        if self.ticks >= self.run_until {
            self.ticks = self.run_until;
            self.running = false
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn simulation() -> Simulation {
        let contact_center = ContactCenter::new();
        Simulation::new(contact_center)
    }

    #[test]
    fn ticks() {
        let mut sim = simulation();
        assert_eq!(0, sim.ticks);
        sim.tick();
        assert_eq!(1000, sim.ticks);
    }
    
    #[test]
    fn ticks_until_completion() {
        let mut sim = simulation();
        assert_eq!(0, sim.ticks);
        while sim.tick() {}
        assert_eq!(ONE_HOUR, sim.ticks);
    }
}
