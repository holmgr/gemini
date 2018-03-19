use rand::Rng;
use statrs::distribution::{Distribution, Exponential, Gamma};
use generators::Gen;
use std::f64::consts::PI;
use std::fmt;

use astronomicals::star::Star;

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        let security_str = match self {
            &PlanetType::Metal => "Metal",
            &PlanetType::Icy => "Icy",
            &PlanetType::Rocky => "Rocky",
            &PlanetType::GasGiant => "Gas Giant",
            &PlanetType::Earth => "Earth",
        };
        write!(f, "{}", security_str)
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
    pub fn predict_type<R: Rng>(rng: &mut R, surface_temperature: f64, mass: f64) -> PlanetType {
        // Based on trained decision tree with modifications to allow for
        // Earth-like planets at a higher rate.
        let random_val: f64 = rng.gen();
        match (surface_temperature, mass) {
            (x, y) if x < 124.5 && y >= 5.185 => PlanetType::GasGiant,
            (x, y) if x < 124.5 && y < 5.185 => PlanetType::Icy,
            (x, _) if x > 280. && x < 310. => PlanetType::Earth,
            (x, _) if x >= 124.5 && random_val < 0.8 => PlanetType::Rocky,
            (x, _) if x >= 124.5 && random_val >= 0.8 => PlanetType::Metal,
            _ => PlanetType::Rocky,
        }
    }
}
