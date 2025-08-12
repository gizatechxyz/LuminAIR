use itertools::Itertools;
use luminal::prelude::{petgraph::visit::EdgeRef, *};

use super::prim::{CopyFromStwo, CopyToStwo};

/// Compiler that optimizes copy operations in the computational graph
/// 
/// Removes unnecessary copy operation chains and simplifies the graph structure
/// by eliminating redundant CopyToStwo -> CopyFromStwo sequences
#[derive(Debug, Default)]
pub struct CopyCompiler();

impl Compiler for CopyCompiler {
    type Output = ();

    /// Compiles the graph by optimizing copy operations
    /// 
    /// This process:
    /// 1. Identifies chains of copy operations (CopyToStwo -> CopyFromStwo)
    /// 2. Removes unnecessary copy nodes when they don't serve a purpose
    /// 3. Simplifies the graph structure while maintaining functionality
    fn compile<To: ToIdsMut>(&self, graph: &mut Graph, mut ids: To) {
        for (first, second) in graph
            .edge_indices()
            .filter_map(|e| graph.edge_endpoints(e))
            .filter(|(a, b)| {
                (graph.node_weight(*a).unwrap().as_any().is::<CopyToStwo>()
                    && graph.node_weight(*b).unwrap().as_any().is::<CopyFromStwo>())
                    || (graph.node_weight(*a).unwrap().as_any().is::<CopyFromStwo>()
                        && graph.node_weight(*b).unwrap().as_any().is::<CopyToStwo>())
            })
            .unique_by(|n| n.0)
            .unique_by(|n| n.1)
            .collect::<Vec<_>>()
        {
            if graph
                .edges_directed(first, petgraph::Direction::Outgoing)
                .filter(|e| graph.contains_node(e.target()))
                .filter(|e| {
                    !graph
                        .node_weight(e.target())
                        .unwrap()
                        .as_any()
                        .is::<CopyFromStwo>()
                        && !graph
                            .node_weight(e.target())
                            .unwrap()
                            .as_any()
                            .is::<CopyToStwo>()
                })
                .count()
                > 0
                || graph.no_delete.contains(&first)
            {
                continue;
            }
            let source = graph.get_sources(first)[0];
            move_outgoing_edge(second, source.0, graph);
            remap(second, source.0, &mut ids, graph);
            graph.remove_node(second);
            for dest in graph
                .get_dests(first)
                .iter()
                .map(|(i, _)| *i)
                .collect::<Vec<_>>()
            {
                move_outgoing_edge(dest, source.0, graph);
                remap(dest, source.0, &mut ids, graph);
                graph.remove_node(dest);
            }
            graph.remove_node(first);
        }
    }
}
