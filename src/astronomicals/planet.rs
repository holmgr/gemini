use std::f64::consts::PI;
use std::fmt;

use astronomicals::star::Star;

#[derive(Debug, Clone, Builder)]
#[builder(field(public))]
/// Represents a visitable planet in game with some attributes.
pub struct Planet {
    pub name: String,
    pub mass: f64,
    pub gravity: f64,
    pub orbit_distance: f64,
    pub surface_temperature: f64,
    pub planet_type: PlanetType,
}

#[derive(Debug, Clone)]
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
        write!(f, "{:?}", self)
    }
}

impl Planet {
    /// Calculate planet surface temperature from star luminosity and distance
    /// to it. Uses the Bond albedo for the Earth.
    pub fn calculate_surface_temperature(orbit_distance: f64, star: &Star) -> f64 {
        (star.luminosity * 3.846 * 10f64.powi(26) * (1. - 0.29)
            / (16. * PI * (299692458. * orbit_distance).powi(2) * 5.670373 * 10f64.powi(-8)))
            .powf(0.25)
    }

    /// Predict the planet type based on surface_temperature and mass.
    pub fn predict_type(surface_temperature: f64, mass: f64) -> PlanetType {
        // Based on trained decision tree with modifications to allow for
        // Earth-like planets at a higher rate.
        // TODO: Rocky seems very unlikely
        match (surface_temperature, mass) {
            (x, y) if x < 124.5 && y >= 8.185 => PlanetType::GasGiant,
            (x, y) if x < 124.5 && y < 8.185 => PlanetType::Icy,
            (x, y) if x > 280. && x < 300. && y <= 2. => PlanetType::Earth,
            (x, y) if x >= 124.5 && y < 0.02032 => PlanetType::Rocky,
            (x, y) if x >= 124.5 && y >= 0.02032 => PlanetType::Metal,
            _ => PlanetType::Rocky,
        }
    }
}
