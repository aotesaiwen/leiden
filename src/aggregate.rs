//! Step 3: shrink the graph.

use crate::graph::Graph;
use crate::partition::Partition;

/// Build a smaller graph with one node per refined community.
/// (`refined` must already be renumbered, so ids are 0, 1, 2, ...)
///
/// - Edge u–v becomes an edge between their communities. If both ends are
///   in the same community, it becomes a self-loop.
/// - Add up the weights of edges that land on the same pair (a HashMap
///   keyed by `(a, b)` with a <= b works).
///
/// Careful: `neighbors` lists a normal edge from both ends, but a self-loop
/// only once. To count every edge once, only take edge (v, u) when v <= u.
///
/// Check: the new graph's `total_weight()` should equal the old one's.
pub(crate) fn aggregate_graph(graph: &Graph, refined: &Partition) -> Graph {
    todo!()
}

/// Carry the step-1 communities over to the smaller graph.
///
/// Each new node (= refined community) joins the step-1 community its
/// original nodes were in. For every old node v:
///     new_membership[refined.community_of(v)] = partition.community_of(v)
/// then `Partition::from_membership(smaller_graph, new_membership)`.
pub(crate) fn lift_partition(
    refined: &Partition,
    partition: &Partition,
    smaller_graph: &Graph,
) -> Partition {
    todo!()
}
