use std::{sync::{Arc, Mutex}, time::Instant};
use rand::{ChaChaRng, Rng, SeedableRng};
use rayon::prelude::*;
use statrs::distribution::{Distribution, Normal};

use utils::{HashablePoint, Point};
use resources::{fetch_resource, AstronomicalNamesResource};
use astronomicals::{hash, Galaxy, planet::{Planet, PlanetBuilder}, system::SystemBuilder};
use game_config::GameConfig;

pub mod names;
pub mod stars;
pub mod planets;
pub mod systems;
pub mod sectors;

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
    let sector_gen = sectors::SectorGen::new();
    let sectors = sector_gen.generate(
        config,
        &name_gen,
        locations
            .iter()
            .map(|point| HashablePoint::new(*point))
            .collect::<Vec<_>>(),
    );
    // Create System generator.
    let system_gen = systems::SystemGen::new();

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
                    systems.push(system_gen.generate(location.clone(), sector.faction.clone()));
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
                                .unwrap_or_else(|| String::from("Unnamed")),
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
                    .unwrap_or_else(|| String::from("Unnamed")),
            };

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
        ((now.elapsed().as_secs() * 1_000) + u64::from(now.elapsed().subsec_nanos() / 1_000_000))
    );

    Galaxy::new(sectors, systems)
}
