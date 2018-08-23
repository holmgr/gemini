use rand::Rng;

use super::*;
use astronomicals::planet::{Planet, PlanetEconomy};
use resources::{fetch_resource, SchematicResource};

/// Describes a formula which turns a set of commodities into another.
/// Requires that the given planet has the a specific economy.
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Schematic {
    pub required_env: PlanetEconomy,
    pub import: Vec<(Commodity, u64)>,
    pub export: Vec<(Commodity, u64)>,
}

impl Schematic {
    /// Loads a random sample of schematics which are compatable with the planet
    /// economy type.
    pub fn get_all<R: Rng>(rng: &mut R, planet: &Planet) -> Vec<Self> {
        let mut schematics = fetch_resource::<SchematicResource>().unwrap().schematics;
        rng.shuffle(&mut schematics);
        schematics
            .into_iter()
            .filter(|schematic| schematic.required_env == planet.economic_type)
            .enumerate()
            .filter_map(|(index, schematic)| {
                if rng.gen_weighted_bool(index as u32) {
                    Some(schematic)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}
