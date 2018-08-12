use std::collections::HashMap;
use rand::{Rng, ChaChaRng, SeedableRng};

use super::*;
use astronomicals::{planet::Planet, system::System};
use resources::{fetch_resource, SchematicResource};
use utils::Point;


/// Economic agent, able to take part in transactions.
#[derive(Serialize, Deserialize, Debug)]
pub struct Agent {
    location: Point,
    workers: u64,
    productions: Vec<Schematic>
}

impl Agent {
    /// Create a new economic agent for the given planet in the given system.
    pub fn new(system: &System, planet: &Planet) -> Self {
        let seed: &[_] = &[u64::from(planet.id) as u32];
        let mut rng = ChaChaRng::from_seed(seed);

        // Load all schematics which are compatable with the planet economy type.
        // Shuffle to get different schematics for different planets.
        let mut schematics = fetch_resource::<SchematicResource>().unwrap().schematics;
        rng.shuffle(&mut schematics);

       let productions = schematics
            .into_iter()
            .filter(|schematic| {
                // TODO: Maybe the economic type for a given planet should be moved into the agent instead?
                schematic.required_env == planet.economic_type
            })
            .enumerate().filter_map(|(index, schematic)| {
                if rng.gen_weighted_bool(index as u32) {
                    Some(schematic)
                }
                else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Agent {
            location: system.location,
            workers: 0,
            productions,
        }
    }

    pub fn workers_mut(&mut self) -> &mut u64 {
        &mut self.workers
    }

    pub fn workers(&self) -> u64 {
        self.workers
    }

    pub fn location(&self) -> Point {
        self.location
    }
}
