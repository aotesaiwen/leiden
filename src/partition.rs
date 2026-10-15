//! Which community each node belongs to.

use std::{collections::HashMap, vec};

use crate::graph::Graph;

/// Community ids are plain numbers. After nodes move around some ids end up
/// empty; `renumber` tidies them back to 0, 1, 2, ...
#[derive(Clone, Debug)]
pub struct Partition {
    /// membership[v] = community of node v, size fixed as num_nodes?
    membership: Vec<usize>,
    /// Degree of each node (copied from the graph). this is fixed
    degrees: Vec<f64>,
    /// Sum of node degrees in each community.
    community_degrees: Vec<f64>,
    /// Number of nodes in each community (0 = empty). empty would ge tidied somehow?
    community_sizes: Vec<usize>,
}

impl Partition {
    /// Every node alone in its own community (node v → community v).
    pub fn singleton(graph: &Graph) -> Partition {
        let num_nodes = graph.num_nodes();
        // let mut degrees = Vec::with_capacity(num_nodes);
        // for i in 0..num_nodes {
        //     degrees.push(graph.degree(i));
        // }
        let degrees: Vec<f64> = (0..num_nodes).map(|v| graph.degree(v)).collect();
        Partition {
            membership: (0..num_nodes).collect(),
            degrees: degrees.clone(),
            community_degrees: degrees,
            community_sizes: vec![1; num_nodes],
        }
    }

    /// Build from a list where `membership[v]` is node v's community.
    /// Fill in the totals (`community_degrees`, `community_sizes`) yourself.
    pub fn from_membership(graph: &Graph, membership: Vec<usize>) -> Partition {
        let num_nodes = graph.num_nodes();
        let degrees: Vec<f64> = (0..num_nodes).map(|v| graph.degree(v)).collect();

        // let k = membership.iter().max().map_or(0, |&m| m + 1);
        // k should be the max index of community + 1, that is the community count
        let k = membership.iter().max().expect("membership is empty") + 1;
        let mut community_degrees: Vec<f64> = vec![0.0; k];
        let mut community_sizes: Vec<usize> = vec![0; k];
        for v in 0..membership.len() {
            let c = membership[v];
            community_degrees[c] += degrees[v];
            community_sizes[c] += 1;
        }

        Partition {
            membership,
            degrees: degrees,
            community_degrees: community_degrees,
            community_sizes: community_sizes,
        }
    }

    pub fn num_nodes(&self) -> usize {
        self.membership.len()
    }

    pub fn membership(&self) -> &[usize] {
        &self.membership
    }

    pub fn community_of(&self, v: usize) -> usize {
        self.membership[v]
    }

    pub fn degree(&self, v: usize) -> f64 {
        self.degrees[v]
    }

    pub fn community_degree(&self, c: usize) -> f64 {
        self.community_degrees[c]
    }

    pub fn community_size(&self, c: usize) -> usize {
        self.community_sizes[c]
    }

    /// How many communities have at least one node.
    pub fn num_communities(&self) -> usize {
        self.community_sizes.iter().filter(|&&s| s > 0).count()
    }

    /// Move node `v` into community `to`. Remember to update the totals of
    /// both the old and the new community.
    pub fn move_node(&mut self, v: usize, to: usize) {
        if self.membership[v] == to {
            return;
        }

        let old_community = self.membership[v];
        self.membership[v] = to;
        self.community_degrees[old_community] -= self.degrees[v];
        self.community_degrees[to] += self.degrees[v];
        self.community_sizes[old_community] -= 1;
        self.community_sizes[to] += 1;
    }

    /// Id of a community with no nodes. Find one with size 0, or add a new
    /// one at the end (push 0.0 / 0 onto the totals).
    pub fn empty_community(&mut self) -> usize {
        if let Some(c) = self.community_sizes.iter().position(|&s| s == 0) {
            return c;
        }

        self.community_sizes.push(0);
        self.community_degrees.push(0.0);
        self.community_sizes.len() - 1
    }

    /// Relabel communities as 0, 1, 2, ... with no gaps. Returns how many
    /// there are. Hint: a HashMap from old id to new id.
    pub fn renumber(&mut self) -> usize {
        let mut map: HashMap<usize, usize> = HashMap::new();
        for v in 0..self.membership.len() {
            let old = self.membership[v];
            let next = map.len();
            let new = *map.entry(old).or_insert(next);
            self.membership[v] = new;
        }

        let k = map.len();
        let mut community_degrees: Vec<f64> = vec![0.0; k];
        let mut community_sizes: Vec<usize> = vec![0; k];
        for v in 0..self.membership.len() {
            let c = self.membership[v];
            community_degrees[c] += self.degrees[v];
            community_sizes[c] += 1;
        }
        self.community_degrees = community_degrees;
        self.community_sizes = community_sizes;
        k
    }

    /// The nodes in each community: `result[c]` = nodes of community c.
    /// Call `renumber` first so there are no gaps.
    pub fn members(&self) -> Vec<Vec<usize>> {
        todo!()
    }

    /// For node `v`: total edge weight from `v` into each neighbouring
    /// community. Returns community id → weight. Skip self-loops (u == v).
    ///
    /// Example: v has edges to a (weight 1) and b (weight 2), both in
    /// community 7 → returns {7: 3.0}.
    pub fn neighbor_community_weights(&self, graph: &Graph, v: usize) -> HashMap<usize, f64> {
        let mut map: HashMap<usize, f64> = HashMap::new();

        for &(u, w) in graph.neighbors(v) {
            if u == v {
                continue;
            }
            *map.entry(self.community_of(u)).or_insert(0.0) += w;
        }

        map
    }
}
