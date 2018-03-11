use std::fmt;
use resources::ShipResource;

/// Ship currently owned by the player.
pub struct Ship {
    integrity: u32,
    fuel: f32,
    base: ShipCharacteristics,
}

impl Ship {}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Represents the characteristics of a given ship model.
pub struct ShipCharacteristics {
    pub name: String,
    pub manufacturer: String,
    pub kind: ShipType,
    pub description: String,
    pub integrity: u32,
    pub size: Dimensions,
    pub mass: u32,
    pub slots: u32,
    pub cost: u32,
    pub range: f32,
    pub fuel: f32,
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
    length: f32,
    width: f32,
    height: f32,
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
    pub fn get_available(&self) -> &Vec<ShipCharacteristics> {
        // TODO: Base the return value on the caller, i.e the faction, system etc.
        &self.ships
    }
}
