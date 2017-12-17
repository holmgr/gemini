use rand::{Rng, SeedableRng};
use std::time::Instant;
use rayon::prelude::*;
use rand::isaac::IsaacRng;
use statrs::distribution::{Distribution, Gamma, Uniform, Normal};
use nalgebra::geometry::Point3 as Point;
use std::sync::{Arc, Mutex};

pub mod names;
pub mod stars;
pub mod planets;

use resources::{fetch_resource, StarTypesResource, PlanetTypesResource, AstronomicalNamesResource};
use astronomicals::{Galaxy, hash};
use astronomicals::system::{SystemBuilder, System};
use game_config::GameConfig;
use generators::stars::StarGen;
use generators::names::NameGen;
use generators::planets::PlanetGen;
use astronomicals::planet::Planet;

/// A generator that can be explicitly seeded in order to the produce the same
/// stream of psuedo randomness each time
pub trait SeedableGenerator {
    /// Reseed a generator with the given seed
    fn reseed(&mut self, seed: u32);

    /// Create a new generator with the given seed
    fn from_seed(seed: u32) -> Self;
}

/// A generator which can be trained by provided some training resource
pub trait TrainableGenerator {
    type TrainRes;

    /// Train the generator with the given data
    fn train(&mut self, &Self::TrainRes);
}

/// Generic mutable Generator, may modify the generator after generating an item
pub trait MutGen: TrainableGenerator + SeedableGenerator {
    type GenItem;

    /// Generate a new item from the generator, can be None if the generator is empty etc.
    fn generate(&mut self) -> Option<Self::GenItem>;
}

/// Generic Generator, does not modify the generator instead uses provided random number generator
pub trait Gen: TrainableGenerator {
    type GenItem;

    /// Generate a new item from the generator, can be None if the generator is empty etc.
    fn generate<R: Rng>(&self, gen: &mut R) -> Option<Self::GenItem>;
}

/// Generate a galaxy with systems etc, will use the provided config to guide
/// the generation
pub fn generate_galaxy(config: &GameConfig) -> Galaxy {
    let new_seed: &[_] = &[config.map_seed];
    let mut rng: IsaacRng = SeedableRng::from_seed(new_seed);

    // Measure time for generation
    let now = Instant::now();

    // Clusters are spaced uniformly, systems gaussian
    let cluster_loc_gen = Uniform::new(0., config.galaxy_size).unwrap();
    let system_loc_gen = Normal::new(config.cluster_size_mean, config.cluster_size_std).unwrap();

    // Generate clusters
    let mut clusters = vec![];
    for _ in 0..config.number_of_clusters {
        clusters.push((
            Point::new(
                cluster_loc_gen.sample::<IsaacRng>(&mut rng),
                cluster_loc_gen.sample::<IsaacRng>(&mut rng),
                cluster_loc_gen.sample::<IsaacRng>(&mut rng),
            ),
            system_loc_gen.sample::<IsaacRng>(&mut rng) as u64,
        ))
    }

    // Create name generator to be shared mutably
    let mut name_gen_unwraped = names::NameGen::from_seed(config.map_seed);
    name_gen_unwraped.train(&fetch_resource::<AstronomicalNamesResource>().unwrap());
    let name_gen = Arc::new(Mutex::new(name_gen_unwraped));

    // Create Star generator
    let mut star_gen = stars::StarGen::new();
    star_gen.train(&fetch_resource::<StarTypesResource>().unwrap());

    // Create Planet generator
    let mut planet_gen = planets::PlanetGen::new();
    planet_gen.train(&fetch_resource::<PlanetTypesResource>().unwrap());

    // Generate systems for each cluster in parallel
    // Fold will generate one vector per thread (per cluster), reduce will
    //combine them to the final result
    let systems = clusters
        .into_par_iter()
        .fold(|| Vec::<System>::new(), |mut cluster_systems: Vec<System>,
         (cluster_pos, cluster_size)| {
            // Generate x,y,z generators based at cluster location
            let norm_x = Normal::new(cluster_pos.coords.x, config.cluster_spread).unwrap();
            let norm_y = Normal::new(cluster_pos.coords.y, config.cluster_spread).unwrap();
            let norm_z = Normal::new(cluster_pos.coords.z, config.cluster_spread).unwrap();

            // TODO: Do something smarter than cloning state of generator,
            // since all clusters will be generated identically now
            let mut rng = rng.clone();

            // Generate systems
            for _ in 0..cluster_size {
                cluster_systems.push(generate_system(
                    Point::new(
                        norm_x.sample::<IsaacRng>(&mut rng),
                        norm_y.sample::<IsaacRng>(&mut rng),
                        norm_z.sample::<IsaacRng>(&mut rng),
                    ),
                    name_gen.clone(),
                    &star_gen,
                    &planet_gen,
                ));
            }
            cluster_systems
        })
        .reduce(|| Vec::<System>::new(), |mut systems, subsystems| {
            systems.extend(subsystems);
            systems
        });

    info!(
        "Generated new galaxy containing: {} clusters and {} systems and {} planets taking {} ms",
        config.number_of_clusters,
        systems.len(),
        systems.iter().fold(
            0,
            |acc, ref sys| acc + sys.satelites.len(),
        ),
        ((now.elapsed().as_secs() * 1_000) + (now.elapsed().subsec_nanos() / 1_000_000) as u64)
    );
    debug!(
        "Generated System examples: {:?}",
        systems.iter().take(5).collect::<Vec<_>>()
    );
    Galaxy::new(systems)
}

// Generate a new star system using the given generators and a location as seed
pub fn generate_system(
    location: Point<f64>,
    name_gen: Arc<Mutex<NameGen>>,
    star_gen: &StarGen,
    planet_gen: &PlanetGen,
) -> System {
    // Calculate hash
    let hash = hash(location);
    let seed: &[_] = &[hash as u32];
    let mut rng: IsaacRng = SeedableRng::from_seed(seed);

    let star = star_gen.generate(&mut rng).unwrap();

    // Unwrap and lock name generator as it is mutated by generation
    let mut name_gen_unwraped = name_gen.lock().unwrap();

    // TODO: Replace constant in config
    let num_planets = Gamma::new(1., 0.5)
        .unwrap()
        .sample::<IsaacRng>(&mut rng)
        .round() as u32;
    let satelites: Vec<Planet> = (0..num_planets)
        .map(|_| {
            let mut builder = planet_gen.generate(&mut rng).unwrap();
            let orbit_distance = builder.orbit_distance.unwrap();
            builder
                .name(name_gen_unwraped.generate().unwrap_or(
                    String::from("Unnamed"),
                ))
                .surface_temperature(Planet::calculate_surface_temperature(orbit_distance, &star))
                .build()
                .unwrap()
        })
        .collect();

    /*
    // Fallback to "Unnamed" for names
    for planet in &mut satelites {
        planet.set_name(name_gen_unwraped.generate().unwrap_or(
            String::from("Unnamed"),
        ));
    }
    */
    // System name is the same as one random planet
    let name = match rng.choose(&satelites) {
        Some(planet) => planet.name.clone(),
        None => {
            name_gen_unwraped.generate().unwrap_or(
                String::from("Unnamed"),
            )
        }
    } + " System";

    SystemBuilder::default()
        .location(location)
        .name(name)
        .star(star)
        .satelites(satelites)
        .build()
        .unwrap()
}
