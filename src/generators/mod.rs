use rand::{seq, ChaChaRng, Rng, SeedableRng};
use std::time::Instant;
use std::collections::HashMap;
use rayon::prelude::*;
use statrs::distribution::{Distribution, Normal, Poisson};
use nalgebra::distance;
use std::sync::{Arc, Mutex};
use std::iter::FromIterator;
use std::usize::MAX;
use std::f64;

pub mod names;
pub mod stars;
pub mod planets;

use utils::{HashablePoint, Point};
use resources::{fetch_resource, AstronomicalNamesResource};
use astronomicals::{hash, Galaxy};
use astronomicals::system::{SystemBuilder, SystemSecurity, SystemState};
use game_config::GameConfig;
use generators::stars::StarGen;
use generators::names::NameGen;
use generators::planets::PlanetGen;
use astronomicals::planet::{Planet, PlanetBuilder};
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
    let new_seed: &[_] = &[config.map_seed as u32];
    let mut rng: ChaChaRng = SeedableRng::from_seed(new_seed);

    // Measure time for generation.
    let now = Instant::now();

    // Clusters are spaced uniformly, systems gaussian.
    let loc_x = Normal::new(0., config.system_spread).unwrap();
    let loc_y = Normal::new(0., config.system_spread).unwrap();

    // Generate system locations.
    let mut locations = vec![];
    for _ in 0..config.number_of_systems {
        locations.push(Point::new(
            loc_x.sample::<ChaChaRng>(&mut rng),
            loc_y.sample::<ChaChaRng>(&mut rng),
        ))
    }

    // Create name generator to be shared mutably.
    let mut name_gen_unwraped = names::NameGen::from_seed(config.map_seed);
    name_gen_unwraped.train(&fetch_resource::<AstronomicalNamesResource>().unwrap());
    let name_gen = Arc::new(Mutex::new(name_gen_unwraped));

    // Generate sectors
    let sectors = into_sectors(
        config,
        name_gen.clone(),
        locations
            .iter()
            .map(|point| HashablePoint::new(point.clone()))
            .collect::<Vec<_>>(),
    );

    // Create Star generator.
    let star_gen = stars::StarGen::new();

    // Create Planet generator.
    let planet_gen = planets::PlanetGen::new();

    // Generate systems for each cluster in parallel.
    // Fold will generate one vector per thread (per cluster), reduce will
    // combine them to the final result.
    let builders = sectors
        .par_iter()
        .fold(
            || Vec::<(SystemBuilder, Vec<PlanetBuilder>)>::new(),
            |mut systems: Vec<(SystemBuilder, Vec<PlanetBuilder>)>, sector| {
                for location in &sector.system_locations {
                    // Generate system
                    systems.push(generate_system(
                        location.clone(),
                        sector.faction.clone(),
                        &star_gen,
                        &planet_gen,
                    ));
                }
                systems
            },
        )
        .reduce(
            || Vec::<(SystemBuilder, Vec<PlanetBuilder>)>::new(),
            |mut systems, subsystems| {
                systems.extend(subsystems);
                systems
            },
        );
    let systems = builders
        .into_iter()
        .map(|(mut system_builder, planet_builders)| {
            // Unwrap and lock name generator as it is mutated by generation.
            let hash = hash(&system_builder.location.unwrap());
            let seed: &[_] = &[hash as u32];
            let mut rng: ChaChaRng = SeedableRng::from_seed(seed);
            let mut name_gen_unwraped = name_gen.lock().unwrap();
            name_gen_unwraped.reseed(hash as u32);

            let planets: Vec<Planet> = planet_builders
                .into_iter()
                .map(|mut builder| {
                    builder
                        .name(
                            name_gen_unwraped
                                .generate()
                                .unwrap_or(String::from("Unnamed")),
                        )
                        .build()
                        .unwrap()
                })
                .collect();

            // System name is the same as one random planet.
            // Fallback to: Unnamed System if it contains no planets and no name could
            // be generated.
            let name = match rng.choose(&planets) {
                Some(planet) => planet.name.clone(),
                None => name_gen_unwraped
                    .generate()
                    .unwrap_or(String::from("Unnamed")),
            } + " System";

            system_builder
                .name(name)
                .satelites(planets)
                .build()
                .unwrap()
        })
        .collect::<Vec<_>>();

    info!(
        "Generated new galaxy containing: {} systems and {} planets taking {} ms",
        systems.len(),
        systems
            .iter()
            .fold(0, |acc, ref sys| acc + sys.satelites.len(),),
        ((now.elapsed().as_secs() * 1_000) + (now.elapsed().subsec_nanos() / 1_000_000) as u64)
    );

    Galaxy::new(sectors, systems)
}

