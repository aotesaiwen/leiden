//! Which community each node belongs to.

use std::{collections::HashMap, vec};

use crate::graph::Graph;

/// Community ids are plain numbers. After nodes move around some ids end up
/// empty; `renumber` tidies them back to 0, 1, 2, ...
#[derive(Clone, Debug)]
pub struct Partition {
    /// membership[v] = community of node v
    membership: Vec<usize>,
    /// Degree of each node (copied from the graph).
    degrees: Vec<f64>,
    /// Sum of node degrees in each community.
    community_degrees: Vec<f64>,
    /// Number of nodes in each community (0 = empty).
    community_sizes: Vec<usize>,
}

impl Partition {
    /// Every node alone in its own community (node v → community v).
    pub fn singleton(graph: &Graph) -> Partition {
        let num_nodes = graph.num_nodes();
        let mut membership: Vec<usize> = Vec::with_capacity(num_nodes);
        let mut degrees: Vec<f64> = Vec::with_capacity(num_nodes);

        for v in 0..num_nodes {
            membership.push(v);
            degrees.push(graph.degree(v));
        }

        let community_sizes: Vec<usize> = vec![1; num_nodes];

        Partition {
            membership,
            degrees: degrees.clone(),
            community_degrees: degrees,
            community_sizes,
        }
    }

    /// Build from a list where `membership[v]` is node v's community.
    /// Fill in the totals (`community_degrees`, `community_sizes`) yourself.
    pub fn from_membership(graph: &Graph, membership: Vec<usize>) -> Partition {
        let num_nodes = graph.num_nodes();
        let mut degrees: Vec<f64> = Vec::with_capacity(num_nodes);
        for v in 0..num_nodes {
            degrees.push(graph.degree(v));
        }

        // should reserve communities with 0 member... tightening should be done later?
        let capacity = membership.iter().max().map_or(0, |&m| m + 1);
        let mut community_degrees: Vec<f64> = vec![0.0; capacity];
        let mut community_sizes: Vec<usize> = vec![0; capacity];

        for (v, &c) in membership.iter().enumerate() {
            community_degrees[c] += degrees[v];
            community_sizes[c] += 1;
        }

        Partition {
            membership,
            degrees,
            community_degrees,
            community_sizes,
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
        self.community_sizes.iter().filter(|&&x| x > 0).count()
    }

    /// Move node `v` into community `to`. Remember to update the totals of
    /// both the old and the new community.
    pub fn move_node(&mut self, v: usize, to: usize) {
        let from = self.membership[v];
        if from == to {
            return;
        }

        self.membership[v] = to;
        self.community_degrees[from] -= self.degrees[v];
        self.community_degrees[to] += self.degrees[v];
        self.community_sizes[from] -= 1;
        self.community_sizes[to] += 1;
    }

    /// Id of a community with no nodes. Find one with size 0, or add a new
    /// one at the end (push 0.0 / 0 onto the totals).
    pub fn empty_community(&mut self) -> usize {
        if let Some(i) = self.community_sizes.iter().position(|&x| x == 0) {
            self.community_degrees[i] = 0.0;
            i
        } else {
            self.community_degrees.push(0.0);
            self.community_sizes.push(0);
            self.community_degrees.len() - 1
        }
    }

    /// Relabel communities as 0, 1, 2, ... with no gaps. Returns how many
    /// there are. Hint: a HashMap from old id to new id.
    pub fn renumber(&mut self) -> usize {
        let mut map: HashMap<usize, usize> = HashMap::new();

        for &c in self.membership.iter() {
            let next = map.len();
            map.entry(c).or_insert(next);
        }

        for v in 0..self.membership.len() {
            let from = self.membership[v];
            self.membership[v] = map[&from];
        }

        let mut community_degrees: Vec<f64> = vec![0.0; map.len()];
        let mut community_sizes: Vec<usize> = vec![0; map.len()];

        for (v, &c) in self.membership.iter().enumerate() {
            community_degrees[c] += self.degrees[v];
            community_sizes[c] += 1;
        }
        self.community_degrees = community_degrees;
        self.community_sizes = community_sizes;

        map.len()
    }

    /// The nodes in each community: `result[c]` = nodes of community c.
    /// Call `renumber` first so there are no gaps.
    pub fn members(&self) -> Vec<Vec<usize>> {
        let n = self.community_sizes.len();
        let mut members: Vec<Vec<usize>> = vec![Vec::new(); n];

        // for v in 0..self.membership.len() {
        //     members[self.membership[v]].push(v);
        // }
        for (v, &c) in self.membership.iter().enumerate() {
            members[c].push(v);
        }

        members
    }

    /// For node `v`: total edge weight from `v` into each neighbouring
    /// community. Returns community id → weight. Skip self-loops (u == v).
    ///
    /// Example: v has edges to a (weight 1) and b (weight 2), both in
    /// community 7 → returns {7: 3.0}.
    pub fn neighbor_community_weights(&self, graph: &Graph, v: usize) -> HashMap<usize, f64> {
        let mut result: HashMap<usize, f64> = HashMap::new();
        for &(u, w) in graph.neighbors(v) {
            if u == v {
                continue;
            }
            *result.entry(self.membership[u]).or_insert(0.0) += w;
        }
        result
    }
}
