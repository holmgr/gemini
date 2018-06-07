use nalgebra::distance;
use rand::{seq, ChaChaRng, SeedableRng};
use rayon::prelude::*;
use std::{collections::HashMap,
          f64,
          iter::FromIterator,
          sync::atomic::{AtomicBool, Ordering},
          time::Instant,
          usize::MAX};

use astronomicals::sector::Sector;
use entities::Faction;
use game_config::GameConfig;
use utils::{HashablePoint, Point};

/// Used for generating sectors.
pub struct SectorGen {}

impl SectorGen {
    /// Create a new sector generator.
    pub fn new() -> SectorGen {
        SectorGen {}
    }

    /// Split the systems in to a set number of clusters using K-means.
    pub fn generate(
        &self,
        config: &GameConfig,
        system_locations: Vec<HashablePoint>,
    ) -> Vec<Sector> {
        // Measure time for generation.
        let now = Instant::now();

        info!("Simulating expansion for initial sectors...");
        let seed: &[_] = &[config.map_seed as u32];
        let mut rng: ChaChaRng = ChaChaRng::from_seed(seed);

        // Setup initial centroids
        let mut centroids =
            seq::sample_iter(&mut rng, system_locations.iter(), config.number_of_sectors)
                .unwrap()
                .into_iter()
                .map(|system_location| *system_location.as_point())
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

        // Run K means until convergence, i.e until no reassignments
        let mut has_assigned = true;
        while has_assigned {
            let wrapped_assigned = AtomicBool::new(false);

            // Assign to closest centroid
            cluster_map
                .par_iter_mut()
                .for_each(|(system_location, cluster_id)| {
                    let mut closest_cluster = *cluster_id;
                    let mut closest_distance =
                        distance(system_location.as_point(), &centroids[*cluster_id]);
                    for (i, centroid) in centroids.iter().enumerate() {
                        let distance = distance(system_location.as_point(), centroid);
                        if distance < closest_distance {
                            wrapped_assigned.store(true, Ordering::Relaxed);
                            closest_cluster = i;
                            closest_distance = distance;
                        }
                    }
                    *cluster_id = closest_cluster;
                });

            has_assigned = wrapped_assigned.load(Ordering::Relaxed);

            // Calculate new centroids
            centroids
                //.par_iter_mut()
                .iter_mut()
                .enumerate()
                .for_each(|(id, centroid)| {
                    let mut count = 0.;
                    let mut new_centroid = Point::origin();
                    for (system_location, _) in cluster_map.iter()
                        .filter(|&(_, c_id)| *c_id == id) {
                            new_centroid += system_location.as_point().coords;
                            count += 1.;
                        }
                    new_centroid *= 1. / count;
                    *centroid = new_centroid;
                });
        }

        // Setup cluster vectors
        let mut sector_vecs = (0..config.number_of_sectors).fold(
            Vec::<Vec<HashablePoint>>::new(),
            |mut sectors, _| {
                sectors.push(vec![]);
                sectors
            },
        );

        // Map systems to final cluster
        for (system_location, id) in cluster_map {
            sector_vecs[id].push(system_location);
        }

        // Assign remaining systems to closest centroid if any left
        rest.into_iter().for_each(|system_location| {
            let mut closest_cluster = 0;
            let mut closest_distance = f64::MAX;
            for (i, centroid) in centroids.iter().enumerate() {
                let distance = distance(system_location.as_point(), centroid);
                if distance < closest_distance {
                    closest_cluster = i;
                    closest_distance = distance;
                }
            }
            sector_vecs[closest_cluster].push(system_location);
        });

        // Create sector for each cluster
        let sectors = sector_vecs
            .into_iter()
            .map(|system_locations| {
                let sector_seed: &[_] = &[system_locations.len() as u32];
                let mut faction_rng: ChaChaRng = SeedableRng::from_seed(sector_seed);
                Sector {
                    system_locations: system_locations
                        .into_iter()
                        .map(|hashpoint| *hashpoint.as_point())
                        .collect::<Vec<_>>(),
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
                .fold(0, |acc, ref sec| acc + sec.system_locations.len())
                / sectors.len(),
            sectors
                .iter()
                .fold(0, |acc, ref sec| acc.max(sec.system_locations.len())),
            sectors
                .iter()
                .fold(MAX, |acc, ref sec| acc.min(sec.system_locations.len())),
            ((now.elapsed().as_secs() * 1_000)
                + u64::from(now.elapsed().subsec_nanos() / 1_000_000)),
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
}
