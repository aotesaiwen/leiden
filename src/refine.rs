//! Step 2: refine. This is what makes Leiden better than Louvain.
//!
//! Step 1 can leave communities that are weakly held together, or even split
//! in two. Here each community is rebuilt from single nodes, and nodes only
//! merge when they are well connected.

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
    todo!()
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
    todo!()
}
