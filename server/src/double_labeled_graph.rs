use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use adf_bdd::adf::Adf;
use adf_bdd::datatypes::{Term, Var};

#[derive(Clone, Deserialize, Serialize, Debug)]
/// This is a DTO for the graph output
pub struct DoubleLabeledGraph {
    // number of nodes equals the number of node labels
    // nodes implicitly have their index as their ID
    node_labels: HashMap<String, String>,
    // every node gets this label containing multiple entries (it might be empty)
    tree_root_labels: HashMap<String, Vec<String>>,
    lo_edges: Vec<(String, String)>,
    hi_edges: Vec<(String, String)>,
}

impl DoubleLabeledGraph {
    pub fn from_adf_and_ac(adf: &Adf, ac: Option<&Vec<Term>>) -> Self {
        let ac: &Vec<Term> = match ac {
            Some(ac) => ac,
            None => &adf.ac,
        };

        let mut node_indices: HashSet<usize> = HashSet::new();
        let mut new_node_indices: HashSet<usize> = ac.iter().map(|term| term.value()).collect();

        while !new_node_indices.is_empty() {
            node_indices = node_indices.union(&new_node_indices).copied().collect();
            new_node_indices = HashSet::new();

            for node_index in &node_indices {
                let lo_node_index = adf.bdd.nodes[*node_index].lo().value();
                if !node_indices.contains(&lo_node_index) {
                    new_node_indices.insert(lo_node_index);
                }

                let hi_node_index = adf.bdd.nodes[*node_index].hi().value();
                if !node_indices.contains(&hi_node_index) {
                    new_node_indices.insert(hi_node_index);
                }
            }
        }

        let node_labels: HashMap<String, String> = adf
            .bdd
            .nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| node_indices.contains(i))
            .map(|(i, &node)| {
                let value_part = match node.var() {
                    Var::TOP => "TOP".to_string(),
                    Var::BOT => "BOT".to_string(),
                    _ => adf.ordering.name(node.var()).expect(
                        "name for each var should exist; special cases are handled separately",
                    ),
                };

                (i.to_string(), value_part)
            })
            .collect();

        let tree_root_labels_with_usize: HashMap<usize, Vec<String>> = ac.iter().enumerate().fold(
            adf.bdd
                .nodes
                .iter()
                .enumerate()
                .filter(|(i, _)| node_indices.contains(i))
                .map(|(i, _)| (i, vec![]))
                .collect(),
            |mut acc, (root_for, root_node)| {
                acc.get_mut(&root_node.value())
                    .expect("we know that the index will be in the map")
                    .push(adf.ordering.name(Var(root_for)).expect(
                        "name for each var should exist; special cases are handled separately",
                    ));

                acc
            },
        );

        let tree_root_labels: HashMap<String, Vec<String>> = tree_root_labels_with_usize
            .into_iter()
            .map(|(i, vec)| (i.to_string(), vec))
            .collect();

        let lo_edges: Vec<(String, String)> = adf
            .bdd
            .nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| node_indices.contains(i))
            .filter(|(_, node)| !vec![Var::TOP, Var::BOT].contains(&node.var()))
            .map(|(i, &node)| (i, node.lo().value()))
            .map(|(i, v)| (i.to_string(), v.to_string()))
            .collect();

        let hi_edges: Vec<(String, String)> = adf
            .bdd
            .nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| node_indices.contains(i))
            .filter(|(_, node)| !vec![Var::TOP, Var::BOT].contains(&node.var()))
            .map(|(i, &node)| (i, node.hi().value()))
            .map(|(i, v)| (i.to_string(), v.to_string()))
            .collect();

        DoubleLabeledGraph {
            node_labels,
            tree_root_labels,
            lo_edges,
            hi_edges,
        }
    }
}
