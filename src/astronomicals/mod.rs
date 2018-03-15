use std::collections::HashMap;
use spade::rtree::RTree;
use nalgebra::distance;

use utils::{HashablePoint, Point};

pub mod star;
pub mod planet;
pub mod system;
pub mod sector;

/// Main galaxy containing all systems.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Galaxy {
    pub sectors: Vec<sector::Sector>,
    pub map: RTree<HashablePoint>,
    pub systems: HashMap<HashablePoint, system::System>,
}

impl Galaxy {
    /// Create a new Galaxy with the given sectors and systems.
    pub fn new(sectors: Vec<sector::Sector>, systems: Vec<system::System>) -> Self {
        let mut map = RTree::new();
        systems
            .iter()
            .for_each(|ref system| map.insert(HashablePoint::new(system.location.clone())));

        let mut systems_map = HashMap::new();

        for system in systems {
            systems_map.insert(HashablePoint::new(system.location.clone()), system);
        }

        Galaxy {
            sectors,
            map,
            systems: systems_map,
        }
    }

    /// Returns a reference system at the given location if it exists.
    pub fn system(&self, location: &Point) -> Option<&system::System> {
        self.systems.get(&HashablePoint::new(location.clone()))
    }

    /// Returns a mutable reference system at the given location if it exists.
    pub fn system_mut(&mut self, location: &Point) -> Option<&mut system::System> {
        self.systems.get_mut(&HashablePoint::new(location.clone()))
    }

    /// Returns references to all systems.
    pub fn systems(&self) -> Vec<&system::System> {
        self.systems.values().collect::<Vec<_>>()
    }

    /// Returns references to all systems ordered by distance to the given point.
    pub fn systems_ordered(&self, location: &Point) -> Vec<&system::System> {
        let mut systems = self.systems.values().collect::<Vec<_>>();
        systems.sort_unstable_by(|a, b| {
            distance(location, &a.location)
                .partial_cmp(&distance(location, &b.location))
                .unwrap()
        });
        systems
    }

    /// Returns mutable references to all systems.
    pub fn systems_mut(&mut self) -> Vec<&mut system::System> {
        self.systems.values_mut().collect::<Vec<_>>()
    }

    /// Returns all system locations reachable from the given location within the given radius.
    pub fn reachable(&self, location: &Point, max_distance: f64) -> Vec<&Point> {
        self.map
            .lookup_in_circle(&HashablePoint::new(location.clone()), &max_distance.sqrt())
            .iter()
            .map(|hashpoint| hashpoint.as_point())
            .collect::<Vec<_>>()
    }

    /// Returns the nearest system location to the given point.
    pub fn nearest(&self, location: &Point) -> Option<&Point> {
        self.map
            .nearest_neighbor(&HashablePoint::new(location.clone()))
            .map(|p| p.as_point())
    }
}

/// Hash based on location, algorithm used is presented in the paper:
/// Optimized Spatial Hashing for Collision Detection of Deformable Objects.
pub fn hash(location: &Point) -> u64 {
    let values = location
        .iter()
        .zip(&[73856093f64, 19349663f64, 83492791f64])
        .map(|(&a, &b)| (a * b) as u64)
        .collect::<Vec<_>>();
    values.iter().fold(0, |acc, &val| acc ^ val)
}
