use rand::{ChaChaRng, SeedableRng};
use statrs::distribution::{Continuous, DiscreteUniform, Distribution, Gamma};
use std::{collections::HashMap, iter::repeat, ops::Range};

use super::*;
use astronomicals::{planet::PlanetType, system::System};
use resources::{fetch_resource, AgentResource};

/// Economic agent, able to take part in transactions.
#[derive(Serialize, Deserialize, Debug)]
pub struct Agent {
    seed: u32,
    credits: u64,
    populations: Vec<f64>,
    inventory: HashMap<Commodity, u64>,
    ideals: Vec<HashMap<Commodity, u64>>,
    productions: Vec<HashMap<Commodity, u64>>,
    price_beliefs: HashMap<Commodity, Range<i64>>,
}

impl Agent {
    // TODO: Move to config?
    const STARTING_CREDITS: u64 = 100_000;
    const INITIAL_LOWER_BELIEF: i64 = 1000;
    const INITIAL_UPPER_BELIEF: i64 = 10000;
    const DEFAULT_PRICE: i64 = (Agent::INITIAL_LOWER_BELIEF + Agent::INITIAL_UPPER_BELIEF) / 2;
    const MIN_PRICE: i64 = 10;
    const POPULATION_FACTOR: f64 = 1.;

    /// Create a new economic agent for the given system.
    pub fn new(system: &System) -> Self {
        let resource = fetch_resource::<AgentResource>().unwrap();

        // Setup ideal consumption.
        let ideals = system.satelites.iter().fold(vec![], |mut ideals, planet| {
            ideals.push(
                Commodity::values()
                    .map(|commodity| {
                        let planet_ideal = *resource.faction_ideals[&system.faction]
                            .get(commodity)
                            .unwrap_or(&0)
                            + *resource.planet_ideals[&planet.economic_type]
                                .get(commodity)
                                .unwrap_or(&0);
                        (commodity.clone(), planet_ideal)
                    })
                    .collect(),
            );
            ideals
        });

        // Setup system commodity production.
        let productions = system
            .satelites
            .iter()
            .fold(vec![], |mut productions, planet| {
                productions.push(
                    Commodity::values()
                        .map(|commodity| {
                            let planet_production = *resource.faction_production[&system.faction]
                                .get(commodity)
                                .unwrap_or(&0)
                                + *resource.planet_production[&planet.economic_type]
                                    .get(commodity)
                                    .unwrap_or(&0);
                            (commodity.clone(), planet_production)
                        })
                        .collect(),
                );
                productions
            });

        // Create initial price beliefs.
        let price_beliefs = Commodity::values()
            .map(|commodity| {
                (
                    commodity.clone(),
                    Agent::INITIAL_LOWER_BELIEF..Agent::INITIAL_UPPER_BELIEF,
                )
            })
            .collect();

        Agent {
            seed: system.location.hash() as u32,
            populations: system
                .satelites
                .iter()
                .map(|planet| Agent::initial_population(planet.mass, &planet.planet_type))
                .collect(),
            credits: Agent::STARTING_CREDITS,
            inventory: HashMap::new(),
            ideals,
            productions,
            price_beliefs,
        }
    }

    /// Calculates the initial planet population based on mass and planet type.
    fn initial_population(mass: f64, kind: &PlanetType) -> f64 {
        let mass_factor = Gamma::new(7., 5.).unwrap();
        let type_factor: f64 = match *kind {
            PlanetType::Metal => 150.,
            PlanetType::Earth => 800.0,
            PlanetType::Rocky => 1.,
            PlanetType::Icy => 0.5,
            PlanetType::GasGiant => 0.,
        };
        mass_factor.pdf(mass) * type_factor * 100.
    }

    /// Returns the hash of the system to which the agent is associated.
    #[allow(dead_code)]
    pub fn hash(&self) -> u32 {
        self.seed
    }

    /// Returns the system populations.
    #[allow(dead_code)]
    pub fn populations(&self) -> Vec<f64> {
        self.populations.clone()
    }

    /// Returns the prices for all commodities known.
    #[allow(dead_code)]
    pub fn prices(&self) -> Vec<(Commodity, i64)> {
        Commodity::values().fold(vec![], |mut prices, commodity| {
            if let Some(range) = self.price_beliefs.get(commodity) {
                let price = (range.start + range.end) / 2;
                if price != Agent::DEFAULT_PRICE {
                    prices.push((commodity.clone(), (range.start + range.end) / 2));
                }
            }

            prices
        })
    }

    /// Returns the balance of a given commodity compared to the ideal amount currently in inventory.
    fn balance(&self, commodity: &Commodity) -> i64 {
        let current_stock = self.inventory.get(commodity).unwrap_or(&0);
        let ideal_stock = self
            .ideals
            .iter()
            .enumerate()
            .fold(0, |acc, (index, ideal)| {
                acc + (self.populations[index]
                    * Agent::POPULATION_FACTOR
                    * *ideal.get(commodity).unwrap_or(&0) as f64) as i64
            });
        (*current_stock as i64) - (ideal_stock as i64)
    }

