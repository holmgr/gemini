use rand::{thread_rng, Rng, SeedableRng};
use std::time::Instant;
use rayon::prelude::*;
use rand::isaac::IsaacRng;
use statrs::distribution::{Distribution, Gamma, Normal, Uniform};
use nalgebra::geometry::Point3 as Point;
use nalgebra::distance;
use std::sync::{Arc, Mutex};
use petgraph::Graph;
use petgraph::Undirected;
use petgraph::algo::tarjan_scc;
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
    let new_seed: &[_] = &[config.map_seed];
    let mut rng: IsaacRng = SeedableRng::from_seed(new_seed);

    // Measure time for generation.
    let now = Instant::now();

    // Clusters are spaced uniformly, systems gaussian.
    let cluster_loc_gen = Uniform::new(0., config.galaxy_size).unwrap();
    let system_loc_gen = Normal::new(config.cluster_size_mean, config.cluster_size_std).unwrap();

    // Generate clusters.
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
    let systems = clusters
        .into_par_iter()
        .fold(
            || Vec::<System>::new(),
            |mut cluster_systems: Vec<System>, (cluster_pos, cluster_size)| {
                // Generate x,y,z generators based at cluster location
                let norm_x = Normal::new(cluster_pos.coords.x, config.cluster_spread).unwrap();
                let norm_y = Normal::new(cluster_pos.coords.y, config.cluster_spread).unwrap();
                let norm_z = Normal::new(cluster_pos.coords.z, config.cluster_spread).unwrap();

                // TODO: Do something smarter than cloning state of generator,
                // since all clusters will be generated identically now.
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
        "Generated new galaxy containing: {} clusters and {} systems and {} planets taking {} ms",
        config.number_of_clusters,
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
    let seed: &[_] = &[hash as u32];
    let mut rng: IsaacRng = SeedableRng::from_seed(seed);

    let star = star_gen.generate(&mut rng).unwrap();

    // Unwrap and lock name generator as it is mutated by generation.
    let mut name_gen_unwraped = name_gen.lock().unwrap();
    name_gen_unwraped.reseed(hash as u32);

    // TODO: Replace constant in config.
    let num_planets = Gamma::new(1., 0.5)
        .unwrap()
        .sample::<IsaacRng>(&mut rng)
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

/// Groups systems into seperate sectors using SCC
pub fn into_sectors(
    config: &GameConfig,
    name_gen: Arc<Mutex<NameGen>>,
    systems: Vec<System>,
) -> Vec<Sector> {
    // Measure time for generation.
    let now = Instant::now();

    info!("Splitting into connected components of sectors using SCC...");
    let mut graph: Graph<System, f64, Undirected> =
        Graph::with_capacity(systems.len(), systems.len().pow(2));

    // Create a fully connected graph except edges further than max_sector_dist
    let mut nodes = vec![];
    for system in systems {
        let new_node = graph.add_node(system);
        for node in &nodes {
            let dist = distance(
                &graph.node_weight(new_node).unwrap().location,
                &graph.node_weight(*node).unwrap().location,
            );

            if dist <= config.max_sector_dist {
                graph.update_edge(new_node, *node, dist);
            }
        }

        nodes.push(new_node);
    }

    // Split into connected components using Tarjan and map to sectors in parallel
    let sectors: Vec<Sector> = tarjan_scc(&graph)
        .into_par_iter()
        .map(|group| {
            let vect: Vec<System> = group.iter().fold(Vec::<System>::new(), |mut res, node_id| {
                res.push(graph.node_weight(*node_id).unwrap().clone());
                res
            });

            // Generate name from sector
            let mut name_gen_unwraped = name_gen.lock().unwrap();
            name_gen_unwraped.reseed((hash as u32) * (vect.len() as u32));
            let sector_name = name_gen_unwraped
                .generate()
                .unwrap_or(String::from("Unnamed")) + " Sector";

            // Set Faction for sector
            let faction = Faction::random_faction(&mut thread_rng());

            Sector {
                systems: vect,
                faction: faction,
                name: sector_name,
            }
        })
        .collect();
    info!("Mapped Galaxy into {} sectors of {} systems, avg size: {}, max size {}, min size {}, taking {} ms",
           sectors.len(),
           sectors.iter().fold(0, |acc, ref sec| {
               acc + sec.systems.len() }),
           sectors.iter().fold(0, |acc, ref sec| {
               acc + sec.systems.len() }) / sectors.len(),
           sectors.iter().fold(0, |acc, ref sec| {
               acc.max(sec.systems.len()) }),
           sectors.iter().fold(MAX, |acc, ref sec| {
               acc.min(sec.systems.len()) }),
        ((now.elapsed().as_secs() * 1_000) + (now.elapsed().subsec_nanos() / 1_000_000) as u64)
    );
    info!(
        "Sectors include: {} Cartel, {} Empire, {} Federation, {} Independent",
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
