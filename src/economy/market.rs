use std::{collections::HashMap, iter::FromIterator};

use super::*;
use astronomicals::System;
use utils::Point;

/// Controls an economic market, i.e a sector of trading systems.
#[derive(Serialize, Deserialize)]
pub struct Market {
    agents: HashMap<Point, Agent>,
}

impl Market {
    /// Creates a market with the given systems and population.
    pub fn new(systems: Vec<&System>, population: u64) -> Self {
        let system_count = systems.len() as u64;
        let agents = HashMap::from_iter(systems.into_iter().map(|system| {
            (
                system.location,
                Agent::new(system, population / system_count),
            )
        }));
        Market { agents }
    }

    /// Returns the agent, if any, which is associated with the given system.
    pub fn agent(&self, location: &Point) -> Option<&Agent> {
        self.agents.get(location)
    }
}

impl Updatable for Market {
    /// Update all agents in this market generate and solve trade for this round
    fn update(&mut self) {
        unimplemented!();
    }
}
