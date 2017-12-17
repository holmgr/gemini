use std::f64::consts::PI;

use astronomicals::star::Star;

#[derive(Debug, Clone, Builder)]
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
    pub fn new(mass: f64, orbit_distance: f64) -> Self {

        // TODO: Make something a bit more accurate
        let gravity = mass;
        Planet {
            mass,
            gravity,
            orbit_distance,
            name: String::new(),
            surface_temperature: 0.,
            planet_type: PlanetType::Rocky,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_surface_temperature(&mut self, star: &Star) {
        self.surface_temperature = (star.luminosity * 3.846 * 10f64.powi(26) * (1. - 0.29) /
                                        (16. * PI * (299692458. * self.orbit_distance).powi(2) *
                                             5.670373 *
                                             10f64.powi(-8)))
            .powf(0.25);
    }
}
