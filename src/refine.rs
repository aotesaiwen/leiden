//! Step 2: refine. This is what makes Leiden better than Louvain.
//!
//! Step 1 can leave communities that are weakly held together, or even split
//! in two. Here each community is rebuilt from single nodes, and nodes only
//! merge when they are well connected.

use std::collections::HashMap;

use crate::graph::Graph;
use crate::modularity::move_gain;
use crate::partition::Partition;
use crate::rng::Rng;

/// 1. Start with every node alone: `Partition::singleton`.
/// 2. For each community from step 1, call `merge_nodes_subset` on its nodes.
///
/// You get more, smaller communities, each inside one community from step 1.
pub(crate) fn refine_partition(
    graph: &Graph,
    partition: &Partition,
    resolution: f64,
    randomness: f64,
    rng: &mut Rng,
) -> Partition {
    let mut refined = Partition::singleton(graph);
    let members = partition.members();
    for (community_id, nodes) in members.iter().enumerate() {
        if nodes.is_empty() {
            continue;
        }

        merge_nodes_subset(
            graph,
            partition,
            &mut refined,
            community_id,
            nodes,
            resolution,
            randomness,
            rng,
        );
    }

    refined
}

/// Merge nodes inside one step-1 community S.
/// `s` is its id, `members` its nodes.
///
/// Only count edges inside S: skip neighbour `u` if
/// `partition.community_of(u) != s`.
///
/// "X is well connected to S" means:
///     edge weight from X to the rest of S
///         >= resolution × degree_X × (degree_S − degree_X) / (2 × total_weight)
///
/// 1. Find the nodes of S that are well connected to S.
/// 2. Go through them in random order. Skip a node if it's no longer alone
///    (someone already joined it).
/// 3. For node v, look at the refined communities next to it inside S.
///    Keep the ones that are well connected to S and have gain >= 0
///    (`move_gain`). Staying alone (gain 0) is also an option.
/// 4. Pick one at random, higher gain = more likely:
///        chance of C ∝ exp(gain_C / randomness)
///    Subtract the largest gain before `exp`, or the numbers overflow.
/// 5. Move v there.
///
/// Tip: for step 3 you need each refined community's "edge weight to the
/// rest of S". Store it in a HashMap and update it when v joins C:
///     new_C = old_C + old_v − 2 × (edge weight between v and C)
#[allow(clippy::too_many_arguments)]
fn merge_nodes_subset(
    graph: &Graph,
    partition: &Partition,
    refined: &mut Partition,
    s: usize,
    members: &[usize],
    resolution: f64,
    randomness: f64,
    rng: &mut Rng,
) {
    if members.len() <= 1 {
        // at least 2 nodes in this community
        return;
    }
    let m = graph.total_weight();
    let degree_s = partition.community_degree(s);
    let mut eligible: Vec<usize> = Vec::new(); // id of nodes allowed to attempt a merge during the refinement
    let mut boundry: HashMap<usize, f64> = HashMap::new(); // total weight of edges connecting that group to other nodes in its original community

    for &v in members {
        let mut weight = 0.0;
        for &(u, w) in graph.neighbors(v) {
            if u == v || partition.community_of(u) != s {
                continue;
            }
            weight += w;
        }
        boundry.insert(refined.community_of(v), weight);

        let degree_v = graph.degree(v);
        let gain = move_gain(weight, degree_v, degree_s - degree_v, resolution, m);
        if gain >= 0.0 {
            eligible.push(v);
        }
    }

    rng.shuffle(&mut eligible);
    for v in eligible {
        let cur_c = refined.community_of(v);
        if refined.community_size(cur_c) > 1 {
            continue;
        }

        let mut weights: HashMap<usize, f64> = HashMap::new(); // total edge weight from the current node v to each neighboring refined group within the original community
        for &(u, w) in graph.neighbors(v) {
            if u == v || partition.community_of(u) != s {
                continue;
            }
            let target = refined.community_of(u);
            *weights.entry(target).or_insert(0.0) += w;
        }

        let mut candidates: Vec<(usize, f64)> = vec![(cur_c, 0.0)]; // candidates of groups node v can attempt to join, along with its gain
        // should filter out groups unwilling to have a new node joined
        for (&target, &w) in &weights {
            let target_degree = refined.community_degree(target);
            let connection_gain = move_gain(
                boundry[&target],
                target_degree,
                degree_s - target_degree,
                resolution,
                m,
            );
            if connection_gain < 0.0 {
                continue;
            }
            let gain = move_gain(w, graph.degree(v), target_degree, resolution, m);
            if gain >= 0.0 {
                candidates.push((target, gain));
            }
        }

        candidates.sort_unstable_by_key(|&(target, _)| target);
        let max_gain = candidates.iter().map(|&(_, gain)| gain).fold(0.0, f64::max);
        let selection_weights: Vec<f64> = candidates
            .iter()
            .map(|&(_, gain)| ((gain - max_gain) / randomness).exp())
            .collect();

        let total: f64 = selection_weights.iter().sum();
        let mut draw = rng.next_f64() * total;
        let mut selected = cur_c;

        for i in 0..candidates.len() {
            let weight = selection_weights[i];

            if weight == 0.0 {
                continue;
            }

            selected = candidates[i].0;
            if draw < weight {
                break;
            }

            draw -= weight;
        }

        if selected != cur_c {
            // The edges between v and the target become internal to the
            // merged group, so remove them from both old boundary totals.
            let merged_boundary = boundry[&cur_c] + boundry[&selected] - 2.0 * weights[&selected];

            refined.move_node(v, selected);
            boundry.insert(selected, merged_boundary);
            boundry.remove(&cur_c);
        }
    }
}
