use petgraph::Graph;
use petgraph::visit::Bfs;
use petgraph::prelude::NodeIndex;
use rand::{Rng, SeedableRng, StdRng};

/// Basic Markov chain for generating random strings which are similar to the
/// trained data provided.
pub struct MarkovChain {
    graph: Graph<char, f64>,
    start: NodeIndex,
    end: NodeIndex,
    strings: Vec<String>,
    rng_generator: StdRng,
}

impl MarkovChain {
    /// Creates a new MarkovChain and training it with the provided strings,
    /// also configures the random generator with the given string.
    /// Providing the same training data and seed will produce the same set
    /// of names in the same order.
    pub fn new(starting_strings: &Vec<String>, seed: u32) -> MarkovChain {
        let max_len = starting_strings
            .iter()
            .clone()
            .max_by_key(|s| s.len())
            .unwrap()
            .len();
        let mut graph = Graph::<char, f64>::new();
        let start = graph.add_node('<');
        let end = graph.add_node('>');

        let alphabet: Vec<char> = String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ").chars().collect();

        // Temporary container of all nodes in previous layer
        let mut prev_layer = Vec::<NodeIndex>::new();
        prev_layer.push(start);

        // Initialize graph with layers, nodes, and edges.
        for _ in 0..max_len {
            let mut current_layer = vec![];

            for chr in alphabet.clone() {
                let current_node = graph.add_node(chr);
                current_layer.push(current_node);

                // Add edges to all nodes in previous layer
                for prev_node in &prev_layer {
                    graph.add_edge(prev_node.clone(), current_node, 0.0);
                }

                // Also include 'end' node so that words can be shorter
                graph.add_edge(current_node, end, 0.0);
            }
            prev_layer = current_layer;
        }

        // Create and initialize random generator using seed
        let new_seed: &[_] = &[seed.clone() as usize];
        let rng: StdRng = SeedableRng::from_seed(new_seed);

        let mut markov = MarkovChain {
            graph: graph,
            start: start,
            end: end,
            strings: starting_strings.clone(),
            rng_generator: rng,
        };

        // Train the model
        for string in starting_strings {
            markov.train(string);
        }

        // Normalize all edges such that the sum of all outgoing edges from a
        // node is 1
        markov.normalize();
        markov
    }

    /// Train the model with the given input string.
    /// Traverses the graph, going from start to end through the nodes with
    /// resulting in the given string.
    /// Increments all edges it walks over by one.
    fn train(&mut self, input: &String) {
        let mut chars = input.chars();
        let mut current_node = self.start;

        fn increment_edge(
            first_node: &NodeIndex,
            second_node: &NodeIndex,
            graph: &mut Graph<char, f64>,
        ) {
            let edge = graph.find_edge(*first_node, *second_node).unwrap();
            let current_edge_weight = graph.edge_weight(edge).unwrap().clone();
            graph.update_edge(*first_node, *second_node, current_edge_weight + 1.0);
        }

        while let Some(current_char) = chars.next() {
            let next_node = self.graph
                .neighbors(current_node)
                .find(|node| self.graph.node_weight(*node) == Some(&current_char))
                .unwrap();

            increment_edge(&current_node, &next_node, &mut self.graph);
            current_node = next_node;
        }

        increment_edge(&current_node, &self.end, &mut self.graph);
    }

    /// Normalizes all edges for all nodes such that the sum of the edge weights
    /// for all edges from a node is 1. I.e the edge weights will be normalized
    /// to a value in [0,1] reflecting the probability of that edge being used
    /// in the training data.
    fn normalize(&mut self) {
        let mut bfs = Bfs::new(&self.graph, self.start);
        while let Some(node) = bfs.next(&self.graph) {
            let edge_sum = match self.graph.edges(node).fold(0.0, |sum, edge| {
                sum + *edge.weight() as f64
            }) {
                x if x > 0.0 => x,
                _ => 1.0,
            };

            // Normalize edges for this node
            let mut edges = self.graph.neighbors(node).detach();
            while let Some(edge) = edges.next_edge(&self.graph) {
                self.graph[edge] /= edge_sum;
            }
        }
    }

