//! Step 1: move single nodes to better communities.

use std::collections::VecDeque; // a queue: push_back() to add, pop_front() to take

use crate::graph::Graph;
use crate::modularity::move_gain;
use crate::partition::Partition;
use crate::rng::Rng;

/// 1. Put all nodes in a queue, in random order (`rng.shuffle`).
///    Keep a `Vec<bool>` saying which nodes are in the queue.
/// 2. Take node `v` from the front.
/// 3. Work out the gain of `v` joining each neighbouring community
///    (`partition.neighbor_community_weights` + `move_gain`).
///    For v's *current* community, subtract v's own degree from the
///    community degree first.
/// 4. Pick the best. Moving to an empty community (gain 0) is also an option.
///    If it beats staying put, move `v` there.
/// 5. If `v` moved, add its neighbours to the back of the queue, unless
///    they're already queued or already in v's new community.
/// 6. Repeat from 2 until the queue is empty.
///
/// Returns `true` if any node moved.
pub(crate) fn move_nodes_fast(
    graph: &Graph,
    partition: &mut Partition,
    resolution: f64,
    rng: &mut Rng,
) -> bool {
    todo!()
}
