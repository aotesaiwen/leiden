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
//! A pass stops when step 1 leaves every node of the current graph alone
//! in its own community. A node at that level may represent many original nodes.
//!
//! Further passes start again on the original graph, using the previous
//! pass's partition. `LeidenConfig::iterations` controls when to stop.

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
    /// How random the refine step is. Must be positive and finite.
    /// 0.01 is a good default.
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
///
/// Assumes nonnegative, finite edge weights and a positive total weight
/// for nonempty graphs. `resolution` must be nonnegative and finite.
/// `Some(0)` iterations returns the singleton partition.
pub fn leiden(graph: &Graph, config: &LeidenConfig) -> Partition {
    let mut rng = Rng::new(config.seed);
    let mut partition = Partition::singleton(graph);

    match config.iterations {
        Some(iterations) => {
            for _ in 0..iterations {
                let (next, _) = leiden_pass(graph, partition, config, &mut rng);
                partition = next;
            }
        }
        None => loop {
            let (next, changed) = leiden_pass(graph, partition, config, &mut rng);
            partition = next;
            if !changed {
                break;
            }
        },
    }

    partition.renumber();
    partition
}

/// One full run of the algorithm, starting from `start`.
/// Returns the new partition and whether any node changed community.
fn leiden_pass(
    graph: &Graph,
    start: Partition,
    config: &LeidenConfig,
    rng: &mut Rng,
) -> (Partition, bool) {
    let mut current_graph = graph.clone();
    let mut partition = start;
    // node_of[i] is the current graph node representing original node i.
    let mut node_of: Vec<usize> = (0..graph.num_nodes()).collect();
    let mut changed = false;

    loop {
        let moved = move_nodes_fast(
            &current_graph,
            &mut partition,
            config.resolution,
            rng,
        );
        changed |= moved;

        if partition.num_communities() == current_graph.num_nodes() {
            break;
        }

        let mut refined = refine_partition(
            &current_graph,
            &partition,
            config.resolution,
            config.randomness,
            rng,
        );
        refined.renumber();

        let smaller_graph = aggregate_graph(&current_graph, &refined);
        let next_partition = lift_partition(&refined, &partition, &smaller_graph);

        for node in &mut node_of {
            *node = refined.community_of(*node);
        }

        current_graph = smaller_graph;
        partition = next_partition;
    }

    // Expand the final communities back to the original graph.
    let membership = node_of
        .into_iter()
        .map(|node| partition.community_of(node))
        .collect();
    (Partition::from_membership(graph, membership), changed)
}
