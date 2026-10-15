use leiden::{leiden, modularity, Graph, LeidenConfig};

fn main() {
    // Two groups of 4 fully connected nodes, joined by the edge 3–4.
    let mut edges = Vec::new();
    for base in [0, 4] {
        for i in 0..4 {
            for j in i + 1..4 {
                edges.push((base + i, base + j, 1.0));
            }
        }
    }
    edges.push((3, 4, 1.0));
    let graph = Graph::from_edges(8, &edges);

    let config = LeidenConfig::default();
    let partition = leiden(&graph, &config);

    println!("membership: {:?}", partition.membership());
    println!("modularity: {:.4}", modularity(&graph, &partition, config.resolution));
}