/// Generate a new star system using the given generators and a location as seed.
pub fn generate_system(
    location: Point,
    faction: Faction,
    star_gen: &StarGen,
    planet_gen: &PlanetGen,
) -> (SystemBuilder, Vec<PlanetBuilder>) {
    // Calculate hash.
    let hash = hash(&location);
    let seed: &[_] = &[hash as u32];
    let mut rng: ChaChaRng = SeedableRng::from_seed(seed);

    let star = star_gen.generate(&mut rng).unwrap();

    // TODO: Replace constant in config.
    let num_planets = (Poisson::new(3.)
        .unwrap()
        .sample::<ChaChaRng>(&mut rng)
        .round() as u32)
        .max(1);

    // Fallback to planet name: Unnamed if no name could be generated.
    let satelites: Vec<PlanetBuilder> = (0..num_planets)
        .map(|_| {
            let mut builder = planet_gen.generate(&mut rng).unwrap();
            let mass = builder.mass.unwrap();
            let surface_temperature =
                Planet::calculate_surface_temperature(builder.orbit_distance.unwrap(), &star);
            let planet_type = Planet::predict_type(&mut rng, surface_temperature, mass);
            builder
                .surface_temperature(surface_temperature)
                .population(Planet::initial_population(mass, &planet_type))
                .planet_type(planet_type);
            builder
        })
        .collect();

    // Set the security level based on faction and a probability.
    let random_val: f64 = rng.gen();
    let security_level = match faction {
        Faction::Empire if random_val < 0.5 => SystemSecurity::High,
        Faction::Empire if random_val >= 0.5 => SystemSecurity::Medium,
        Faction::Federation if random_val < 0.4 => SystemSecurity::Low,
        Faction::Federation if random_val < 0.8 => SystemSecurity::Medium,
        Faction::Federation if random_val >= 0.8 => SystemSecurity::High,
        Faction::Cartel if random_val < 0.5 => SystemSecurity::Medium,
        Faction::Cartel if random_val >= 0.5 => SystemSecurity::Anarchy,
        Faction::Independent if random_val < 0.5 => SystemSecurity::Anarchy,
        Faction::Independent if random_val >= 0.5 => SystemSecurity::Low,
        _ => SystemSecurity::Low,
    };

    let mut system = SystemBuilder::default();
    system
        .location(location)
        .faction(faction)
        .security(security_level)
        .state(SystemState::Boom)
        .star(star);
    (system, satelites)
}

