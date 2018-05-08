use std::collections::HashMap;

use astronomicals::system::System;

use super::*;

/// Controls an economic market, i.e a sector of trading systems.
#[derive(Serialize, Deserialize)]
pub struct Market {
    average_prices: HashMap<Commodity, u32>,
    agents: Vec<Arc<Mutex<Agent>>>,
}

impl Market {
    /// Creates a new empty market.
    pub fn new() -> Self {
        let average_prices: HashMap<Commodity, u32> = Commodity::values()
            .map(|commodity| (commodity.clone(), 1000))
            .collect();

        Market {
            average_prices,
            agents: vec![],
        }
    }

    /// Adds the given system to this market.
    pub fn add_system(&mut self, system: &System) {
        self.agents.push(Arc::new(Mutex::new(Agent::new(system))));
    }

    /// Attemps to resolve the bids and asks for the given commodity by matching
    /// the highest asks with the lowest bids performing the transaction.
    /// Returns the quantity traded.
    fn resolve_offers(
        &mut self,
        commodity: &Commodity,
        mut bids: Vec<Bid>,
        mut asks: Vec<Ask>,
    ) -> u32 {
        bids.sort_unstable_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
        asks.sort_unstable_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());

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
                buyer.update_credits(-(money_traded as i32));
                seller.update_credits(money_traded as i32);

                // Transfer goods.
                buyer.update_inventory(&commodity, quantity_traded as i32);
                seller.update_inventory(&commodity, -(quantity_traded as i32));

                // Update agent price beliefs on success
                buyer.update_price_belief(&commodity, clearing_price, true);
                seller.update_price_belief(&commodity, clearing_price, true);
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
            let average_price = self.average_prices.get_mut(&commodity).unwrap();
            *average_price = money_traded / amount_traded;
        }

        let average_price = self.average_prices[&commodity];

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

        amount_traded
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

        // Until we no longer make any transaction we simulate trading.
        let mut trades_made = true;
        while trades_made {
            trades_made = false;
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

                // If we managed to resolve any offers, we need to simulate one more round.
                let amount_traded = self.resolve_offers(commodity, bids, asks);
                if amount_traded > 0 {
                    trades_made = true;
                }
            }
        }
    }
}
