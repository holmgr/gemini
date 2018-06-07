use rand::{ChaChaRng, Rng, SeedableRng};
use statrs::distribution::{Distribution, Poisson};
use std::f64;

use astronomicals::{planet::PlanetBuilder,
                    system::{Reputation, SystemBuilder, SystemSecurity, SystemState}};
use entities::Faction;
use generators::{planets::PlanetGen, stars::StarGen};
use utils::Point;

/// Used for generating systems.
pub struct SystemGen {
    num_planets_gen: Poisson,
    star_gen: StarGen,
    planet_gen: PlanetGen,
}

impl SystemGen {
    /// Create a new system generator.
    pub fn new() -> SystemGen {
        // Create Star generator.
        let star_gen = StarGen::new();

        // Create Planet generator.
        let planet_gen = PlanetGen::new();

        SystemGen {
            num_planets_gen: Poisson::new(3.).unwrap(),
            star_gen,
            planet_gen,
        }
    }

    /// Generate a new star system at the given location with the given faction.
    pub fn generate(
        &self,
        location: Point,
        faction: Faction,
    ) -> (SystemBuilder, Vec<PlanetBuilder>) {
        // Calculate hash.
        let hash = location.hash();
        let seed: &[_] = &[hash as u32];
        let mut rng = ChaChaRng::from_seed(seed);

        let star = self.star_gen.generate(&mut rng).unwrap();

        // TODO: Replace constant in config.
        let num_planets =
            (self.num_planets_gen.sample::<ChaChaRng>(&mut rng).round() as u32).max(1);

        // Fallback to planet name: Unnamed if no name could be generated.
        let satelites: Vec<PlanetBuilder> = (0..num_planets)
            .map(|_| {
                let mut builder = self.planet_gen.generate(&mut rng).unwrap();
                let mass = builder.mass.unwrap();
                let surface_temperature = PlanetGen::calculate_surface_temperature(
                    builder.orbit_distance.unwrap(),
                    &star,
                );
                let planet_type = PlanetGen::predict_type(&mut rng, surface_temperature, mass);
                let economic_type = PlanetGen::predict_economy(&mut rng, &planet_type);
                builder
                    .surface_temperature(surface_temperature)
                    .planet_type(planet_type)
                    .economic_type(economic_type);
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
            .reputation(Reputation::default())
            .star(star);
        (system, satelites)
    }
}
