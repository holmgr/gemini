use rand::{Rng, SeedableRng, StdRng};
use std::cmp::max;
use std::usize;
use std::collections::{BTreeSet, HashSet};
use petgraph::Graph;
use petgraph::prelude::NodeIndex;
use generators::{MutGen, SeedableGenerator, TrainableGenerator};
use resources::AstronomicalNamesResource;

/// Basic non deterministic name generator for generating random strings which
/// are similar to the trained data provided.
pub struct NameGen {
    rng: StdRng,
    generated: HashSet<String>,
    graph: Graph<char, f64>,
    start: NodeIndex,
    end: NodeIndex,
    suffixes: Vec<String>,
}

impl SeedableGenerator for NameGen {
    /// Creates a new NameGen with the given seed.
    fn from_seed(seed: u32) -> NameGen {

        // Create and initialize random generator using seed.
        let new_seed: &[_] = &[seed as usize];
        let rng: StdRng = SeedableRng::from_seed(new_seed);
        let mut graph = Graph::<char, f64>::new();
        let start = graph.add_node('<');
        let end = graph.add_node('>');
        let generated = HashSet::<String>::new();
        let suffixes = Vec::<String>::new();

        NameGen {
            rng,
            generated,
            graph,
            start,
            end,
            suffixes,
        }
    }

    /// Creates a new NameGen with the given seed.
    fn reseed(&mut self, seed: u32) {

        // Create and initialize random generator using seed.
        let new_seed: &[_] = &[seed as usize];
        let rng: StdRng = SeedableRng::from_seed(new_seed);
        self.rng = rng;
    }
}

impl TrainableGenerator for NameGen {
    type TrainRes = AstronomicalNamesResource;

    /// Trains the underlying model using the given AstronomicalNamesResource.
    fn train(&mut self, data: &AstronomicalNamesResource) {

        let depth = data.names.iter().fold(0, |acc, ref s| max(acc, s.len()));

        // Instansiate layers, number of layers is equal to the longest training
        // string.
        let mut layers = Vec::<BTreeSet<NodeIndex>>::new();
        for _ in 0..depth {
            layers.push(BTreeSet::<NodeIndex>::new());
        }

        // Add edges between all characters following each other at every
        // position producing a forward connected graph.
        for name in &data.names {
            let chars = name.chars();
            let mut prev = self.start;

            for (index, chr) in chars.enumerate() {
                let node = match layers[index].iter().find(|&&node| {
                    *self.graph.node_weight(node).unwrap() == chr
                }) {
                    Some(&node) => node,
                    _ => self.graph.add_node(chr),
                };
                layers[index].insert(node.clone());

                self.graph.update_edge(prev, node, 0.0);
                prev = node;
            }
            // Add connection to end from last character.
            self.graph.update_edge(prev, self.end, 0.0);
        }
        info!(
            "Model has been trained, is using {} layers, {} nodes and {} edges",
            depth,
            self.graph.node_count(),
            self.graph.edge_count()
        );

        // Load suffixes.
        self.suffixes.extend_from_slice(&data.greek[..]);
        self.suffixes.extend_from_slice(&data.decorators[..]);
    }
}

impl MutGen for NameGen {
    type GenItem = String;

    /// Attempts to generate a new name from the model.
    /// This name is guaranteed to exist in the training set or to have been
    /// previously generated.
    /// Attempts N number of tries, if no unique name could be found it will
    /// return None.
    fn generate(&mut self) -> Option<String> {

        // Non deterministicly generate a new string from the model.
        // Note: This may produce an exisiting string in the training set or
        // previously generated set.
        fn generate_attempt(
            graph: &Graph<char, f64>,
            start: &NodeIndex,
            end: &NodeIndex,
            rng: &mut StdRng,
        ) -> String {
            let mut final_string = String::new();
            let mut current_node = start.clone();

            // Traverse until we hit end.
            while current_node != end.clone() {

                // Step to random neighbor in next layer for which it exists
                // an edge.
                let neighbors = graph.neighbors(current_node).collect::<Vec<NodeIndex>>();
                let next_node = rng.choose(neighbors.as_slice()).unwrap();
                final_string.push(*graph.node_weight(current_node).unwrap());
                current_node = *next_node;
            }
            // Remove start node character.
            final_string.remove(0);
            final_string
        };

        // Randomly generate a suffix uniformlly from training data, has a high
        // chance of returning None.
        // TODO: Load threshold from config.
        fn get_suffix(rng: &mut StdRng, suffixes: &Vec<String>) -> Option<String> {
            let suffix_chance = 0.1;
            match rng.next_f64() {
                x if x < suffix_chance => Some(rng.choose(&suffixes[..]).unwrap().clone()),
                _ => None,
            }
        }

        // Check if a name is valid according to constraints.
        // TODO: Extract this to a seperate method, should also probably be
        // based on configuration entries.
        let is_valid_name = |name: &String| name.contains(" ") || name.len() < 9;

        // Attempt N number of attempts retuning none if no unique string was
        // generated which fullfils the criteria.
        // TODO: Move name retries to config.
        let gen_num_attempts = 27;
        for _ in 0..gen_num_attempts {
            let name = generate_attempt(&self.graph, &self.start, &self.end, &mut self.rng);
            if is_valid_name(&name) && !self.generated.contains(&name) {
                self.generated.insert(name.clone());
                return match get_suffix(&mut self.rng, &self.suffixes) {
                    Some(suffix) => Some(format!("{} {}", name, &suffix)),
                    _ => Some(name),
                };
            }
        }
        info!("Unsuccessful generation used {} tries", gen_num_attempts);
        None
    }
}

#[cfg(test)]
mod names_test {
    use super::*;
    extern crate env_logger;
    use resources::{AstronomicalNamesResource, fetch_resource};

    #[test]
    // All genrated names must be unique.
    fn test_generate_unique() {
        let _ = env_logger::init();

        let mut gen = NameGen::from_seed(0);
        let res = fetch_resource::<AstronomicalNamesResource>().unwrap();
        gen.train(&res);

        let mut names = HashSet::<String>::new();
        for _ in 0..30 {
            names.insert(gen.generate().unwrap());
        }

        assert_eq!(names.len(), 30);
    }
}
