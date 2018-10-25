use super::*;
use core::faction::Faction;
use core::game::Updatable;
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use utils::Point;

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
    pub reputation: Reputation,
    pub star: Star,
    pub satelites: Vec<Planet>,
}

impl Updatable for System {
    /// Updates the system one time step.
    fn update(&mut self) {
        // Update repuation levels.
        self.reputation.update();
    }
}

impl Hash for System {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.location, state);
    }
}

impl PartialEq for System {
    fn eq(&self, other: &System) -> bool {
        self.location == other.location
    }
}

impl Eq for System {}

/// Represents the current player level of reputation with the system.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reputation(i32);

impl Updatable for Reputation {
    /// Updates the reputation level, one time step.
    fn update(&mut self) {
        // "Extreme" repuation levels converges towards lower levels.
        self.0 += match self.0 {
            -1000...-300 => 5,
            300...1000 => -5,
            _ => 0,
        }
    }
}

impl Default for Reputation {
    fn default() -> Reputation {
        Reputation { 0: 0 }
    }
}

impl fmt::Display for Reputation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out_str = match self.0 {
            -1000...-300 => "Hostile",
            -299...-100 => "Unfriendly",
            -99...100 => "Neutral",
            101...300 => "Friendly",
            301...1000 => "Allied",
            _ => "Neutral",
        };
        write!(f, "{}", out_str)
    }
}

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
        let security_str = match *self {
            SystemSecurity::Anarchy => "Anarchy",
            SystemSecurity::Low => "Low",
            SystemSecurity::Medium => "Medium",
            SystemSecurity::High => "High",
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
        let state_str = match *self {
            SystemState::Contested => "Contested",
            SystemState::CivilWar => "Civil War",
            SystemState::Boom => "Boom",
            SystemState::Bust => "Bust",
            SystemState::CivilUnrest => "Civil Unrest",
            SystemState::Famine => "Famine",
            SystemState::Outbreak => "Outbreak",
        };
        write!(f, "{}", state_str)
    }
}
