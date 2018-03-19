use std::hash::{Hash, Hasher};
use std::fmt;
use utils::Point;
use entities::Faction;
use astronomicals::hash;
use astronomicals::star::Star;
use astronomicals::planet::Planet;

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
#[builder(field(public))]
/// Represets a single star system with at a given location with the given
/// star and planets.
pub struct System {
    pub location: Point,
    pub name: String,
    pub faction: Faction,
    pub security: SystemSecurity,
    pub state: SystemState,
    pub star: Star,
    pub satelites: Vec<Planet>,
}

impl Hash for System {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash(&self.location).hash(state);
    }
}

impl PartialEq for System {
    fn eq(&self, other: &System) -> bool {
        self.location == other.location
    }
}

impl Eq for System {}

/// Represents the different security levels a system is in at a given point.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SystemSecurity {
    Anarchy,
    Low,
    Medium,
    High,
}

impl fmt::Display for SystemSecurity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let security_str = match self {
            &SystemSecurity::Anarchy => "Anarchy",
            &SystemSecurity::Low => "Low",
            &SystemSecurity::Medium => "Medium",
            &SystemSecurity::High => "High",
        };
        write!(f, "{}", security_str)
    }
}

/// Represents the different states a system is in at a given point.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SystemState {
    Contested,
    CivilWar,
    Boom,
    Bust,
    CivilUnrest,
    Famine,
    Outbreak,
}

impl fmt::Display for SystemState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state_str = match self {
            &SystemState::Contested => "Contested",
            &SystemState::CivilWar => "Civil War",
            &SystemState::Boom => "Boom",
            &SystemState::Bust => "Bust",
            &SystemState::CivilUnrest => "Civil Unrest",
            &SystemState::Famine => "Famine",
            &SystemState::Outbreak => "Outbreak",
        };
        write!(f, "{}", state_str)
    }
}
