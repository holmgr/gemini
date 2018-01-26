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
