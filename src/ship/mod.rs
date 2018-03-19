use std::fmt;
use resources::ShipResource;
use entities::Faction;
use astronomicals::system::System;

/// Ship currently owned by the player.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Ship {
    integrity: u32,
    fuel: u32,
    base: ShipCharacteristics,
}

impl Ship {
    pub fn new(model: ShipCharacteristics) -> Ship {
        Ship {
            integrity: model.integrity,
            fuel: model.fuel,
            base: model,
        }
    }

    /// Returns a reference to the ship's current integrity.
    pub fn integrity(&self) -> &u32 {
        &self.integrity
    }

    /// Returns maximum jump range.
    pub fn range(&self) -> f64 {
        self.base.range
    }

    /// Returns the ship's current fuel.
    pub fn fuel(&self) -> u32 {
        self.fuel
    }

    /// Update the fuel level.
    pub fn set_fuel(&mut self, fuel: u32) {
        self.fuel = fuel;
    }

    /// Returns a reference to the ship's characteristics.
    pub fn characteristics(&self) -> &ShipCharacteristics {
        &self.base
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents the characteristics of a given ship model.
pub struct ShipCharacteristics {
    pub name: String,
    pub manufacturer: String,
    pub faction: Option<Faction>,
    pub kind: ShipType,
    pub description: String,
    pub integrity: u32,
    pub size: Dimensions,
    pub mass: u32,
    pub slots: u32,
    pub cost: u32,
    pub range: f64,
    pub fuel: u32,
    pub cargo: u32,
    pub detectability: u32,
    pub maneuverability: u32,
    pub defense: u32,
    pub shield: u32,
}

impl ShipCharacteristics {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ShipType {
    Assault,
    Corvette,
    Freighter,
}

impl fmt::Display for ShipType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &ShipType::Assault => write!(f, "Assault Ship"),
            &ShipType::Corvette => write!(f, "Light Corvette"),
            &ShipType::Freighter => write!(f, "Light Freighter"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The size of a ship.
pub struct Dimensions {
    length: f64,
    width: f64,
    height: f64,
}

impl fmt::Display for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}L, {}W, {}H", self.length, self.width, self.height)
    }
}

/// Holds the different ships in the game.
pub struct Shipyard {
    ships: Vec<ShipCharacteristics>,
}

impl Shipyard {
    /// Returns a new shipyard.
    pub fn new() -> Shipyard {
        Shipyard { ships: vec![] }
    }

    /// Extend shipyard with more ships.
    pub fn add_ships(&mut self, resource: ShipResource) {
        self.ships.extend(resource.ships);
    }

    /// Get all available ships.
    pub fn get_available(&self, system: &System) -> Vec<ShipCharacteristics> {
        self.ships
            .iter()
            .cloned()
            .filter(|ref ship| {
                // Only return if the faction matches, if any faction is specified.
                if let Some(ref faction) = ship.faction {
                    return *faction == system.faction;
                }
                true
            })
            .collect::<Vec<_>>()
    }

    /// Create a new starting ship.
    pub fn create_base_ship(&self) -> Ship {
        Ship::new(self.ships[0].clone())
    }
}
