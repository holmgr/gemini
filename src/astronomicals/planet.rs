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
    Metal_rich,
    Icy,
    Rocky,
    Gas_giant,
    Earth_like,
    Water,
    Water_giant,
}

impl Planet {
    pub fn calculate_surface_temperature(orbit_distance: f64, star: &Star) -> f64 {
        (star.luminosity * 3.846 * 10f64.powi(26) * (1. - 0.29) /
             (16. * PI * (299692458. * orbit_distance).powi(2) * 5.670373 * 10f64.powi(-8)))
            .powf(0.25)
    }
}
