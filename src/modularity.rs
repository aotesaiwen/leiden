//! Modularity: the score Leiden tries to maximise.
//!
//! When node `v` joins community `C`, modularity changes by (up to a
//! constant factor that doesn't matter for picking the best move):
//!
//! ```text
//! gain = edges_to_c − resolution × degree_v × degree_c / (2 × total_weight)
//! ```
//!
//! - `edges_to_c`: total edge weight between v and C. More edges = better.
//! - the second part is a penalty that grows as communities get bigger.
//!   Without it, putting everything in one community would always win.
//! - `degree_c`: sum of the degrees of C's nodes (not counting v).
//! - `resolution`: bigger = smaller communities. 1.0 is standard.

use crate::graph::Graph;
use crate::partition::Partition;

/// The gain formula above.
pub fn move_gain(
    edges_to_c: f64,
    degree_v: f64,
    degree_c: f64,
    resolution: f64,
    total_weight: f64,
) -> f64 {
    edges_to_c - resolution * degree_v * degree_c / (2f64 * total_weight)
}

/// Total modularity of a partition. The algorithm doesn't need this; it's
/// for tests and for checking your code (the score should never go down).
///
/// For each community C, add:
///     inside_C / m  −  resolution × (degree_C / 2m)²
/// inside_C = total weight of edges with both ends in C (count each once)
/// degree_C = sum of degrees of C's nodes
/// m        = graph.total_weight()
pub fn modularity(graph: &Graph, partition: &Partition, resolution: f64) -> f64 {
    todo!()
}
