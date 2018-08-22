use rayon::prelude::*;
use std::{fmt, slice::Iter};

use astronomicals::{system::System, Galaxy};
use game::Updatable;

mod agent;
mod market;
pub mod schematic;

use self::agent::Agent;
use self::market::Market;
pub use self::schematic::Schematic;

/// The total amount of workers in the galaxy at any point in time.
static GALAXY_WORKER_POPULATION: u64 = 1_000_000_000_000;

/// Holds the economic state for the entire game.
#[derive(Default, Serialize, Deserialize)]
pub struct Economy {
    markets: Vec<Market>,
}

impl Economy {
    /// Creates the game economy using the given galaxy.
    pub fn new(galaxy: &Galaxy) -> Economy {
        // Create one market per sector.
        let mut markets = vec![];
        for sector in &galaxy.sectors {
            let systems = sector
                .system_locations
                .iter()
                .map(|loc| galaxy.system(loc).unwrap())
                .collect::<Vec<_>>();

            // Assign an equal amount of workers to each sector.
            let market = Market::new(
                systems,
                GALAXY_WORKER_POPULATION / galaxy.sectors.len() as u64,
            );
            markets.push(market);
        }

        Economy { markets }
    }

    /// Returns the prices for the available commodities the the given system.
    pub fn commodity_prices(&self, system: &System) -> Vec<(Commodity, i64)> {
        // TODO: Implement calculation of local market prices for all commodities
        vec![]
    }
}

impl Updatable for Economy {
    /// Advances time and updates all agents etc.
    fn update(&mut self) {
        self.markets.par_iter_mut().for_each(|market| {
            market.update();
        });
    }
}

/// A tradable and possibly producable commodity
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Commodity {
    Chemical,
    ConsumerItem,
    Food,
    IllegalDrug,
    IndustrialMaterial,
    LegalDrug,
    Machinery,
    Medicine,
    Metal,
    Mineral,
    Salvage,
    Slavery,
    Technology,
    Textile,
    Waste,
    Weapon,
}

impl Commodity {
    pub fn values() -> Iter<'static, Commodity> {
        static COMMODITIES: [Commodity; 16] = [
            Commodity::Chemical,
            Commodity::ConsumerItem,
            Commodity::Food,
            Commodity::IllegalDrug,
            Commodity::IndustrialMaterial,
            Commodity::LegalDrug,
            Commodity::Machinery,
            Commodity::Medicine,
            Commodity::Metal,
            Commodity::Mineral,
            Commodity::Salvage,
            Commodity::Slavery,
            Commodity::Technology,
            Commodity::Textile,
            Commodity::Waste,
            Commodity::Weapon,
        ];
        COMMODITIES.into_iter()
    }
}

impl fmt::Display for Commodity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Commodity::Chemical => "Chemicals",
                Commodity::ConsumerItem => "Consumer Items",
                Commodity::Food => "Food",
                Commodity::IllegalDrug => "Illegal Drugs",
                Commodity::IndustrialMaterial => "Industrial Materials",
                Commodity::LegalDrug => "Legal Drugs",
                Commodity::Machinery => "Machinery",
                Commodity::Medicine => "Medicine",
                Commodity::Metal => "Metals",
                Commodity::Mineral => "Minerals",
                Commodity::Salvage => "Salvage",
                Commodity::Slavery => "Slaves",
                Commodity::Technology => "Technology",
                Commodity::Textile => "Textiles",
                Commodity::Waste => "Waste",
                Commodity::Weapon => "Weapons",
            }
        )
    }
}
