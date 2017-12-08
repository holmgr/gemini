use rand::Rng;
use resources::{ResourceHandler, StarTypesResource, StarType};
use astronomicals::Star;

/// Basic non deterministic name generator for generating new Stars which
/// are similar to the trained data provided.
pub struct StarGen {
    star_types: Vec<StarType>,
}

impl StarGen {
    /// Create a new Star generator which loads the star resources needed
    pub fn new() -> Self {
        let star_resources = ResourceHandler::new()
            .fetch_resource::<StarTypesResource>()
            .unwrap();
        let mut star_types = vec![];
        star_types.extend(star_resources.main_sequence);
        star_types.extend(star_resources.giant);
        star_types.extend(star_resources.supergiant);

        StarGen { star_types }
    }

    /// Generates a new Star from the _distribution_ using the provided random
    /// generator
    pub fn generate<R: Rng>(&self, gen: &mut R) -> Star {
        // Data may not sum to 1
        let max_abundance = self.star_types.iter().fold(0., |abund, ref star_type| {
            abund + star_type.abundance
        });

        // Select one star type based on probabilities
        let mut star_prob = gen.next_f64().min(max_abundance);
        let selected_star = self.star_types
            .iter()
            .find(|&star_type| {
                star_prob -= star_type.abundance;
                star_prob <= 0.
            })
            .unwrap();

        Star::new(selected_star.mass, selected_star.luminosity, 0.0)
    }
}
