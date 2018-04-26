use std::fmt;

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
