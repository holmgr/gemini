use rand;
use statrs::distribution::{Distribution, Gamma};
use generators::Gen;
use astronomicals::star::Star;

/// Basic non deterministic name generator for generating new Stars.
pub struct StarGen {
    mass_gen: Gamma,
}

impl StarGen {
    /// Create a new Star generator which loads the star resources needed.
    pub fn new() -> Self {
        let mass_gen = Gamma::new(2., 1.).unwrap();
        StarGen { mass_gen }
    }
}

impl Gen for StarGen {
    type GenItem = Star;

    /// Generates a new Star from the _distribution_ using the provided random
    /// generator.
    fn generate<R: rand::Rng>(&self, gen: &mut R) -> Option<Star> {
        // Do not want too small stars.
        let mass = self.mass_gen.sample(gen).max(0.1);

        // Mass-luminosity relation.
        let luminosity = mass.powf(3.5);
        Some(Star::new(mass, luminosity))
    }
}
