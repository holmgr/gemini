use std::{collections::HashMap, iter::FromIterator, ops::Range};
use rand::{ChaChaRng, SeedableRng};

use statrs::distribution::{DiscreteUniform, Distribution};

use super::*;
use astronomicals::{hash, system::System};
use resources::{fetch_resource, AgentResource};

/// Economic agent, able to take part in transactions.
#[derive(Serialize, Deserialize, Debug)]
pub struct Agent {
    seed: u32,
    credits: u32,
    inventory: HashMap<Commodity, u32>,
    ideal: HashMap<Commodity, u32>,
    production: HashMap<Commodity, u32>,
    price_beliefs: HashMap<Commodity, Range<i64>>,
}

impl Agent {
    // TODO: Move to config?
    const STARTING_CREDITS: u32 = 100_000_000;
    const INITIAL_LOWER_BELIEF: i64 = 1;
    const INITIAL_UPPER_BELIEF: i64 = 1000;

    /// Create a new economic agent for the given system.
    pub fn new(system: &System) -> Self {
        let resource = fetch_resource::<AgentResource>().unwrap();

        // Setup ideal consumption.
        let ideals: HashMap<Commodity, u32> = Commodity::values()
            .map(|commodity| {
                let mut ideal = 0;

                for planet in &system.satelites {
                    let population_factor = planet.population;
                    let planet_ideal = *resource.faction_ideals[&system.faction]
                        .get(&commodity)
                        .unwrap_or(&0)
                        + *resource.planet_ideals[&planet.economic_type]
                            .get(&commodity)
                            .unwrap_or(&0);
                    ideal += (f64::from(planet_ideal) * population_factor) as u32;
                }
                (commodity.clone(), ideal)
            })
            .collect();

        // Setup system commodity production.
        let productions: HashMap<Commodity, u32> = Commodity::values()
            .map(|commodity| {
                let mut production = 0;

                for planet in &system.satelites {
                    let population_factor = planet.population;
                    let planet_production = *resource.faction_production[&system.faction]
                        .get(&commodity)
                        .unwrap_or(&0)
                        + *resource.planet_production[&planet.economic_type]
                            .get(&commodity)
                            .unwrap_or(&0);
                    production += (f64::from(planet_production) * population_factor) as u32;
                }
                (commodity.clone(), production)
            })
            .collect();

        // Create initial price beliefs.
        let price_beliefs = HashMap::from_iter(Commodity::values().map(|commodity| {
            (
                commodity.clone(),
                Agent::INITIAL_LOWER_BELIEF..Agent::INITIAL_UPPER_BELIEF,
            )
        }));

        Agent {
            seed: hash(&system.location) as u32,
            credits: Agent::STARTING_CREDITS,
            inventory: HashMap::new(),
            ideal: ideals,
            production: productions,
            price_beliefs,
        }
    }

    /// Returns the balance of a given commodity compared to the ideal amount currently in inventory.
    fn balance(&self, commodity: &Commodity) -> i32 {
        let current_stock = self.inventory.get(commodity).unwrap_or(&0);
        let ideal_stock = self.ideal.get(commodity).unwrap_or(&0);
        (*current_stock as i32) - (*ideal_stock as i32)
    }

    /// Adds delta amount of the given commodity to the inventory.
    pub fn update_inventory(&mut self, commodity: &Commodity, delta: i32) {
        let current_stock = self.inventory.entry(commodity.clone()).or_insert(0);
        *current_stock = (*current_stock as i32 + delta).max(0) as u32;
    }

    /// Adds delta amount of credits.
    pub fn update_credits(&mut self, delta: i32) {
        self.credits = (self.credits as i32 + delta) as u32;
    }

    /// Updates the price beliefs for the given commodity based on the given unit price.
    /// Success indicates whether the agent was successful in trading.
    /// Successful trades will strengthen the belief whereas unsucessful trades will weaken the belief.
    pub fn update_price_belief(&mut self, commodity: &Commodity, unit_price: u32, success: bool) {
        let price_belief = self.price_beliefs.get_mut(commodity).unwrap();
        let curr_mean = (price_belief.start + price_belief.end) / 2;

        // Translate mean towards unit_price by 10%.
        let new_mean = curr_mean + (i64::from(unit_price) - curr_mean) / 10;

        price_belief.end += new_mean - curr_mean;
        price_belief.start -= curr_mean - new_mean;

        // If successful, compress around mean by 10%, else expand by 10%.
        if success {
            price_belief.start += (curr_mean - price_belief.start) / 10;
            price_belief.end += (price_belief.end + curr_mean) / 10;
        } else {
            price_belief.start -= ((curr_mean - price_belief.start) / 10).min(price_belief.start);
            price_belief.end -= (price_belief.end - curr_mean) / 10;
        }

        // Ensure reasonable price beliefs.
        price_belief.start = price_belief.start.max(0).min(price_belief.end);
    }

    /// Generate a bid for the given commodity if there exists a demand.
    pub fn generate_bid(&self, commodity: &Commodity) -> Option<BidBuilder> {
        let balance = self.balance(commodity);

        // Place bid if we have demand.
        if balance < 0 {
            let seed: &[u32] = &[self.seed];
            let mut rng: ChaChaRng = SeedableRng::from_seed(seed);

            let price_belief = &self.price_beliefs[&commodity];

            let price_model = DiscreteUniform::new(price_belief.start, price_belief.end).unwrap();
            let mut partial_bid = BidBuilder::default();
            partial_bid
                .commodity(commodity.clone())
                .amount((-balance) as u32)
                .unit_price(price_model.sample(&mut rng) as u32);
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

            let price_belief = &self.price_beliefs[&commodity];

            let price_model = DiscreteUniform::new(price_belief.start, price_belief.end).unwrap();
            let mut partial_ask = AskBuilder::default();
            partial_ask
                .commodity(commodity.clone())
                .amount(balance as u32)
                .unit_price(price_model.sample(&mut rng) as u32);
            Some(partial_ask)
        } else {
            None
        }
    }
}

impl Updatable for Agent {
    /// Updates the inventory based on the consumption and production.
    fn update(&mut self) {
        for (commodity, amount) in self.production.clone() {
            self.update_inventory(&commodity, amount as i32);
        }
        for (commodity, amount) in self.ideal.clone() {
            self.update_inventory(&commodity, -(amount as i32));
        }
    }
}
