//! Rebuild each community from local_move as smaller groups.
//!
//! `original_partition` stays fixed and decides which nodes may merge.
//! `refined` starts with every node alone and records the groups being built.
//! Here "group" always means a community in `refined`.

use std::collections::HashMap;

use crate::graph::Graph;
use crate::modularity::move_gain;
use crate::partition::Partition;
use crate::rng::Rng;

/// Refine each original community independently. Nodes never cross its border.
pub(crate) fn refine_partition(
    graph: &Graph,
    original_partition: &Partition,
    resolution: f64,
    randomness: f64,
    rng: &mut Rng,
) -> Partition {
    let members = original_partition.members();
    let mut refined = Partition::singleton(graph);

    for (original_community, nodes) in members.iter().enumerate() {
        if nodes.len() <= 1 {
            continue;
        }
        let mut refiner = CommunityRefiner {
            graph,
            original_partition,
            original_community,
            refined: &mut refined,
            resolution,
            randomness,
            weight_to_rest_of_community: HashMap::new(),
        };

        refiner.merge_nodes(nodes, rng);
    }
    refined
}

/// State used while processing one original community.
struct CommunityRefiner<'a> {
    graph: &'a Graph,
    original_partition: &'a Partition,
    original_community: usize,
    refined: &'a mut Partition,
    resolution: f64,
    randomness: f64,

    /// group -> edge weight from this group to the rest of the original
    /// community. Excludes edges within the group and outside that community.
    weight_to_rest_of_community: HashMap<usize, f64>,
}

struct MergeCandidate {
    group: usize,
    gain: f64,
}

