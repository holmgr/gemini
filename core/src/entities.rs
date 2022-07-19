use game::Updatable;
use rand::Rng;
use rayon::iter::IntoParallelRefMutIterator;
use statrs::distribution::{Categorical, Distribution};
use std::{
    fmt,
    hash::{Hash, Hasher},
};
use utils::Point;

use rayon::iter::ParallelIterator;
use spade::rtree::RTree;
use std::collections::HashMap;

/// Represents a single Faction which is assigned on Sector level.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Faction {
    Empire,
    Federation,
    Cartel,
    Independent,
}

impl Faction {
    /// Generate a random faction according to the "distribution".
    pub fn random_faction<R: Rng>(gen: &mut R) -> Faction {
        let probs = Categorical::new(&[15., 45., 30., 10.]).unwrap();

        match probs.sample::<R>(gen) as usize {
            0 => Faction::Cartel,
            1 => Faction::Empire,
            2 => Faction::Federation,
            3 => Faction::Independent,
            _ => Faction::Independent,
        }
    }
}

impl fmt::Display for Faction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A galaxy of systems.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Galaxy {
    pub sectors: Vec<Sector>,
    pub map: RTree<Point>,
    pub systems: HashMap<Point, System>,
}

impl Galaxy {
    /// Create a new galaxy with the given sectors and systems.
    pub fn new(sectors: Vec<Sector>, systems: Vec<System>) -> Self {
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
    pub fn system(&self, location: &Point) -> Option<&System> {
        self.systems.get(location)
    }

    /// Returns a mutable reference to the system at the given location.
    #[allow(dead_code)]
    pub fn system_mut(&mut self, location: &Point) -> Option<&mut System> {
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

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
#[builder(field(public))]
/// Represents a visitable planet in game with some attributes.
pub struct Planet {
    pub name: String,
    pub mass: f64,
    pub gravity: f64,
    pub orbit_distance: f64,
    pub surface_temperature: f64,
    pub planet_type: PlanetType,
    pub economic_type: PlanetEconomy,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
/// Different types of planet, i.e environments. Depends on surface_temperature
/// and mass.
pub enum PlanetType {
    Metal,
    Icy,
    Rocky,
    GasGiant,
    Earth,
}

impl fmt::Display for PlanetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let security_str = match *self {
            PlanetType::Metal => "Metal",
            PlanetType::Icy => "Icy",
            PlanetType::Rocky => "Rocky",
            PlanetType::GasGiant => "Gas Giant",
            PlanetType::Earth => "Earth",
        };
        write!(f, "{}", security_str)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
/// Different types of planet economies, depends on the planet type.
pub enum PlanetEconomy {
    Agriculture,
    Extraction,
    HighTech,
    Industrial,
    None,
    Military,
    Refinary,
}

impl fmt::Display for PlanetEconomy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                PlanetEconomy::Agriculture => "Agriculture",
                PlanetEconomy::Extraction => "Extraction",
                PlanetEconomy::HighTech => "High Tech",
                PlanetEconomy::Industrial => "Industrial",
                PlanetEconomy::None => "None",
                PlanetEconomy::Military => "Military",
                PlanetEconomy::Refinary => "Refinary",
            }
        )
    }
}

/// Represents a group of systems in close proximity within the same faction.
/// Markets in the economy is handled on this level of scale.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Sector {
    pub faction: Faction,
    pub system_locations: Vec<Point>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Builder)]
/// Represents a Star in a system.
pub struct Star {
    pub mass: f64,
    pub luminosity: f64,
    pub startype: StarType,
}

impl Star {
    pub fn new(mass: f64, luminosity: f64, kind: StarType) -> Self {
        Star {
            mass,
            luminosity,
            startype: kind,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Describes the type of star: Single or Binary.
pub enum StarType {
    Single,
    Binary,
}

impl fmt::Display for StarType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let type_str = match *self {
            StarType::Single => "Single",
            StarType::Binary => "Binary",
        };
        write!(f, "{}", type_str)
    }
}

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
            -1000..=-300 => 5,
            300..=1000 => -5,
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
            -1000..=-300 => "Hostile",
            -299..=-100 => "Unfriendly",
            -99..=100 => "Neutral",
            101..=300 => "Friendly",
            301..=1000 => "Allied",
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
