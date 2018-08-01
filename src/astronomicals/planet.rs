use std::fmt;

/// Unique planet identifier.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlanetID(u64);

impl From<u64> for PlanetID {
    fn from(id: u64) -> PlanetID {
        PlanetID { 0: id }
    }
}

#[derive(Serialize, Deserialize, Debug, Builder, Clone)]
#[builder(field(public))]
/// Represents a visitable planet in game with some attributes.
pub struct Planet {
    pub id: PlanetID,
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
