use std::{collections::HashMap, iter::FromIterator};

use astronomicals::System;

use super::*;

/// Controls an economic market, i.e a sector of trading systems.
#[derive(Serialize, Deserialize)]
pub struct Market {
    sdm: Sdm,
    agents: HashMap<PlanetID, Agent>,
}

impl Market {
    /// Creates a new market with the given population and systems.
    pub fn new<'a, I>(population: u64, systems: I) -> Self
    where
        I: IntoIterator<Item = &'a System>,
    {
        // Create price map.
        let mut sdm = Sdm::new(RBF::Quadratic);

        // Create initial agents.
        let mut agents = HashMap::new();
        for system in systems {
            sdm.add_point(system.location);
            agents.extend(
                system
                    .satelites
                    .iter()
                    .map(|planet| (planet.id, Agent::new(system, planet))),
            );
        }

        // Assign initial worker population to each planet uniformly.
        let planet_population = population / agents.len() as u64;
        for agent in agents.values_mut() {
            *agent.workers_mut() += planet_population;
        }

        Market { sdm, agents }
    }
}

impl Updatable for Market {
    /// Update all agents in this market and resolve commodity prices.
    fn update(&mut self) {
        // Update demand/supply
        self.sdm.reset();
        for agent in self.agents.values_mut() {
            // TODO: Update price map etc.
            /*
            let exports = agent.produce();
            let imports = agent.consume();
            self.sdm.update(agent.location(), &exports);
            self.sdm.update(agent.location(), &imports);
            */
        }

        // TODO: Handle worker migration.
    }
}
