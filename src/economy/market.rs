use std::collections::HashMap;

use super::*;
use astronomicals::System;
use utils::Point;

/// Controls an economic market, i.e a sector of trading systems.
#[derive(Serialize, Deserialize)]
pub struct Market {
    agents: HashMap<Point, Agent>,
}

impl Market {
    /// Creates a new empty market.
    pub fn new() -> Self {
        Market {
            agents: HashMap::new(),
        }
    }

    /// Returns the agent, if any, which is associated with the given system.
    pub fn agent(&self, location: &Point) -> Option<&Agent> {
        self.agents.get(location)
    }

    /// Adds the given system to this market.
    pub fn add_system(&mut self, system: &System) {
        self.agents.insert(system.location, Agent::new(&system));
    }
}

impl Updatable for Market {
    /// Update all agents in this market generate and solve trade for this round
    fn update(&mut self) {
        unimplemented!();
    }
}
