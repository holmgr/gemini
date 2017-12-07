use rand::SeedableRng;
use std::time::Instant;
use rayon::prelude::*;
use rand::isaac::IsaacRng;
use statrs::distribution::{Distribution, Uniform, Normal};
use nalgebra::geometry::Point3 as Point;

pub mod names;

use astronomicals::{Galaxy, System};
use game_config::GameConfig;
use resources::{ResourceHandler, StarTypesResource};

/// Generic Generator trait to be implemented by concrete generators of different kinds.
pub trait Gen {
    type GenItem;
    type TrainData;

    /// Create a new genrator with the given seed for the random generator
    fn new(seed: u32) -> Self;

    /// Train the generator with the given data
    fn train(&mut self, &Self::TrainData);

    /// Generate a new item from the generator, can be None if the generator is empty etc.
    fn generate(&mut self) -> Option<Self::GenItem>;
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
                cluster_systems.push(System::new(Point::new(
                    norm_x.sample::<IsaacRng>(&mut rng),
                    norm_y.sample::<IsaacRng>(&mut rng),
                    norm_z.sample::<IsaacRng>(&mut rng),
                )));
            }
            cluster_systems
        })
        .reduce(|| Vec::<System>::new(), |mut systems, subsystems| {
            systems.extend(subsystems);
            systems
        });

    info!(
        "Generated new galaxy containing: {} clusters and {} systems taking {} ms",
        config.number_of_clusters,
        systems.len(),
        ((now.elapsed().as_secs() * 1_000) + (now.elapsed().subsec_nanos() / 1_000_000) as u64)
    );
    Galaxy::new(systems)
}
