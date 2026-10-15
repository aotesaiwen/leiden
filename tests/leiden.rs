use std::collections::{HashSet, VecDeque};

use leiden::{leiden, modularity, Graph, LeidenConfig, Partition};

/// `count` fully connected groups of `size` nodes, joined in a ring by
/// single edges.
fn ring_of_cliques(count: usize, size: usize) -> Graph {
    let mut edges = Vec::new();
    for c in 0..count {
        let base = c * size;
        for i in 0..size {
            for j in i + 1..size {
                edges.push((base + i, base + j, 1.0));
            }
        }
        let next = ((c + 1) % count) * size;
        edges.push((base + size - 1, next, 1.0));
    }
    Graph::from_edges(count * size, &edges)
}

fn two_cliques() -> Graph {
    let mut edges = Vec::new();
    for base in [0, 4] {
        for i in 0..4 {
            for j in i + 1..4 {
                edges.push((base + i, base + j, 1.0));
            }
        }
    }
    edges.push((3, 4, 1.0));
    Graph::from_edges(8, &edges)
}

fn num_communities(p: &Partition) -> usize {
    p.membership().iter().collect::<HashSet<_>>().len()
}

/// Leiden promises every community is connected (no separate pieces).
fn assert_communities_connected(g: &Graph, p: &Partition) {
    let n = g.num_nodes();
    let mut seen = vec![false; n];
    let mut roots = HashSet::new();
    for start in 0..n {
        if seen[start] {
            continue;
        }
        let c = p.community_of(start);
        assert!(roots.insert(c), "community {c} is disconnected");
        let mut queue = VecDeque::from([start]);
        seen[start] = true;
        while let Some(v) = queue.pop_front() {
            for &(u, _) in g.neighbors(v) {
                if !seen[u] && p.community_of(u) == c {
                    seen[u] = true;
                    queue.push_back(u);
                }
            }
        }
    }
}

#[test]
fn splits_two_cliques() {
    let g = two_cliques();
    let p = leiden(&g, &LeidenConfig::default());
    let m = p.membership();
    assert!(m[0..4].iter().all(|&c| c == m[0]));
    assert!(m[4..8].iter().all(|&c| c == m[4]));
    assert_ne!(m[0], m[4]);
}

#[test]
fn ring_of_cliques_finds_each_clique() {
    let g = ring_of_cliques(6, 5);
    let p = leiden(&g, &LeidenConfig::default());
    assert_eq!(num_communities(&p), 6);
    assert_communities_connected(&g, &p);
}

#[test]
fn improves_on_singletons() {
    let g = ring_of_cliques(6, 5);
    let p = leiden(&g, &LeidenConfig::default());
    assert!(modularity(&g, &p, 1.0) > modularity(&g, &Partition::singleton(&g), 1.0));
}
