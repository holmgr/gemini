use rand::{ChaChaRng, SeedableRng};
use rayon::prelude::*;
use statrs::distribution::{Distribution, Normal};
use std::time::Instant;

use astronomicals::{planet::{Planet, PlanetBuilder},
                    system::SystemBuilder,
                    Galaxy};
use game_config::GameConfig;
use resources::{fetch_resource, AstronomicalNamesResource};
use utils::Point;

pub mod names;
pub mod planets;
pub mod sectors;
pub mod stars;
pub mod systems;

/// Generate a galaxy with systems etc, will use the provided config to guide
/// the generation.
pub fn generate_galaxy(config: &GameConfig) -> Galaxy {
    let new_seed: &[_] = &[config.map_seed as u32];
    let mut rng = ChaChaRng::from_seed(new_seed);

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
    let mut name_gen = names::NameGen::from_seed(config.map_seed);
    name_gen.train(fetch_resource::<AstronomicalNamesResource>().unwrap());

    // Generate sectors
    let sector_gen = sectors::SectorGen::new();
    let sectors = sector_gen.generate(config, locations);
    // Create System generator.
    let system_gen = systems::SystemGen::new();

    // Generate systems for each cluster in parallel.
    // Fold will generate one vector per thread (per cluster), reduce will
    // combine them to the final result.
    let mut builders = sectors
        .par_iter()
        .fold(
            Vec::<(SystemBuilder, Vec<PlanetBuilder>)>::new,
            |mut systems: Vec<(SystemBuilder, Vec<PlanetBuilder>)>, sector| {
                for location in &sector.system_locations {
                    // Generate system
                    systems.push(system_gen.generate(location.clone(), sector.faction.clone()));
                }
                systems
            },
        )
        .reduce(
            Vec::<(SystemBuilder, Vec<PlanetBuilder>)>::new,
            |mut systems, subsystems| {
                systems.extend(subsystems);
                systems
            },
        );

    // Sort to ensure that naming etc, will be deterministic.
    builders.sort_by_key(|&(ref system_builder, _)| system_builder.location.unwrap().hash());

    let systems = builders
        .into_iter()
        .map(|(mut system_builder, planet_builders)| {
            let hash = system_builder.location.unwrap().hash();
            name_gen.reseed(hash as u32);

            let (system_name, planet_names) = name_gen.generate(planet_builders.len());

            let planets: Vec<Planet> = planet_builders
                .into_iter()
                .zip(planet_names.into_iter())
                .map(|(mut builder, name)| builder.name(name).build().unwrap())
                .collect();

            system_builder
                .name(system_name)
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
        ((now.elapsed().as_secs() * 1_000) + u64::from(now.elapsed().subsec_nanos() / 1_000_000))
    );

    Galaxy::new(sectors, systems)
}
