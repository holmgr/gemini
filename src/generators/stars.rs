use rand;
use statrs::distribution::{Distribution, Gamma};
use astronomicals::star::{Star, StarType};

/// Basic non deterministic name generator for generating new Stars.
pub struct StarGen {
    mass_gen: Gamma,
}

impl StarGen {
    /// Create a new Star generator which loads the star resources needed.
    pub fn new() -> Self {
        let mass_gen = Gamma::new(2., 1.5).unwrap();
        StarGen { mass_gen }
    }

    /// Generates a new Star from the _distribution_ using the provided random
    /// generator.
    pub fn generate<R: rand::Rng>(&self, gen: &mut R) -> Option<Star> {
        // Do not want too small stars.
        let mass = self.mass_gen.sample(gen).max(0.1);

        // Stars with high mass are binary stars.
        let startype = if mass > 3. {
            StarType::Binary
        } else {
            StarType::Single
        };

        // Mass-luminosity relation.
        let luminosity = mass.powf(3.5);
        Some(Star::new(mass, luminosity, startype))
    }
}
