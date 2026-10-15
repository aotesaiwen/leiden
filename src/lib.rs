//! Leiden community detection, written from scratch.
//!
//! Goal: split a graph's nodes into groups ("communities") with many edges
//! inside each group and few edges between groups.
//!
//! Paper: Traag, Waltman & van Eck (2019), "From Louvain to Leiden".
//!
//! ## How it works
//!
//! Repeat:
//! 1. **Move nodes** (`local_move.rs`): each node joins the neighbouring
//!    community that improves the score the most.
//! 2. **Refine** (`refine.rs`): split each community into smaller pieces
//!    that are well connected inside. Louvain doesn't have this step, which
//!    is why it can produce broken communities and Leiden can't.
//! 3. **Aggregate** (`aggregate.rs`): turn each piece into one node, giving
//!    a smaller graph. Go back to step 1 on that graph.
//!
//! Stop when step 1 leaves every node alone in its own community.
//!
//! ## What's done and what's yours
//! - `graph.rs` and `rng.rs` are finished.
//! - Every `todo!()` is yours. `todo!()` compiles but crashes if it runs,
//!   so you can fill in one function at a time.
//! - Suggested order: modularity.rs → partition.rs → local_move.rs →
//!   refine.rs → aggregate.rs → `leiden_pass` / `leiden` below.

// Hides "unused" warnings while functions are still todo!().
// Delete this line when you're done.
#![allow(unused_variables, dead_code, unused_imports, unused_mut)]

mod aggregate;
pub mod graph;
mod local_move;
pub mod modularity;
pub mod partition;
mod refine;
pub mod rng;

pub use graph::Graph;
pub use modularity::modularity;
pub use partition::Partition;

use aggregate::{aggregate_graph, lift_partition};
use local_move::move_nodes_fast;
use refine::refine_partition;
use rng::Rng;

/// Settings for a run.
#[derive(Clone, Debug)]
pub struct LeidenConfig {
    /// Bigger = smaller communities. 1.0 is standard modularity.
    pub resolution: f64,
    /// How random the refine step is. 0.01 is a good default.
    pub randomness: f64,
    /// How many times to run the whole algorithm (each run starts from the
    /// last result). `None` = keep going until nothing changes.
    pub iterations: Option<usize>,
    /// Same seed = same result every time.
    pub seed: u64,
}

impl Default for LeidenConfig {
    fn default() -> Self {
        Self {
            resolution: 1.0,
            randomness: 0.01,
            iterations: Some(2),
            seed: 42,
        }
    }
}

/// Run Leiden. Returns which community each node is in.
pub fn leiden(graph: &Graph, config: &LeidenConfig) -> Partition {
    // 1. Make an Rng from config.seed.
    // 2. Start with every node alone: Partition::singleton(graph).
    // 3. Call leiden_pass `config.iterations` times, passing the result
    //    of one call into the next.
    // 4. renumber() the result and return it.
    todo!()
}

/// One full run of the algorithm, starting from `start`.
/// Returns the new partition and whether any node changed community.
fn leiden_pass(
    graph: &Graph,
    start: Partition,
    config: &LeidenConfig,
    rng: &mut Rng,
) -> (Partition, bool) {
    // The graph shrinks every round, so keep track of where each original
    // node ended up:
    //     node_of[original node] = node in the current (smaller) graph
    // It starts as 0, 1, 2, ... n-1.
    //
    // Loop:
    //   a. move_nodes_fast(...)              (step 1)
    //   b. if every node is alone in its community → stop
    //   c. refined = refine_partition(...)   (step 2), then refined.renumber()
    //   d. smaller = aggregate_graph(...)    (step 3)
    //   e. partition = lift_partition(...)   (carry communities to `smaller`)
    //   f. update node_of: node_of[i] = refined.community_of(node_of[i])
    //   g. current_graph = smaller
    //
    // Finally: original node i is in community partition.community_of(node_of[i]).
    // Build a Partition from that with Partition::from_membership.
    todo!()
}
