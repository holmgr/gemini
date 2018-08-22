use rand::{ChaChaRng, SeedableRng};
use std::{collections::HashMap, iter::FromIterator};

use super::*;
use astronomicals::planet::PlanetID;
use utils::Point;

/// Represents a type of industry.
#[derive(Serialize, Deserialize, Clone, Debug)]
enum IndustryKind {
    Production(Schematic),
    Trade,
}

/// A planet side industry with assigned workers, importers and exporters.
#[derive(Serialize, Deserialize, Debug)]
pub struct Industry {
    importers: Vec<Point>,
    exporters: Vec<Point>,
    kind: IndustryKind,
    workers: u64,
}

/// Economic agent connected to a system, takes part in economy buy trading/producing.
#[derive(Serialize, Deserialize, Debug)]
pub struct Agent {
    industries: HashMap<PlanetID, Vec<Industry>>,
}

impl Agent {
    /// Create a new economic agent for the given system.
    pub fn new(system: &System, population: u64) -> Self {
        // Setup random gen to get random schematics for planets.
        let seed: &[_] = &[system.location.hash() as u32];
        let mut rng = ChaChaRng::from_seed(seed);

        let planet_count = system.satelites.len() as u64;
        let industries = HashMap::from_iter(system.satelites.iter().map(|planet| {
            let schematics = Schematic::get_all(&mut rng, planet);
            let schematic_count = schematics.len() as u64;
            let mut local_industries = schematics
                .into_iter()
                .map(|schematic| {
                    // Save 50% of population for trading.
                    Industry {
                        importers: vec![],
                        exporters: vec![],
                        kind: IndustryKind::Production(schematic),
                        workers: population / (2 * planet_count * schematic_count),
                    }
                })
                .collect::<Vec<_>>();

            // Add trade industry manually.
            local_industries.push(Industry {
                importers: vec![],
                exporters: vec![],
                kind: IndustryKind::Trade,
                workers: population / 2 * planet_count,
            });
            (planet.id, local_industries)
        }));
        Agent { industries }
    }
}
