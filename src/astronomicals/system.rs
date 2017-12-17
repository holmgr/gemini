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
        let hash = System::hash(location);
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

    /// Hash based on location, algorithm used is presented in the paper:
    /// Optimized Spatial Hashing for Collision Detection of Deformable Objects
    fn hash(location: Point<f64>) -> u64 {
        let values = location
            .iter()
            .zip(&[73856093f64, 19349663f64, 83492791f64])
            .map(|(&a, &b)| (a * b) as u64)
            .collect::<Vec<_>>();
        values.iter().fold(0, |acc, &val| acc ^ val)
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::isaac::Isaac64Rng;
    use super::*;
    extern crate env_logger;
    use statrs::distribution::{Distribution, Uniform};
    use std::collections::HashMap;

    #[test]
    fn test_hash_uniqueness() {
        let _ = env_logger::init();

        let new_seed: &[_] = &[42 as u64];
        let mut rng: Isaac64Rng = SeedableRng::from_seed(new_seed);
        let n = Uniform::new(0., 100000.).unwrap();

        let mut hashes = HashMap::new();
        let tries = 10000;
        for _ in 0..tries {
            let loc = Point::new(
                n.sample::<Isaac64Rng>(&mut rng),
                n.sample::<Isaac64Rng>(&mut rng),
                n.sample::<Isaac64Rng>(&mut rng),
            );
            hashes.insert(System::hash(loc), loc);
        }
        assert_eq!(hashes.len(), tries);

    }
}
