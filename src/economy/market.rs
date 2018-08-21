use astronomicals::System;

use super::*;

/// Controls an economic market, i.e a sector of trading systems.
#[derive(Serialize, Deserialize)]
pub struct Market {
    agents: Vec<Agent>,
}

impl Market {
    /// Creates a new empty market.
    pub fn new() -> Self {
        Market { agents: vec![] }
    }

    /// Returns the agent, if any, which is associated with the given system.
    pub fn agent(&self, system_hash: u32) -> Option<Agent> {
        unimplemented!();
    }

    /// Adds the given system to this market.
    pub fn add_system(&mut self, system: &System) {
        self.agents.push(Agent::new(&system));
    }
}

impl Updatable for Market {
    /// Update all agents in this market generate and solve trade for this round
    fn update(&mut self) {
        unimplemented!();
    }
}
