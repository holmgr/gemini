use nalgebra::geometry::Point3;
use rand::SeedableRng;
use rand::isaac::Isaac64Rng;
use std::sync::{Arc, Mutex};

use generators::stars::StarGen;
use generators::names::NameGen;
use generators::MutGen;
use generators::Gen;

#[derive(Debug)]
pub struct Galaxy {
    systems: Vec<System>,
}

impl Galaxy {
    pub fn new(systems: Vec<System>) -> Self {
        Galaxy { systems }
    }
}

#[derive(Debug)]
pub struct Star {
    mass: f64,
    luminosity: f64,
    metalicity: f64,
}

impl Star {
    pub fn new(mass: f64, luminosity: f64, metalicity: f64) -> Self {
        Star {
            mass,
            luminosity,
            metalicity,
        }
    }
}

#[derive(Debug)]
struct Planet {
    mass: f64,
    orbit_distance: f64,
    orbit_time: f64,
}

#[derive(Debug)]
pub struct System {
    location: Point3<f64>,
    name: String,
    star: Star,
    satelites: Vec<Planet>,
}

impl System {
    pub fn new(location: Point3<f64>, name_gen: Arc<Mutex<NameGen>>, star_gen: &StarGen) -> Self {

        // Calculate hash
        let hash = System::hash(location);
        let seed: &[_] = &[hash];
        let mut rng: Isaac64Rng = SeedableRng::from_seed(seed);

        let star = star_gen.generate(&mut rng).unwrap();

        // Unwrap and lock name generator as it is mutated by generation
        let mut name_gen_unwraped = name_gen.lock().unwrap();

        // Fallback to "Unnamed"
        let name = name_gen_unwraped.generate().unwrap_or(
            String::from("Unnamed"),
        );

        // TODO: Planets
        let satelites = vec![];

        System {
            location,
            name,
            star,
            satelites,
        }
    }

    /// Hash based on location, algorithm used is presented in the paper:
    /// Optimized Spatial Hashing for Collision Detection of Deformable Objects
    fn hash(location: Point3<f64>) -> u64 {
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
            let loc = Point3::new(
                n.sample::<Isaac64Rng>(&mut rng),
                n.sample::<Isaac64Rng>(&mut rng),
                n.sample::<Isaac64Rng>(&mut rng),
            );
            hashes.insert(System::hash(loc), loc);
        }
        assert_eq!(hashes.len(), tries);

    }
}
