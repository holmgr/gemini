use std::f64::consts::PI;

use astronomicals::star::Star;

#[derive(Debug, Clone, Builder)]
#[builder(field(public))]

pub struct Planet {
    pub name: String,
    pub mass: f64,
    pub gravity: f64,
    pub orbit_distance: f64,
    pub surface_temperature: f64,
    pub planet_type: PlanetType,
}

#[derive(Debug, Clone)]
pub enum PlanetType {
    Metal,
    Icy,
    Rocky,
    GasGiant,
    Earth,
}

impl Planet {
    pub fn calculate_surface_temperature(orbit_distance: f64, star: &Star) -> f64 {
        (star.luminosity * 3.846 * 10f64.powi(26) * (1. - 0.29) /
             (16. * PI * (299692458. * orbit_distance).powi(2) * 5.670373 * 10f64.powi(-8)))
            .powf(0.25)
    }

    /// Based on trained decision tree with modifications to allow for
    /// Earth-like planets at a higher rate
    pub fn predict_type(surface_temperature: f64, mass: f64) -> PlanetType {
        match (surface_temperature, mass) {
            (x, y) if x < 124.5 && y >= 8.185 => PlanetType::GasGiant,
            (x, y) if x < 124.5 && y < 8.185 => PlanetType::Icy,
            (x, y) if x <= 300. && y <= 2. => PlanetType::Earth,
            (x, y) if x >= 124.5 && y < 0.02032 => PlanetType::Rocky,
            (x, y) if x >= 124.5 && y >= 0.02032 => PlanetType::Metal,
            _ => PlanetType::Rocky,
        }
    }
}
