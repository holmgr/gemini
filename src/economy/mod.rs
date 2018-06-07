use rayon::prelude::*;
use std::{fmt,
          slice::Iter,
          sync::{Arc, Mutex}};

use astronomicals::{hash, Galaxy, System};
use game::Updatable;

mod agent;
mod market;

use self::agent::Agent;
use self::market::Market;

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
            let mut market = Market::new();
            for system in sector
                .system_locations
                .iter()
                .map(|loc| galaxy.system(loc).unwrap())
            {
                market.add_system(system);
            }
            markets.push(market);
        }

        Economy { markets }
    }

    /// Returns the prices for the available commodities the the given system.
    pub fn commodity_prices(&self, system: &System) -> Vec<(Commodity, i64)> {
        let mut prices = vec![];

        let system_hash = hash(&system.location);
        for market in &self.markets {
            if let Some(agent) = market.agent(system_hash as u32) {
                prices = agent.lock().unwrap().prices();
                break;
            }
        }

        prices
    }

    pub fn populations(&self, system: &System) -> Vec<f64> {
        let mut populations = vec![];

        let system_hash = hash(&system.location);
        for market in &self.markets {
            if let Some(agent) = market.agent(system_hash as u32) {
                populations = agent.lock().unwrap().populations();
                break;
            }
        }

        populations
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

/// An offer to buy some commodity.
#[derive(Builder, Debug)]
pub struct Bid {
    pub agent: Arc<Mutex<Agent>>,
    pub commodity: Commodity,
    pub amount: u64,
    pub unit_price: u64,
}

/// An offer to sell some commodity.
#[derive(Builder, Debug)]
pub struct Ask {
    pub agent: Arc<Mutex<Agent>>,
    pub commodity: Commodity,
    pub amount: u64,
    pub unit_price: u64,
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
