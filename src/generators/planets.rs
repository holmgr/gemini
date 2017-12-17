use rand::Rng;
use statrs::distribution::{Distribution, Exponential, Gamma};
use resources::PlanetTypesResource;
use generators::{TrainableGenerator, Gen};
use astronomicals::planet::Planet;

/// Basic non deterministic name generator for generating new Planets which
/// are similar to the trained data provided.
pub struct PlanetGen {
    mass_gen: Exponential,
    orbit_dist_gen: Gamma,
}

impl PlanetGen {
    /// Minimal distance to star
    // TODO: Move to config
    const MIN_DIST: f64 = 500.;

    /// Create a new Planet generator which loads the star resources needed
    pub fn new() -> Self {
        let mass_gen = Exponential::new(1.432).unwrap();
        let orbit_dist_gen = Gamma::new(0.28, 0.17).unwrap();
        PlanetGen {
            mass_gen,
            orbit_dist_gen,
        }
    }
}

impl TrainableGenerator for PlanetGen {
    type TrainRes = PlanetTypesResource;

    /// Train the generator with the given data
    fn train(&mut self, _: &PlanetTypesResource) {}
}


impl Gen for PlanetGen {
    type GenItem = Planet;

    /// Generates a new Planet from the _distribution_ using the provided random
    /// generator
    fn generate<R: Rng>(&self, gen: &mut R) -> Option<Planet> {
        let mass = self.mass_gen.sample(gen);

        // Magic constant, needed to scale back since scaling needed to fit gamma
        let orbit_distance = PlanetGen::MIN_DIST + 1000. * self.orbit_dist_gen.sample(gen);

        Some(Planet::new(mass, orbit_distance))
    }
}
