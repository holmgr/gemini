use rand::{sample, Rng, SeedableRng, StdRng};
use std::time::Instant;
use std::collections::HashMap;
use rayon::prelude::*;
use statrs::distribution::{Distribution, Gamma, Normal};
use nalgebra::geometry::Point3 as Point;
use nalgebra::distance;
use std::sync::{Arc, Mutex};
use std::usize::MAX;

pub mod names;
pub mod stars;
pub mod planets;

use resources::{fetch_resource, AstronomicalNamesResource};
use astronomicals::{hash, Galaxy};
use astronomicals::system::{System, SystemBuilder};
use game_config::GameConfig;
use generators::stars::StarGen;
use generators::names::NameGen;
use generators::planets::PlanetGen;
use astronomicals::planet::Planet;
use astronomicals::sector::Sector;
use entities::Faction;

/// A generator that can be explicitly seeded in order to the produce the same
/// stream of psuedo randomness each time.
pub trait SeedableGenerator {
    /// Reseed a generator with the given seed.
    fn reseed(&mut self, seed: u32);

    /// Create a new generator with the given seed.
    fn from_seed(seed: u32) -> Self;
}

/// A generator which can be trained by provided some training resource.
pub trait TrainableGenerator {
    type TrainRes;

    /// Train the generator with the given data.
    fn train(&mut self, &Self::TrainRes);
}

/// Generic mutable Generator, may modify the generator after generating an item.
pub trait MutGen: TrainableGenerator + SeedableGenerator {
    type GenItem;

    /// Generate a new item from the generator, can be None if the generator is empty etc.
    fn generate(&mut self) -> Option<Self::GenItem>;
}

/// Generic Generator, does not modify the generator instead uses provided random number generator.
pub trait Gen {
    type GenItem;

    /// Generate a new item from the generator, can be None if the generator is empty etc.
    fn generate<R: Rng>(&self, gen: &mut R) -> Option<Self::GenItem>;
}

/// Generate a galaxy with systems etc, will use the provided config to guide
/// the generation.
pub fn generate_galaxy(config: &GameConfig) -> Galaxy {
    let new_seed: &[_] = &[config.map_seed as usize];
    let mut rng: StdRng = SeedableRng::from_seed(new_seed);

    // Measure time for generation.
    let now = Instant::now();

    // Clusters are spaced uniformly, systems gaussian.
    let loc_x = Normal::new(0., config.system_spread).unwrap();
    let loc_y = Normal::new(0., config.system_spread).unwrap();
    let loc_z = Normal::new(0., config.system_spread / 10.).unwrap();

    // Generate system locations.
    let mut locations = vec![];
    for _ in 0..config.number_of_systems {
        locations.push(Point::new(
            loc_x.sample::<StdRng>(&mut rng),
            loc_y.sample::<StdRng>(&mut rng),
            loc_z.sample::<StdRng>(&mut rng),
        ))
    }

    // Create name generator to be shared mutably.
    let mut name_gen_unwraped = names::NameGen::from_seed(config.map_seed);
    name_gen_unwraped.train(&fetch_resource::<AstronomicalNamesResource>().unwrap());
    let name_gen = Arc::new(Mutex::new(name_gen_unwraped));

    // Create Star generator.
    let star_gen = stars::StarGen::new();

    // Create Planet generator.
    let planet_gen = planets::PlanetGen::new();

    // Generate systems for each cluster in parallel.
    // Fold will generate one vector per thread (per cluster), reduce will
    // combine them to the final result.
    let systems = locations
        .into_par_iter()
        .fold(
            || Vec::<System>::new(),
            |mut systems: Vec<System>, location| {
                // Generate system
                systems.push(generate_system(
                    location,
                    name_gen.clone(),
                    &star_gen,
                    &planet_gen,
                ));
                systems
            },
        )
        .reduce(
            || Vec::<System>::new(),
            |mut systems, subsystems| {
                systems.extend(subsystems);
                systems
            },
        );

    info!(
        "Generated new galaxy containing: {} systems and {} planets taking {} ms",
        systems.len(),
        systems
            .iter()
            .fold(0, |acc, ref sys| acc + sys.satelites.len(),),
        ((now.elapsed().as_secs() * 1_000) + (now.elapsed().subsec_nanos() / 1_000_000) as u64)
    );
    debug!("Generated System examples:");
    for system in systems.iter().take(10) {
        debug!("{:#?}\n", system);
    }

    Galaxy::new(into_sectors(config, name_gen, systems))
}

