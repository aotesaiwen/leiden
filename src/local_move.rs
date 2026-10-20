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
    let n = partition.num_nodes();
    let mut order: Vec<usize> = (0..n).collect();
    rng.shuffle(&mut order);
    let mut queue: VecDeque<usize> = order.into_iter().collect();
    let mut in_queue = vec![true; n];

    let mut flag: bool = false;
    while let Some(v) = queue.pop_front() {
        in_queue[v] = false;
        let map = partition.neighbor_community_weights(graph, v);
        let cur_c = partition.community_of(v);
        let edges_to_cur = *map.get(&cur_c).unwrap_or(&0.0);

        let mut best_c = partition.community_of(v);
        let mut best_gain = move_gain(
            edges_to_cur,
            partition.degree(v),
            partition.community_degree(cur_c) - partition.degree(v),
            resolution,
            graph.total_weight(),
        );

        let mut candidates: Vec<usize> = map.keys().copied().collect();
        candidates.sort_unstable();
        for c in candidates {
            if c == cur_c {
                continue;
            }
            let w = map[&c];
            let gain = move_gain(
                w,
                partition.degree(v),
                partition.community_degree(c),
                resolution,
                graph.total_weight(),
            );
            if gain > best_gain {
                best_gain = gain;
                best_c = c;
            }
        }
        if best_gain < 0.0 {
            best_c = partition.empty_community();
        }

        if best_c != cur_c {
            partition.move_node(v, best_c);
            flag = true;
            for &(u, _) in graph.neighbors(v) {
                if !in_queue[u] && partition.community_of(u) != best_c {
                    queue.push_back(u);
                    in_queue[u] = true;
                }
            }
        }
    }
    flag
}
