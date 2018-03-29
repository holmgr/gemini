use utils::Point;
use entities::Faction;

/// Represents a group of systems in close proximity within the same faction.
/// Markets in the economy is handled on this level of scale.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Sector {
    pub name: String,
    pub faction: Faction,
    pub system_locations: Vec<Point>,
}

impl Sector {
    /// Returns the point representing the centroid of the cluster.
    pub fn center(&self) -> Point {
        self.system_locations
            .iter()
            .fold(Point::origin(), |center, system| {
                center + (system.coords / (self.system_locations.len() as f64))
            })
    }
}