/// Generate a new star system using the given generators and a location as seed.
pub fn generate_system(
    location: Point<f64>,
    name_gen: Arc<Mutex<NameGen>>,
    star_gen: &StarGen,
    planet_gen: &PlanetGen,
) -> System {
    // Calculate hash.
    let hash = hash(location);
    let seed: &[_] = &[hash as usize];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let star = star_gen.generate(&mut rng).unwrap();

    // Unwrap and lock name generator as it is mutated by generation.
    let mut name_gen_unwraped = name_gen.lock().unwrap();
    name_gen_unwraped.reseed(hash as u32);

    // TODO: Replace constant in config.
    let num_planets = Gamma::new(1., 0.5)
        .unwrap()
        .sample::<StdRng>(&mut rng)
        .round() as u32;

    // Fallback to planet name: Unnamed if no name could be generated.
    let satelites: Vec<Planet> = (0..num_planets)
        .map(|_| {
            let mut builder = planet_gen.generate(&mut rng).unwrap();
            let mass = builder.mass.unwrap();
            let surface_temperature =
                Planet::calculate_surface_temperature(builder.orbit_distance.unwrap(), &star);
            builder
                .name(
                    name_gen_unwraped
                        .generate()
                        .unwrap_or(String::from("Unnamed")),
                )
                .surface_temperature(surface_temperature)
                .planet_type(Planet::predict_type(surface_temperature, mass))
                .build()
                .unwrap()
        })
        .collect();

    // System name is the same as one random planet.
    // Fallback to: Unnamed System if it contains no planets and no name could
    // be generated.
    let name = match rng.choose(&satelites) {
        Some(planet) => planet.name.clone(),
        None => name_gen_unwraped
            .generate()
            .unwrap_or(String::from("Unnamed")),
    } + " System";

    SystemBuilder::default()
        .location(location)
        .name(name)
        .star(star)
        .satelites(satelites)
        .build()
        .unwrap()
}

/// Split the systems in to a set number of clusters using K-means.
fn into_sectors(
    config: &GameConfig,
    name_gen: Arc<Mutex<NameGen>>,
    systems: Vec<System>,
) -> Vec<Sector> {
    // Measure time for generation.
    let now = Instant::now();

    info!("Simulating expansion for initial sectors...");
    let seed: &[_] = &[hash as usize];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    // System to cluster_id mapping
    let mut cluster_map = systems
        .into_par_iter()
        .fold(
            || HashMap::<System, usize>::new(),
            |mut partial_map: HashMap<System, usize>, system| {
                // Generate system
                partial_map.insert(system, 0);
                partial_map
            },
        )
        .reduce(
            || HashMap::<System, usize>::new(),
            |mut cluster_map, partial_map| {
                cluster_map.extend(partial_map);
                cluster_map
            },
        );

    // Setup initial centroids
    let mut centroids = sample(&mut rng, cluster_map.iter(), config.number_of_sectors)
        .into_iter()
        .map(|(system, _)| system.location.clone())
        .collect::<Vec<_>>();

    // Run K means until convergence, i.e until no reassignments
    let mut has_assigned = true;
    while has_assigned {
        let wrapped_assigned = Mutex::new(false);

        // Assign to closest centroid
        cluster_map.par_iter_mut().for_each(|(system, cluster_id)| {
            let mut closest_cluster = *cluster_id;
            let mut closest_distance = distance(&system.location, &centroids[*cluster_id]);
            for i in 0..centroids.len() {
                let distance = distance(&system.location, &centroids[i]);
                if distance < closest_distance {
                    *wrapped_assigned.lock().unwrap() = true;
                    closest_cluster = i;
                    closest_distance = distance;
                }
            }
            *cluster_id = closest_cluster;
        });

        has_assigned = *wrapped_assigned.lock().unwrap();

        // Calculate new centroids
        centroids
            .par_iter_mut()
            .enumerate()
            .for_each(|(id, centroid)| {
                let mut count = 0.;
                let mut new_centroid = Point::origin();
                for (system, _) in cluster_map.iter().filter(|&(_, c_id)| *c_id == id) {
                    new_centroid += system.location.coords;
                    count += 1.;
                }
                new_centroid *= 1. / count;
                *centroid = new_centroid;
            });
    }

    // Setup cluster vectors
    let mut sector_vecs =
        (0..config.number_of_sectors).fold(Vec::<Vec<System>>::new(), |mut sectors, _| {
            sectors.push(vec![]);
            sectors
        });

    // Map systems to final cluster
    for (system, id) in cluster_map.into_iter() {
        sector_vecs[id].push(system);
    }

    // Unwrap and lock name generator as it is mutated by generation.
    let mut name_gen_unwraped = name_gen.lock().unwrap();
    name_gen_unwraped.reseed(config.number_of_sectors as u32);

    // Create sector for each cluster
    let sectors = sector_vecs
        .into_iter()
        .map(|systems| Sector {
            systems: systems,
            name: name_gen_unwraped
                .generate()
                .unwrap_or(String::from("Unnamed")) + " Sector",
            faction: Faction::random_faction(&mut rng),
        })
        .collect::<Vec<Sector>>();

    info!(
        "Mapped galaxy into {} sectors of {} systems, avg size: {}, 
          max size {}, min size {}, taking {} ms \n 
          Sectors include: {} Cartel, {} Empire, {} Federation, {} Independent",
        sectors.len(),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc + sec.systems.len()),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc + sec.systems.len()) / sectors.len(),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc.max(sec.systems.len())),
        sectors
            .iter()
            .fold(MAX, |acc, ref sec| acc.min(sec.systems.len())),
        ((now.elapsed().as_secs() * 1_000) + (now.elapsed().subsec_nanos() / 1_000_000) as u64),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc + match sec.faction {
                Faction::Cartel => 1,
                _ => 0,
            }),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc + match sec.faction {
                Faction::Empire => 1,
                _ => 0,
            }),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc + match sec.faction {
                Faction::Federation => 1,
                _ => 0,
            }),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc + match sec.faction {
                Faction::Independent => 1,
                _ => 0,
            })
    );

    sectors
}