    /// Generate a new string from the model, the new string cannot be longer
    /// than any string in the training data.
    /// Can currently generate duplicate strings, also may produce string which
    /// exist in the training data.
    pub fn generate(&mut self) -> String {
        let mut final_string = String::new();
        let mut current_node = self.start;

        // Traverse until we hit end
        while current_node != self.end {
            let mut sample = self.rng_generator.gen::<f64>();

            // Go through all edges, when an edge weight exceeds the random
            // value, go through that edge
            let mut edges = self.graph.neighbors(current_node).detach();
            while let Some(edge) = edges.next_edge(&self.graph) {
                sample -= self.graph[edge];

                // Go through this edge
                if sample <= 0.0 {
                    // Add current node value to the final string
                    final_string.push(*self.graph.node_weight(current_node).unwrap());
                    if let Some(nodes) = self.graph.edge_endpoints(edge) {
                        current_node = nodes.1;
                        break;
                    }
                }
            }
        }
        // TODO: Ensure that all generated strings are unique, and not part of
        // training data
        final_string.drain(0..1);
        final_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_SEED: u32 = 42;

    #[test]
    fn test_new_markov() {
        let starting_strings = vec![String::from("HELLO")];
        let markov = MarkovChain::new(&starting_strings, DEFAULT_SEED);
        assert_eq!(markov.graph.node_count(), 5 * 26 + 2);
        assert_eq!(markov.graph.edge_count(), 26 + 4 * 26 * 27 + 26);
    }

    #[test]
    fn test_train() {
        let training_str = String::from("ABC");
        let starting_strings = vec![training_str.clone()];
        let markov = MarkovChain::new(&starting_strings, DEFAULT_SEED);

        assert_eq!(
            markov.graph.edge_references().fold(0.0, |sum, edge| {
                sum + *edge.weight() as f64
            }),
            training_str.len() as f64 + 1.0
        );
    }

    #[test]
    fn test_normalize() {
        let starting_strings = vec![
            String::from("ABC"),
            String::from("ACC"),
            String::from("ACD"),
        ];
        let mut markov = MarkovChain::new(&starting_strings, DEFAULT_SEED);
        for string in &starting_strings {
            markov.train(string);
        }

        markov.normalize();
        assert!(markov.graph.raw_edges().iter().any(|ref edge| {
            edge.source() == markov.start && edge.target() == NodeIndex::new(2) &&
                edge.weight == 1.0
        }));
        assert!(markov.graph.raw_edges().iter().any(|ref edge| {
            edge.source() == NodeIndex::new(2) && edge.target() == NodeIndex::new(2 + 26 + 1) &&
                edge.weight == 1.0 / 3.0
        }));
        assert!(markov.graph.raw_edges().iter().any(|ref edge| {
            edge.source() == NodeIndex::new(2) && edge.target() == NodeIndex::new(2 + 26 + 2) &&
                edge.weight == 2.0 / 3.0
        }));
        assert!(markov.graph.raw_edges().iter().any(|ref edge| {
            edge.source() == NodeIndex::new(2 + 26 + 1) &&
                edge.target() == NodeIndex::new(2 + 2 * 26 + 2) && edge.weight == 1.0
        }));
        assert!(markov.graph.raw_edges().iter().any(|ref edge| {
            edge.source() == NodeIndex::new(2 + 26 + 2) &&
                edge.target() == NodeIndex::new(2 + 2 * 26 + 2) &&
                edge.weight == 1.0 / 2.0
        }));
        assert!(markov.graph.raw_edges().iter().any(|ref edge| {
            edge.source() == NodeIndex::new(2 + 26 + 2) &&
                edge.target() == NodeIndex::new(2 + 2 * 26 + 3) &&
                edge.weight == 1.0 / 2.0
        }));
        assert!(markov.graph.raw_edges().iter().any(|ref edge| {
            edge.source() == NodeIndex::new(2 + 2 * 26 + 3) && edge.target() == markov.end &&
                edge.weight == 1.0
        }));
        assert!(markov.graph.raw_edges().iter().any(|ref edge| {
            edge.source() == NodeIndex::new(2 + 2 * 26 + 2) && edge.target() == markov.end &&
                edge.weight == 1.0
        }));
    }

    #[test]
    fn test_generate() {
        let starting_strings = vec![
            String::from("ABCD"),
            String::from("ACCC"),
            String::from("ACDC"),
        ];
        let mut markov = MarkovChain::new(&starting_strings, DEFAULT_SEED);

        assert_eq!(markov.generate(), String::from("ABCC"));
        assert_eq!(markov.generate(), String::from("ACDC"));
        assert_eq!(markov.generate(), String::from("ACCC"));
    }
}
