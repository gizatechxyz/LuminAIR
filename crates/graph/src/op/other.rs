use itertools::Itertools;
use luminal::prelude::{petgraph::visit::EdgeRef, *};

use super::prim::{CopyFromStwo, CopyToStwo};

/// A Luminal `Compiler` pass that optimizes redundant copy operations.
///
/// After the `PrimitiveCompiler` inserts necessary `CopyToStwo` and `CopyFromStwo` nodes,
/// this pass identifies and removes consecutive pairs of these operations
/// (i.e., `CopyToStwo -> CopyFromStwo` or `CopyFromStwo -> CopyToStwo`)
/// where the intermediate node is not essential.
/// This simplifies the graph by eliminating unnecessary data format conversions.
#[derive(Debug, Default)]
pub struct CopyCompiler();

impl Compiler for CopyCompiler {
    type Output = ();

    /// Executes the copy optimization pass on the graph.
    /// Modifies the graph in-place by removing redundant copy node pairs.
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
