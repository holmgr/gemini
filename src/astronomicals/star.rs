
#[derive(Debug, Clone, Builder)]
/// Represents a Star in a system.
pub struct Star {
    pub mass: f64,
    pub luminosity: f64,
}

impl Star {
    pub fn new(mass: f64, luminosity: f64) -> Self {
        Star { mass, luminosity }
    }
}
