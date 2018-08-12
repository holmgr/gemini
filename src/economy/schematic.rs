use super::*;
use std::collections::HashMap;

use astronomicals::planet::PlanetEconomy;

/// Describes a formula which turns a set of commodities into another.
/// Requires that the given planet has the required economy.
#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct Schematic {
    pub required_env: PlanetEconomy,
    pub import: Vec<(Commodity, u64)>,
    pub export: Vec<(Commodity, u64)>,
}

impl Schematic {
    pub fn profit(&self, supply_demand: &HashMap<Commodity, f64>) -> f64 {
        self.export.iter().fold(0., |acc, (commodity, amount)| {
            acc + (*amount as f64) * supply_demand.get(commodity).unwrap()
        }) + self.import.iter().fold(0., |acc, (commodity, amount)| {
            acc - (*amount as f64) * supply_demand.get(commodity).unwrap()
        })
    }
}
