use nalgebra::geometry::Point3 as Point;

use astronomicals::system::System;
use entities::Faction;

/// Represents a group of systems in close proximity within the same faction.
/// Markets in the economy is handled on this level of scale.
#[derive(Debug)]
pub struct Sector {
    pub name: String,
    pub faction: Faction,
    pub systems: Vec<System>,
}

impl Sector {
    /// Returns the point representing the centroid of the cluster.
    pub fn center(&self) -> Point<f64> {
        self.systems.iter().fold(Point::origin(), |center, system| {
            center + (system.location.coords / (self.systems.len() as f64))
        })
    }
}
