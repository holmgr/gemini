use rand::Rng;
use resources::{StarTypesResource, StarType};
use generators::{TrainableGenerator, Gen};
use astronomicals::Star;

/// Basic non deterministic name generator for generating new Stars which
/// are similar to the trained data provided.
pub struct StarGen {
    star_types: Vec<StarType>,
}

impl StarGen {
    /// Create a new Star generator which loads the star resources needed
    pub fn new() -> Self {
        let star_types = Vec::<StarType>::new();
        StarGen { star_types }
    }
}

impl TrainableGenerator for StarGen {
    type TrainRes = StarTypesResource;

    /// Train the generator with the given data
    fn train(&mut self, data: &StarTypesResource) {
        let mut star_types = vec![];
        star_types.extend_from_slice(&data.main_sequence);
        star_types.extend_from_slice(&data.giant);
        star_types.extend_from_slice(&data.supergiant);
        self.star_types = star_types;
    }
}


impl Gen for StarGen {
    type GenItem = Star;

    /// Generates a new Star from the _distribution_ using the provided random
    /// generator
    fn generate<R: Rng>(&self, gen: &mut R) -> Option<Star> {
        // Data may not sum to 1
        let max_abundance = self.star_types.iter().fold(0., |abund, ref star_type| {
            abund + star_type.abundance
        });

        // Select one star type based on probabilities
        let mut star_prob = gen.next_f64().min(max_abundance);
        match self.star_types.iter().find(|&star_type| {
            star_prob -= star_type.abundance;
            star_prob <= 0.
        }) {
            Some(selected_star) => Some(
                Star::new(selected_star.mass, selected_star.luminosity, 0.0),
            ),
            _ => None,
        }

    }
}
