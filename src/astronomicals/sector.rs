use astronomicals::system::System;

/// Represents a group of systems in close proximity within the same faction.
/// Markets in the economy is handled on this level of scale.
#[derive(Debug)]
pub struct Sector {
    pub name: String,
    pub systems: Vec<System>,
}
