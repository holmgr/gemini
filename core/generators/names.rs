use inflector::Inflector;
use rand::{ChaChaRng, Rng, SeedableRng};
use resources::AstronomicalNamesResource;
use statrs::distribution::{Categorical, Distribution};

/// Name generator which generates based on names given in training data.
pub struct NameGen {
    rng: ChaChaRng,
    base_names: Vec<String>,
    cache: Vec<String>,
    greek_suffix: Vec<String>,
    roman_suffix: Vec<String>,
    decorator_suffix: Vec<String>,
    scientific_names: Vec<String>,
}

impl NameGen {
    /// Creates a new NameGen with the given seed.
    pub fn from_seed(seed: u32) -> NameGen {
        // Create and initialize random generator using seed.
        let new_seed: &[_] = &[seed];
        let rng = ChaChaRng::from_seed(new_seed);

        NameGen {
            rng,
            cache: vec![],
            base_names: vec![],
            greek_suffix: vec![],
            roman_suffix: vec![],
            decorator_suffix: vec![],
            scientific_names: vec![],
        }
    }

    /// Reseeds the name generator.
    pub fn reseed(&mut self, seed: u32) {
        // Create and initialize random generator using seed.
        let new_seed: &[_] = &[seed];
        self.rng.reseed(new_seed);
    }

    /// Trains the underlying model using the given resouce.
    pub fn train(&mut self, data: AstronomicalNamesResource) {
        self.base_names = data.names;
        self.scientific_names = data.scientific_names;

        self.rng.shuffle(&mut self.base_names);
        self.rng.shuffle(&mut self.scientific_names);

        // Load suffixes.
        self.greek_suffix = data.greek;
        self.roman_suffix = data.roman;
        self.decorator_suffix = data.decorators;
    }

    /// Generates a new main name using base names and suffixes.
    fn generate_name(&mut self) -> Option<String> {
        if self.cache.is_empty() {
            if let Some(base_name) = self.base_names.pop() {
                let suffix_probs = Categorical::new(&[80., 8., 4., 8.]).unwrap();

                let suffixes = match suffix_probs.sample(&mut self.rng) as usize {
                    1 => self.greek_suffix.clone(),
                    2 => self.roman_suffix.clone(),
                    3 => self.decorator_suffix.clone(),
                    _ => vec![],
                };

                self.cache.push(base_name.to_title_case());
                for suffix in suffixes {
                    self.cache
                        .push(format!("{} {}", base_name.to_title_case(), suffix));
                }
            } else if !self.scientific_names.is_empty() {
                self.cache.append(&mut self.scientific_names);
            }
        }
        self.cache.pop()
    }

    /// Attempts to generate a new name main name and several secondary names from the model.
    pub fn generate(&mut self, subcount: usize) -> (String, Vec<String>) {
        let main_name = self.generate_name().unwrap_or_else(|| {
            error!("Failed to generate main system name");
            panic!()
        });

        let subname_type = Categorical::new(&[10., 40., 40.]).unwrap();

        let mut sub_names = vec![];
        let alphabet = vec![
            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P",
        ];

        match subname_type.sample(&mut self.rng) as usize {
            0 => {
                for _ in 1..subcount {
                    let sub_name = self
                        .generate_name()
                        .unwrap_or_else(|| String::from("Unnamed"));
                    sub_names.push(sub_name);
                }
                sub_names.push(main_name.clone());
            }
            1 => {
                for index in 1..subcount {
                    sub_names.push(format!("{} A-{}", main_name, index));
                }
            }
            _ => {
                for character in alphabet.iter().take(subcount) {
                    sub_names.push(format!("{} {}", main_name, character));
                }
            }
        };
        self.rng.shuffle(&mut sub_names);

        (main_name, sub_names)
    }
}
