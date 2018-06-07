use super::*;

use rayon::prelude::*;
use spade::rtree::RTree;
use std::{collections::{BinaryHeap, HashMap},
          u32::MAX};

/// A galaxy of systems.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Galaxy {
    pub sectors: Vec<sector::Sector>,
    pub map: RTree<HashablePoint>,
    pub systems: HashMap<HashablePoint, system::System>,
}

impl Galaxy {
    /// Create a new galaxy with the given sectors and systems.
    pub fn new(sectors: Vec<sector::Sector>, systems: Vec<system::System>) -> Self {
        let mut map = RTree::new();
        systems
            .iter()
            .for_each(|ref system| map.insert(HashablePoint::new(system.location)));

        let mut systems_map = HashMap::new();

        for system in systems {
            systems_map.insert(HashablePoint::new(system.location), system);
        }

        Galaxy {
            sectors,
            map,
            systems: systems_map,
        }
    }

    /// Returns a reference to the system at the given location.
    pub fn system(&self, location: &Point) -> Option<&system::System> {
        self.systems.get(&HashablePoint::new(*location))
    }

    /// Returns a mutable reference to the system at the given location.
    pub fn system_mut(&mut self, location: &Point) -> Option<&mut system::System> {
        self.systems.get_mut(&HashablePoint::new(*location))
    }

    /// Returns references to all systems.
    pub fn systems(&self) -> impl Iterator<Item = &System> {
        self.systems.values()
    }

    /// Returns mutable references to all systems.
    pub fn systems_mut(&mut self) -> impl Iterator<Item = &mut System> {
        self.systems.values_mut()
    }

    /// Finds the system with the closest matching name.
    pub fn search_name(&self, query: &str) -> Option<&system::System> {
        self.systems
            .values()
            .min_by_key(|sys| edit_distance(query, &sys.name).abs())
    }

    /// Returns all system locations reachable from the given location within the given radius.
    pub fn reachable(&self, location: &Point, max_distance: f64) -> Vec<&Point> {
        let center = *location;
        self.map
            .lookup_in_circle(&HashablePoint::new(center), &max_distance.powi(2))
            .iter()
            .map(|hashpoint| hashpoint.as_point())
            .collect::<Vec<_>>()
    }

    /// Returns the nearest system location to the given point.
    pub fn nearest(&self, location: &Point) -> Option<&Point> {
        self.map
            .nearest_neighbor(&HashablePoint::new(*location))
            .map(|p| p.as_point())
    }

    /// Finds the shortest path from start to goal with at most range along
    /// any edge and a maximum max_steps number of nodes visited.
    pub fn route(
        &self,
        start: &Point,
        goal: &Point,
        range: f64,
        max_steps: u32,
    ) -> Option<(u32, Vec<Point>)> {
        // Node -> steps, cost mapping.
        let mut dist = HashMap::<HashablePoint, u32>::new();
        let mut frontier = BinaryHeap::new();
        let mut previous = HashMap::<HashablePoint, HashablePoint>::new();

        // We're at `start`, with a zero cost
        dist.insert(HashablePoint::new(*start), 0);
        frontier.push(OrdPoint {
            weight: 0,
            point: *start,
        });

        let mut cost = None;
        // Examine the frontier with lower cost nodes first (min-heap)
        while let Some(OrdPoint { point, weight }) = frontier.pop() {
            // Alternatively we could have continued to find all shortest paths
            if point == *goal {
                cost = Some(weight);
                break;
            }

            // Important as we may have already found a better way
            if weight > *dist.get(&HashablePoint::new(point)).unwrap_or(&MAX) {
                continue;
            }

            // For each node we can reach, see if we can find a way with
            // a lower cost going through this node
            for neighbor in self.reachable(&point, (range).max(0.)) {
                let next = OrdPoint {
                    weight: weight + 1,
                    point: *neighbor,
                };

                // If so, add it to the frontier and continue
                if next.weight <= max_steps
                    && next.weight < *dist.get(&HashablePoint::new(next.point)).unwrap_or(&MAX)
                {
                    frontier.push(next.clone());
                    // Relaxation, we have now found a better way
                    dist.insert(HashablePoint::new(next.point), next.weight);
                    previous.insert(HashablePoint::new(next.point), HashablePoint::new(point));
                }
            }
        }

        match cost {
            Some(cost) => {
                let mut path = vec![];
                let mut current = HashablePoint::new(*goal);
                while current.as_point() != start {
                    path.push(*current.as_point());
                    current = previous.remove(&current).unwrap();
                }
                path.reverse();
                Some((cost, path))
            }
            None => None,
        }
    }
}

impl Default for Galaxy {
    fn default() -> Self {
        Galaxy {
            sectors: vec![],
            map: RTree::new(),
            systems: HashMap::new(),
        }
    }
}

impl Updatable for Galaxy {
    /// Advances time and updates all systems etc.
    fn update(&mut self) {
        self.systems.par_iter_mut().for_each(|(_, system)| {
            system.update();
        });
    }
}