use rand::SeedableRng;
use std::time::Instant;
use rand::isaac::IsaacRng;
use statrs::distribution::{Distribution, Uniform, Normal};
use nalgebra::geometry::Point3 as Point;

use astronomicals::{Galaxy, System};
use game_config::GameConfig;

pub mod names;

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

    let now = Instant::now();

    // Generate clusters
    // Clusters are spaced uniformly, systems gaussian
    let mut systems = Vec::<System>::new();

    let cluster_loc_gen = Uniform::new(0., config.galaxy_size).unwrap();
    let system_loc_gen = Normal::new(config.cluster_size_mean, config.cluster_size_std).unwrap();

    // TODO: Generate systems for each cluster in parallel
    for _ in 0..config.number_of_clusters {
        let cluster_pos = Point::new(
            cluster_loc_gen.sample::<IsaacRng>(&mut rng),
            cluster_loc_gen.sample::<IsaacRng>(&mut rng),
            cluster_loc_gen.sample::<IsaacRng>(&mut rng),
        );

        // Add randomness to cluster size
        let cluster_size = system_loc_gen.sample::<IsaacRng>(&mut rng) as u64;

        // Generate x,y,z generators based at cluster location
        let norm_x = Normal::new(cluster_pos.coords.x, config.cluster_spread).unwrap();
        let norm_y = Normal::new(cluster_pos.coords.y, config.cluster_spread).unwrap();
        let norm_z = Normal::new(cluster_pos.coords.z, config.cluster_spread).unwrap();

        // Generate systems
        for _ in 0..cluster_size {
            systems.push(System::new(Point::new(
                norm_x.sample::<IsaacRng>(&mut rng),
                norm_y.sample::<IsaacRng>(&mut rng),
                norm_z.sample::<IsaacRng>(&mut rng),
            )));
        }
    }

    info!(
        "Generated new galaxy containing: {} clusters and {} systems taking {} seconds",
        config.cluster_size_mean,
        systems.len(),
        now.elapsed().as_secs()
    );
    Galaxy::new(systems)
}