    /// Adds delta amount of the given commodity to the inventory.
    pub fn update_inventory(&mut self, commodity: &Commodity, delta: i64) {
        let current_stock = self.inventory.entry(commodity.clone()).or_insert(0);
        *current_stock = (*current_stock as i64 + delta).max(0) as u64;
    }

    /// Adds delta amount of credits.
    pub fn update_credits(&mut self, delta: i64) {
        self.credits = (self.credits as i64 + delta as i64) as u64;
    }

    /// Updates the price beliefs for the given commodity based on the given unit price.
    /// Success indicates whether the agent was successful in trading.
    /// Successful trades will strengthen the belief whereas unsucessful trades will weaken the belief.
    pub fn update_price_belief(&mut self, commodity: &Commodity, unit_price: u64, success: bool) {
        let price_belief = self.price_beliefs.get_mut(commodity).unwrap();
        let curr_mean = (price_belief.start + price_belief.end) / 2;

        // Translate mean towards unit_price by 10%.
        let new_mean = curr_mean + ((unit_price as f64 - curr_mean as f64) * 1.1) as i64;

        price_belief.end += new_mean - curr_mean;
        price_belief.start -= curr_mean - new_mean;

        if success {
            price_belief.start = (price_belief.start as f64 * 1.05) as i64;
            price_belief.end = (price_belief.end as f64 * 0.95) as i64;
        } else {
            price_belief.start = (price_belief.start as f64 * 0.95) as i64;
            price_belief.end = (price_belief.end as f64 * 1.05) as i64;
        }

        // Ensure reasonable price beliefs.
        price_belief.start = price_belief
            .start
            .max(Agent::MIN_PRICE)
            .min(price_belief.end);
        price_belief.end = price_belief.end.max(price_belief.start + 1);
    }

    /// Generate a bid for the given commodity if there exists a demand.
    pub fn generate_bid(&self, commodity: &Commodity) -> Option<BidBuilder> {
        let balance = self.balance(commodity);

        // Place bid if we have demand.
        if balance < 0 {
            let seed: &[u32] = &[self.seed];
            let mut rng: ChaChaRng = SeedableRng::from_seed(seed);

            let price_belief = &self.price_beliefs[commodity];

            let price_model = DiscreteUniform::new(price_belief.start, price_belief.end).unwrap();
            let mut partial_bid = BidBuilder::default();
            partial_bid
                .commodity(commodity.clone())
                .amount((-balance) as u64)
                .unit_price((price_model.sample(&mut rng) as u64).min(self.credits));
            Some(partial_bid)
        } else {
            None
        }
    }

    /// Generate an ask for the given commodity if there exists surplus.
    pub fn generate_ask(&self, commodity: &Commodity) -> Option<AskBuilder> {
        let balance = self.balance(commodity);

        // Place ask if we have excess.
        if balance > 0 {
            let seed: &[u32] = &[self.seed];
            let mut rng: ChaChaRng = SeedableRng::from_seed(seed);

            let price_belief = &self.price_beliefs[commodity];

            let price_model = DiscreteUniform::new(price_belief.start, price_belief.end).unwrap();
            let mut partial_ask = AskBuilder::default();
            partial_ask
                .commodity(commodity.clone())
                .amount(balance as u64)
                .unit_price(price_model.sample(&mut rng) as u64);
            Some(partial_ask)
        } else {
            None
        }
    }

    /// Updates the planet populations held by the agent based on potential economic growth.
    pub fn update_population(&mut self, demand_supply: &[(Commodity, u64, u64)]) {
        for (index, population) in self.populations.iter_mut().enumerate() {
            let productions = &self.productions[index];
            let potential_earnings =
                demand_supply
                    .iter()
                    .fold(0, |acc, (commodity, demand, supply)| {
                        match productions.get(commodity) {
                            Some(0) | None => acc,
                            Some(prod) => acc + ((demand - supply) as i64) / (*prod as i64),
                        }
                    });
            *population += match potential_earnings.signum() {
                1 => (*population * 0.1).min(10.),
                -1 => (*population * -0.1).max(-10.).max(-*population),
                _ => 0.,
            };
        }
    }
}

impl Updatable for Agent {
    /// Updates the inventory based on the consumption and production.
    fn update(&mut self) {
        for (index, (commodity, amount)) in self
            .productions
            .clone()
            .iter()
            .enumerate()
            .flat_map(|(index, production)| repeat(index).zip(production.iter()))
        {
            let population = self.populations[index];
            self.update_inventory(
                commodity,
                (*amount as f64 * population * Agent::POPULATION_FACTOR) as i64,
            );
        }
        for (index, (commodity, amount)) in self
            .ideals
            .clone()
            .iter()
            .enumerate()
            .flat_map(|(index, ideal)| repeat(index).zip(ideal.iter()))
        {
            let population = self.populations[index];
            self.update_inventory(
                commodity,
                -(*amount as f64 * population * Agent::POPULATION_FACTOR) as i64,
            );
        }
    }
}
