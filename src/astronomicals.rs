use nalgebra::geometry::Point3;
use rand::SeedableRng;
use rand::isaac::Isaac64Rng;

use generators::stars::StarGen;
use generators::Gen;

pub struct Galaxy {
    systems: Vec<System>,
}

impl Galaxy {
    pub fn new(systems: Vec<System>) -> Self {
        Galaxy { systems }
    }
}

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

struct Planet {
    mass: f64,
    orbit_distance: f64,
    orbit_time: f64,
}

pub struct System {
    location: Point3<f64>,
    star: Star,
    satelites: Vec<Planet>,
}

impl System {
    pub fn new(location: Point3<f64>, star_gen: &StarGen) -> Self {

        // Calculate hash
        let hash = System::hash(location);
        let seed: &[_] = &[hash];
        let mut rng: Isaac64Rng = SeedableRng::from_seed(seed);

        let star = star_gen.generate(&mut rng).unwrap();

        // TODO: Planets
        let satelites = vec![];

        System {
            location,
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
    use rand::{Rng, SeedableRng};
    use rand::isaac::Isaac64Rng;
    use super::*;
    extern crate env_logger;
    use statrs::distribution::{Distribution, Uniform};
    use statrs::statistics::{Median, Variance};
    use std::collections::HashMap;
    use std::i64::MAX;

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
