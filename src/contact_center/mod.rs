
mod agent;
pub mod skill;

pub use agent::Agent;

#[derive(Default, Clone)]
pub struct ContactCenter {
    agents: Vec<Agent>,
}

impl ContactCenter {
    pub fn new() -> Self {
        Self { agents: vec![] }
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
    }

    /// Remove agent by agent id
    pub fn remove_agent(&mut self, id: usize) {
        self.agents.retain(|agent| agent.id != id)
    }
    
    pub fn agents(&mut self) -> impl Iterator<Item = &mut Agent> {
        self.agents.iter_mut()
    }
}
