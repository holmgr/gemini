use std::collections::HashMap;

use crate::entities::System;

use super::*;

/// Controls an economic market, i.e a sector of trading systems.
#[derive(Serialize, Deserialize)]
pub struct Market {
    average_prices: HashMap<Commodity, u64>,
    agents: Vec<Arc<Mutex<Agent>>>,
}

impl Market {
    /// Creates a new empty market.
    pub fn new() -> Self {
        let average_prices: HashMap<Commodity, u64> = Commodity::values()
            .map(|commodity| (commodity.clone(), 1000))
            .collect();

        Market {
            average_prices,
            agents: vec![],
        }
    }

    /// Returns the agent, if any, which is associated with the given system.
    #[allow(dead_code)]
    pub fn agent(&self, system_hash: u32) -> Option<&Arc<Mutex<Agent>>> {
        self.agents
            .iter()
            .find(|agent| agent.lock().unwrap().hash() == system_hash)
    }

    /// Returns a reference to all agents.
    #[allow(dead_code)]
    pub fn agents(&self) -> &Vec<Arc<Mutex<Agent>>> {
        &self.agents
    }

    /// Adds the given system to this market.
    pub fn add_system(&mut self, system: &System) {
        self.agents.push(Arc::new(Mutex::new(Agent::new(system))));
    }

    /// Attemps to resolve the bids and asks for the given commodity by matching
    /// the highest asks with the lowest bids performing the transaction.
    /// Returns the quantity traded.
    fn resolve_offers(&mut self, commodity: &Commodity, mut bids: Vec<Bid>, mut asks: Vec<Ask>) {
        bids.sort_unstable_by(|a, b| a.amount.partial_cmp(&b.unit_price).unwrap());
        asks.sort_unstable_by(|a, b| b.amount.partial_cmp(&a.unit_price).unwrap());

        let mut money_traded = 0;
        let mut amount_traded = 0;

        // Keep going until we are out of bids or asks.
        while !bids.is_empty() && !asks.is_empty() {
            let mut bid = bids.remove(0);
            let mut ask = asks.remove(0);

            let quantity_traded = bid.amount.min(ask.amount);
            let clearing_price = (bid.unit_price + ask.unit_price) / 2;

            if quantity_traded > 0 {
                bid.amount -= quantity_traded;
                ask.amount -= quantity_traded;

                let mut buyer = bid.agent.lock().unwrap();
                let mut seller = ask.agent.lock().unwrap();

                // Log stats
                money_traded += quantity_traded * clearing_price;
                amount_traded += quantity_traded;

                // Transfer money.
                buyer.update_credits(-(money_traded as i64));
                seller.update_credits(money_traded as i64);

                // Transfer goods.
                buyer.update_inventory(commodity, quantity_traded as i64);
                seller.update_inventory(commodity, -(quantity_traded as i64));

                // Update agent price beliefs on success
                buyer.update_price_belief(commodity, clearing_price, true);
                seller.update_price_belief(commodity, clearing_price, true);
            }
            // Remove bid or ask if the seller/buyer is out of need or stock
            if ask.amount > 0 {
                asks.insert(0, ask);
            }

            if bid.amount > 0 {
                bids.insert(0, bid);
            }
        }

        // Use previous average if no trades were made.
        if amount_traded > 0 {
            let average_price = self.average_prices.get_mut(commodity).unwrap();
            *average_price = money_traded / amount_traded;
        }

        let average_price = self.average_prices[commodity];

        // Update price beliefs for unsuccessful bids/asks.
        for bid in bids {
            bid.agent
                .lock()
                .unwrap()
                .update_price_belief(&bid.commodity, average_price, false);
        }
        for ask in asks {
            ask.agent
                .lock()
                .unwrap()
                .update_price_belief(&ask.commodity, average_price, false);
        }
    }
}

impl Updatable for Market {
    /// Update all agents in this market generate and solve transactions to update
    /// prices for commodities.
    fn update(&mut self) {
        // Make agents generate items for this simulation round.
        for agent in &self.agents {
            agent.lock().unwrap().update();
        }

        let mut supply = HashMap::new();
        let mut demand = HashMap::new();

        // Simulate trading.
        for commodity in Commodity::values() {
            // Gather bids/asks from agents.
            let bids = self.agents.iter().fold(vec![], |mut bids, agent| {
                if let Some(mut partial_bid) = agent.lock().unwrap().generate_bid(commodity) {
                    bids.push(partial_bid.agent(agent.clone()).build().unwrap());
                }
                bids
            });
            let asks = self.agents.iter().fold(vec![], |mut asks, agent| {
                if let Some(mut partial_ask) = agent.lock().unwrap().generate_ask(commodity) {
                    asks.push(partial_ask.agent(agent.clone()).build().unwrap());
                }
                asks
            });

            // Inform agents about supply and demand for this round.
            bids.iter().for_each(|bid| {
                let count = demand.entry(bid.commodity.clone()).or_insert(0);
                *count += bid.amount;
            });

            asks.iter().for_each(|ask| {
                let count = supply.entry(ask.commodity.clone()).or_insert(0);
                *count += ask.amount;
            });

            // If we managed to resolve any offers, we need to simulate one more round.
            self.resolve_offers(commodity, bids, asks);
        }

        // Update supply and demand.
        let demand_supply = Commodity::values()
            .map(|commodity| {
                (
                    commodity.clone(),
                    *demand.get(commodity).unwrap_or(&0) as u64,
                    *supply.get(commodity).unwrap_or(&0) as u64,
                )
            })
            .collect::<Vec<(Commodity, u64, u64)>>();

        for agent in &self.agents {
            agent.lock().unwrap().update_population(&demand_supply);
        }
    }
}
