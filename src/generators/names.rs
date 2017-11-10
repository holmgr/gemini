use rand::{Rng, SeedableRng, StdRng};
use std::collections::{BTreeSet, HashMap, HashSet};
use petgraph::{Graph, Direction};
use petgraph::visit::Bfs;
use petgraph::prelude::NodeIndex;
use generators::Gen;
use resources::AstronomicalNamesResource as anr;
use rayon::prelude::*;

pub struct NameGen {
    rng: StdRng,
}

impl Gen for NameGen {
    type GenItem = String;
    type TrainData = anr;

    fn new(seed: u32) -> NameGen {

        // Create and initialize random generator using seed
        let new_seed: &[_] = &[seed.clone() as usize];
        let rng: StdRng = SeedableRng::from_seed(new_seed);
        NameGen {rng}
    }

    fn train(&mut self, data: &anr) {
        let max_len = data.names
            .iter()
            .clone()
            .max_by_key(|s| s.len())
            .unwrap()
            .len();
        let mut graph = Graph::<char, f64>::new();
        let start = graph.add_node('<');
        let end = graph.add_node('>');

        // Store each unique character per index in repsective layer
        let mut layers: Vec<BTreeSet<char>> = vec![];
        for _ in 0..max_len {
            let set = BTreeSet::new();
            layers.push(set);
        }
        for string in data.names.clone() {
            for (i, chr) in string.chars().enumerate() {
                layers[i].insert(chr);
            }
        }

        // Temporary container of all nodes in previous layer
        let mut prev_layer = Vec::<NodeIndex>::new();
        prev_layer.push(start);

        // Setup fully forward connecting edges between each layer
        for layer in layers {
            let mut current_layer = vec![];

            for chr in layer.iter() {
                let current_node = graph.add_node(*chr);
                current_layer.push(current_node);

                // Add edges to all nodes in previous layer
                for prev_node in &prev_layer {
                    graph.add_edge(*prev_node, current_node, 0.0);
                }

                // Also include 'end' node so that words can be shorter
                graph.add_edge(current_node, end, 0.0);
            }
            prev_layer = current_layer;
        }
        
        fn edge_fix(graph: &mut Graph<char, f64>, start: NodeIndex, end: NodeIndex, input: String) {
            let mut chars = input.chars();
            let mut current_node = start;

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
                let next_node = graph
                    .neighbors(current_node)
                    .find(|node| graph.node_weight(*node) == Some(&current_char))
                    .unwrap();

                increment_edge(&current_node, &next_node, graph);
                current_node = next_node;
            }

            increment_edge(&current_node, &end, graph);
        }

        for string in data.names.clone() {
            edge_fix(&mut graph, start, end, string);
        }
        graph.retain_edges(|graph, edge| graph.edge_weight(edge).unwrap() > &0.0);
        println!("{}, {}", graph.node_count(), graph.edge_count());

        let mut prefixes = HashMap::<NodeIndex, Vec<String>>::new();
        let mut bfs = Bfs::new(&graph, start);
        while let Some(node) = bfs.next(&graph) {

            if node == end {
                continue;
            }

            let mut prefix = Vec::<String>::new();
            if node == start {
                prefix.push(String::new());
                prefixes.insert(node, prefix);
                continue;
            }

            //println!("OK before prev check");

            for previous_node in graph.neighbors_directed(node, Direction::Incoming) {
                let prev_prefix = prefixes.get(&previous_node).unwrap().to_owned();
                prefix.extend(prev_prefix);
            }

            //println!("OK after prev check");

            let node_char = *graph.node_weight(node).unwrap();
            prefix.par_iter_mut().for_each(|val: &mut String| {
                val.push(node_char);
            });

            //println!("Finished node: {:?}, prefixes is {}", node, prefix.len());
            prefixes.insert(node, prefix);
        }

        let mut prefix = Vec::<String>::new();
        for previous_node in graph.neighbors_directed(end, Direction::Incoming) {
            let prev_prefix = prefixes.get(&previous_node).unwrap().to_owned();
            prefix.extend(prev_prefix);
        }

        //println!("OK after prev check");
        //println!("{:?}", prefix.len());
        let mut nameSet = HashSet::<String>::new();
        data.names.clone().into_iter().for_each(|word| { nameSet.insert(word.clone()); });
        prefix = prefix.into_iter().filter(|word| word.len() < 12 && !nameSet.contains(word)).collect();
        println!("{:?}", prefix.len());
        for _ in 0..1000 {
            //println!("{:?}", self.rng.choose(&prefix));
        }
    }

    fn generate(&mut self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod names_test {
    use super::*;
    use resources::{AstronomicalNamesResource, ResourceHandler};

    #[test]
    fn test_train() {
       let mut gen = NameGen::new(0); 
       let factory = ResourceHandler::new();
       let res = factory
           .fetch_resource::<AstronomicalNamesResource>()
           .unwrap();
       gen.train(&res);
    }
}
