use nalgebra::geometry::Point3 as Point;
use rand::{Rng, SeedableRng};
use rand::isaac::Isaac64Rng;
use std::sync::{Arc, Mutex};
use statrs::distribution::{Gamma, Distribution};

use generators::stars::StarGen;
use generators::names::NameGen;
use generators::planets::PlanetGen;
use generators::MutGen;
use generators::Gen;
use astronomicals::hash;
use astronomicals::star::Star;
use astronomicals::planet::Planet;

#[derive(Debug, Builder)]
pub struct System {
    location: Point<f64>,
    name: String,
    star: Star,
    pub satelites: Vec<Planet>,
}

impl System {
    pub fn new(
        location: Point<f64>,
        name_gen: Arc<Mutex<NameGen>>,
        star_gen: &StarGen,
        planet_gen: &PlanetGen,
    ) -> Self {

        // Calculate hash
        let hash = hash(location);
        let seed: &[_] = &[hash];
        let mut rng: Isaac64Rng = SeedableRng::from_seed(seed);

        let star = star_gen.generate(&mut rng).unwrap();

        // Unwrap and lock name generator as it is mutated by generation
        let mut name_gen_unwraped = name_gen.lock().unwrap();


        // TODO: Replace constant in config
        let num_planets = Gamma::new(1., 0.5)
            .unwrap()
            .sample::<Isaac64Rng>(&mut rng)
            .round() as u32;
        let mut satelites: Vec<Planet> = (0..num_planets)
            .map(|_| planet_gen.generate(&mut rng).unwrap())
            .collect();

        // Fallback to "Unnamed" for names
        for planet in &mut satelites {
            planet.set_name(name_gen_unwraped.generate().unwrap_or(
                String::from("Unnamed"),
            ));
            planet.set_surface_temperature(&star);
        }

        // System name is the same as one random planet
        let name = match rng.choose(&satelites) {
            Some(planet) => planet.name.clone(),
            None => {
                name_gen_unwraped.generate().unwrap_or(
                    String::from("Unnamed"),
                )
            }
        } + " System";

        System {
            location,
            name,
            star,
            satelites,
        }
    }

}
