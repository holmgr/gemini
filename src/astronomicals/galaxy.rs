use super::*;

use rayon::prelude::*;
use spade::rtree::RTree;
use std::collections::HashMap;

/// A galaxy of systems.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Galaxy {
    pub sectors: Vec<sector::Sector>,
    pub map: RTree<Point>,
    pub systems: HashMap<Point, system::System>,
}

impl Galaxy {
    /// Create a new galaxy with the given sectors and systems.
    pub fn new(sectors: Vec<sector::Sector>, systems: Vec<system::System>) -> Self {
        let map = RTree::bulk_load(
            systems
                .iter()
                .map(|system| system.location)
                .collect::<Vec<_>>(),
        );

        let mut systems_map = HashMap::new();

        for system in systems {
            systems_map.insert(system.location, system);
        }

        Galaxy {
            sectors,
            map,
            systems: systems_map,
        }
    }

    /// Returns a reference to the system at the given location.
    pub fn system(&self, location: &Point) -> Option<&system::System> {
        self.systems.get(location)
    }

    /// Returns a mutable reference to the system at the given location.
    #[allow(dead_code)]
    pub fn system_mut(&mut self, location: &Point) -> Option<&mut system::System> {
        self.systems.get_mut(location)
    }

    /// Returns references to all systems.
    #[allow(dead_code)]
    pub fn systems(&self) -> impl Iterator<Item = &System> {
        self.systems.values()
    }

    /// Returns mutable references to all systems.
    #[allow(dead_code)]
    pub fn systems_mut(&mut self) -> impl Iterator<Item = &mut System> {
        self.systems.values_mut()
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