/// Split the systems in to a set number of clusters using K-means.
fn into_sectors(
    config: &GameConfig,
    name_gen: Arc<Mutex<NameGen>>,
    system_locations: Vec<HashablePoint>,
) -> Vec<Sector> {
    // Measure time for generation.
    let now = Instant::now();

    info!("Simulating expansion for initial sectors...");
    let seed: &[_] = &[config.map_seed as u32];
    let mut rng: ChaChaRng = SeedableRng::from_seed(seed);

    // Setup initial centroids
    let mut centroids =
        seq::sample_iter(&mut rng, system_locations.iter(), config.number_of_sectors)
            .unwrap()
            .into_iter()
            .map(|system_location| system_location.as_point().clone())
            .collect::<Vec<_>>();

    // Split data into two sets if using approximation
    let mut idx = 0;
    let (cluster_set, rest): (Vec<HashablePoint>, Vec<HashablePoint>) =
        system_locations.into_iter().partition(|_| {
            idx += 1;
            idx < config.num_approximation_systems || !config.sector_approximation
        });

    // System to cluster_id mapping
    let mut cluster_map: HashMap<HashablePoint, usize> =
        HashMap::from_iter(cluster_set.into_iter().map(|point| (point, 0)));

    debug!("Initial centroids: {:#?}", centroids);

    // Run K means until convergence, i.e until no reassignments
    let mut has_assigned = true;
    while has_assigned {
        let wrapped_assigned = Mutex::new(false);

        // Assign to closest centroid
        cluster_map
            .par_iter_mut()
            .for_each(|(system_location, cluster_id)| {
                let mut closest_cluster = *cluster_id;
                let mut closest_distance =
                    distance(system_location.as_point(), &centroids[*cluster_id]);
                for i in 0..centroids.len() {
                    let distance = distance(system_location.as_point(), &centroids[i]);
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
            //.par_iter_mut()
            .iter_mut()
            .enumerate()
            .for_each(|(id, centroid)| {
                let mut count = 0.;
                let mut new_centroid = Point::origin();
                for (system_location, _) in cluster_map.iter().filter(|&(_, c_id)| *c_id == id) {
                    new_centroid += system_location.as_point().coords;
                    count += 1.;
                }
                new_centroid *= 1. / count;
                *centroid = new_centroid;
            });
    }

    // Setup cluster vectors
    let mut sector_vecs =
        (0..config.number_of_sectors).fold(Vec::<Vec<HashablePoint>>::new(), |mut sectors, _| {
            sectors.push(vec![]);
            sectors
        });

    // Map systems to final cluster
    for (system_location, id) in cluster_map.into_iter() {
        sector_vecs[id].push(system_location);
    }

    // Assign remaining systems to closest centroid if any left
    rest.into_iter().for_each(|system_location| {
        let mut closest_cluster = 0;
        let mut closest_distance = f64::MAX;
        for i in 0..centroids.len() {
            let distance = distance(system_location.as_point(), &centroids[i]);
            if distance < closest_distance {
                closest_cluster = i;
                closest_distance = distance;
            }
        }
        sector_vecs[closest_cluster].push(system_location);
    });

    // Unwrap and lock name generator as it is mutated by generation.
    let mut name_gen_unwraped = name_gen.lock().unwrap();

    // Create sector for each cluster
    let sectors = sector_vecs
        .into_iter()
        .map(|system_locations| {
            let sector_seed: &[_] = &[system_locations.len() as u32];
            let mut faction_rng: ChaChaRng = SeedableRng::from_seed(sector_seed);
            name_gen_unwraped.reseed(*sector_seed.first().unwrap());
            Sector {
                system_locations: system_locations
                    .into_iter()
                    .map(|hashpoint| hashpoint.as_point().clone())
                    .collect::<Vec<_>>(),
                name: name_gen_unwraped
                    .generate()
                    .unwrap_or(String::from("Unnamed")) + " Sector",
                faction: Faction::random_faction(&mut faction_rng),
            }
        })
        .collect::<Vec<Sector>>();

    info!(
        "Mapped galaxy into {} sectors of {} systems, avg size: {}, 
          max size {}, min size {}, taking {} ms \n 
          Sectors include: {} Cartel, {} Empire, {} Federation, {} Independent",
        sectors.len(),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc + sec.system_locations.len()),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc + sec.system_locations.len()) / sectors.len(),
        sectors
            .iter()
            .fold(0, |acc, ref sec| acc.max(sec.system_locations.len())),
        sectors
            .iter()
            .fold(MAX, |acc, ref sec| acc.min(sec.system_locations.len())),
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