impl CommunityRefiner<'_> {
    /// Prepare the nodes, then try each allowed singleton once in random order.
    fn merge_nodes(&mut self, nodes: &[usize], rng: &mut Rng) {
        self.initialize_weights_to_rest(nodes);
        let mut node_allowed_to_merge = self.find_nodes_allowed_to_merge(nodes);
        rng.shuffle(&mut node_allowed_to_merge);
        for node in node_allowed_to_merge {
            let current_group = self.refined.community_of(node);

            if self.refined.community_size(current_group) > 1 {
                continue;
            }

            let node_weight_to_group = self.sum_node_weights_to_groups(node);
            let candidates = self.find_merge_candidates(node, &node_weight_to_group);
            let target_group = choose_group(&candidates, self.randomness, rng);

            if target_group != current_group {
                self.merge_node_into_group(node, target_group, node_weight_to_group[&target_group]);
            }
        }
    }

    /// Initialize `weight_to_rest_of_community` for each singleton group:
    /// sum its node's edge weights to other nodes in the same original community.
    /// Exclude self-loops and edges to nodes outside the original community.
    fn initialize_weights_to_rest(&mut self, nodes: &[usize]) {
        for &node in nodes {
            let mut weight_to_rest = 0.0;

            for &(neighbor, edge_weight) in self.graph.neighbors(node) {
                if neighbor == node
                    || self.original_partition.community_of(neighbor) != self.original_community
                {
                    continue;
                }
                weight_to_rest += edge_weight;
            }

            self.weight_to_rest_of_community
                .insert(self.refined.community_of(node), weight_to_rest);
        }
    }

    /// A singleton may attempt a merge if its gain toward the rest of its
    /// original community is nonnegative. Compute this list only once.
    fn find_nodes_allowed_to_merge(&self, nodes: &[usize]) -> Vec<usize> {
        let mut allowed_nodes = Vec::new();
        for &node in nodes {
            let gain = self.gain_to_rest_of_community(node);

            if gain >= 0.0 {
                allowed_nodes.push(node);
            }
        }
        allowed_nodes
    }

    /// group -> weight of edges from this node to that group.
    /// Only neighbors inside the original community count; self-loops do not.
    fn sum_node_weights_to_groups(&self, node: usize) -> HashMap<usize, f64> {
        let mut node_weight_to_group: HashMap<usize, f64> = HashMap::new();

        for &(neighbor, weight) in self.graph.neighbors(node) {
            if neighbor == node
                || self.original_partition.community_of(neighbor) != self.original_community
            {
                continue;
            }

            *node_weight_to_group
                .entry(self.refined.community_of(neighbor))
                .or_insert(0.0) += weight;
        }
        node_weight_to_group
    }

    /// Include staying alone, plus neighboring groups that pass both checks:
    /// the whole group connects sufficiently to the rest of the original
    /// community, and joining it gives this node a nonnegative gain.
    /// Sort by group id so seeded sampling does not depend on HashMap order.
    fn find_merge_candidates(
        &self,
        node: usize,
        node_weight_to_group: &HashMap<usize, f64>,
    ) -> Vec<MergeCandidate> {
        let mut candidates = vec![MergeCandidate {
            group: self.refined.community_of(node),
            gain: 0.0,
        }];

        for (&target_group, &weight_to_target) in node_weight_to_group {
            if self.gain_to_rest_of_community(target_group) < 0.0 {
                continue;
            }
            let gain = move_gain(
                weight_to_target,
                self.graph.degree(node),
                self.refined.community_degree(target_group),
                self.resolution,
                self.graph.total_weight(),
            );

            if gain >= 0.0 {
                candidates.push(MergeCandidate {
                    group: target_group,
                    gain,
                });
            }
        }
        candidates.sort_unstable_by_key(|candidate| candidate.group);
        candidates
    }

    /// Gain from joining a whole group to the rest of its original community.
    /// This is a check only: it does not actually perform that merge.
    fn gain_to_rest_of_community(&self, group: usize) -> f64 {
        let weight_to_rest = self.weight_to_rest_of_community[&group];
        let group_degree = self.refined.community_degree(group);
        let original_degree = self                                                  // total community degree
            .original_partition
            .community_degree(self.original_community);

        move_gain(
            weight_to_rest,
            group_degree,
            original_degree - group_degree,
            self.resolution,
            self.graph.total_weight(),
        )
    }

    /// Move a singleton and update its target's `weight_to_rest_of_community`.
    /// Edges between the node and target become internal to the merged group.
    fn merge_node_into_group(&mut self, node: usize, target_group: usize, weight_to_target: f64) {
        let source_group = self.refined.community_of(node);
        let source_weight_to_rest = self.weight_to_rest_of_community[&source_group];
        let target_weight_to_rest = self.weight_to_rest_of_community[&target_group];
        let merged_weight_to_rest =
            source_weight_to_rest + target_weight_to_rest - 2.0 * weight_to_target;

        self.refined.move_node(node, target_group);
        self.weight_to_rest_of_community
            .insert(target_group, merged_weight_to_rest);
        self.weight_to_rest_of_community.remove(&source_group);
    }
}

/// Pick a group with probability proportional to exp(gain / randomness).
/// The list is nonempty because staying alone is always a candidate.
/// Subtract the largest gain before exp to avoid overflow. Skip zero weights
/// and use the last positive-weight candidate as a rounding fallback.
fn choose_group(candidates: &[MergeCandidate], randomness: f64, rng: &mut Rng) -> usize {
    let max_gain = candidates
        .iter()
        .map(|candidate| candidate.gain)
        .fold(0.0, f64::max);
    let selection_weight: Vec<f64> = candidates
        .iter()
        .map(|candidate| ((candidate.gain - max_gain) / randomness).exp())
        .collect();

    let total_weight: f64 = selection_weight.iter().sum();
    let mut draw = rng.next_f64() * total_weight;
    let mut selected_group = candidates[0].group;
    for (candidate, weight) in candidates.iter().zip(selection_weight) {
        if weight == 0.0 {
            continue;
        }

        selected_group = candidate.group;
        if draw < weight {
            break;
        }
        draw -= weight;
    }
    selected_group
}
