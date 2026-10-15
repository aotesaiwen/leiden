//! An undirected graph with weighted edges. Finished, nothing to do here.

/// Nodes are numbered 0, 1, 2, ... n-1.
#[derive(Clone, Debug)]
pub struct Graph {
    /// adjacency[v] = list of (neighbour, edge weight).
    /// A normal edge u–v is stored in both adjacency[u] and adjacency[v].
    /// A self-loop v–v is stored once, in adjacency[v].
    adjacency: Vec<Vec<(usize, f64)>>,
    /// Total weight of edges touching each node. A self-loop counts twice.
    degrees: Vec<f64>,
    /// Total weight of all edges, each counted once.
    total_weight: f64,
}

impl Graph {
    /// Build a graph with `n` nodes from a list of edges `(u, v, weight)`.
    /// List each edge once.
    pub fn from_edges(n: usize, edges: &[(usize, usize, f64)]) -> Graph {
        let mut adjacency = vec![Vec::new(); n];
        let mut degrees = vec![0.0; n];
        let mut total_weight = 0.0;

        for &(u, v, w) in edges {
            adjacency[u].push((v, w));
            if u != v {
                adjacency[v].push((u, w));
            }
            degrees[u] += w;
            degrees[v] += w; // for a self-loop (u == v) this adds w twice, on purpose
            total_weight += w;
        }

        Graph {
            adjacency,
            degrees,
            total_weight,
        }
    }

    pub fn num_nodes(&self) -> usize {
        self.adjacency.len()
    }

    /// `(neighbour, weight)` pairs of node `v`. Use it like:
    /// `for &(u, w) in graph.neighbors(v) { ... }`
    pub fn neighbors(&self, v: usize) -> &[(usize, f64)] {
        &self.adjacency[v]
    }

    pub fn degree(&self, v: usize) -> f64 {
        self.degrees[v]
    }

    pub fn total_weight(&self) -> f64 {
        self.total_weight
    }
}
