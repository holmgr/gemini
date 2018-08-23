use std::{collections::HashMap, iter::FromIterator};

use super::*;
use astronomicals::System;
use utils::Point;

static MAX_TRADE_DISTANCE: f64 = 20.;

/// Controls an economic market, i.e a sector of trading systems.
#[derive(Serialize, Deserialize)]
pub struct Market {
    agents: HashMap<Point, Agent>,
    neighbors: HashMap<Point, Vec<Point>>,
}

impl Market {
    /// Creates a market with the given systems and population.
    pub fn new(systems: Vec<&System>, population: u64) -> Self {
        // Build an adjecency list of all neighboring systems needed for trade resolving.
        let mut neighbors = HashMap::new();
        for i in 0..systems.len() {
            let mut point_neighbors = vec![];
            for j in 0..systems.len() {
                if i != j && systems[i].location.distance(&systems[j].location) < MAX_TRADE_DISTANCE
                {
                    point_neighbors.push(systems[j].location);
                }
            }

            neighbors.insert(systems[i].location, point_neighbors);
        }

        let system_count = systems.len() as u64;
        let agents = HashMap::from_iter(systems.into_iter().map(|system| {
            (
                system.location,
                Agent::new(system, population / system_count),
            )
        }));
        Market { agents, neighbors }
    }

    /// Returns the agent, if any, which is associated with the given system.
    pub fn agent(&self, location: &Point) -> Option<&Agent> {
        self.agents.get(location)
    }
}

impl Updatable for Market {
    /// Update all agents in this market generate and solve trade for this round
    fn update(&mut self) {
        unimplemented!();
    }
}
