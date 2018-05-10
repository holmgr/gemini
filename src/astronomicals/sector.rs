use utils::Point;
use entities::Faction;

/// Represents a group of systems in close proximity within the same faction.
/// Markets in the economy is handled on this level of scale.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Sector {
    pub faction: Faction,
    pub system_locations: Vec<Point>,
}
