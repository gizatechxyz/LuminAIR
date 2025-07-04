use crate::{
    op::{
        prim::{CopyFromStwo, CopyToStwo, LuminairConstant},
        HasProcessTrace,
    },
    utils::compute_padded_range_from_srcs,
};
use itertools::Itertools;
use luminair_air::{
    components::{
        add::table::{AddColumn, AddTraceTable},
        exp2::table::{Exp2Column, Exp2TraceTable},
        less_than::table::{LessThanColumn, LessThanTraceTable},
        lookups::{
            exp2::{table::Exp2LookupTraceTable, Exp2Lookup},
            range_check::{table::RangeCheckLookupTraceTable, RangeCheckLayout, RangeCheckLookup},
            sin::{table::SinLookupTraceTable, SinLookup},
            Lookups,
        },
        max_reduce::table::{MaxReduceColumn, MaxReduceTraceTable},
        mul::table::{MulColumn, MulTraceTable},
        recip::table::{RecipColumn, RecipTraceTable},
        sin::table::{SinColumn, SinTraceTable},
        sqrt::table::{SqrtColumn, SqrtTraceTable},
        sum_reduce::table::{SumReduceColumn, SumReduceTraceTable},
    },
    pie::{
        ExecutionResources, InputInfo, LuminairPie, Metadata, NodeInfo, OpCounter, OutputInfo,
        TraceTable,
    },
    preprocessed::{LookupLayout, Range},
    settings::CircuitSettings,
    utils::calculate_log_size,
};
use luminair_utils::LuminairError;
use luminal::{op::*, prelude::*};
use numerair::Fixed;
use petgraph::{stable_graph::StableGraph, visit::EdgeRef, Direction};
use regex::Regex;
use rustc_hash::FxHashMap;

/// Trait defining the core functionality of a LuminAIR computation graph.
///
/// Provides methods to generate execution traces, retrieve outputs, and handle proof
/// generation and verification using Stwo.
pub trait LuminairGraph {
    /// Infers circuit settings using simulated representative inputs.
    fn gen_circuit_settings(&mut self) -> CircuitSettings;

    /// Generates an execution trace for the graph's computation.
    fn gen_trace(&mut self, settings: &mut CircuitSettings) -> Result<LuminairPie, LuminairError>;

    /// View the graph
    fn graph_viz(&self) -> String;
}

/// Implementation of `LuminairGraph` for the `luminal::Graph` struct.
impl LuminairGraph for Graph {
    /// Generates circuit settings, primarily by inferring lookup table requirements.
    ///
    /// Runs a pass over the graph to identify the range of values used
    /// by lookup-based operations (like `sin`).
    /// This information is crucial for constructing the preprocessed trace later.
    fn gen_circuit_settings(&mut self) -> CircuitSettings {
        // Track the number of views pointing to each tensor so we know when to clear
        if self.linearized_graph.is_none() {
            self.toposort();
        }
        let mut consumers = self.consumers_map.as_ref().unwrap().clone();
        let mut dim_stack = Vec::new();

        // Accumulate ranges per non-linear op
        let mut sin_ranges: Vec<Range> = Vec::new();
        let mut exp2_ranges: Vec<Range> = Vec::new();
        let mut less_than_ranges: Vec<Range> = Vec::new();

        for (node, src_ids) in self.linearized_graph.as_ref().unwrap() {
            if self.tensors.contains_key(&(*node, 0)) {
                continue;
            }

            let mut srcs =
                get_source_tensors(&self.no_delete, &mut self.tensors, src_ids, &consumers);

            // Substitute in the dyn dims
            for (_, st) in srcs.iter_mut() {
                st.resolve_global_dyn_dims_stack(&self.dyn_map, &mut dim_stack);
            }

            // Range
            let op = &*self.graph.node_weight(*node).unwrap();
            if <Box<dyn Operator> as HasProcessTrace<SinColumn, SinTraceTable, SinLookup>>::has_process_trace(op) {
                sin_ranges.push(compute_padded_range_from_srcs(&srcs));
            }
            if <Box<dyn Operator> as HasProcessTrace<Exp2Column, Exp2TraceTable, Exp2Lookup>>::has_process_trace(op) {
                exp2_ranges.push(compute_padded_range_from_srcs(&srcs));
            }
            if <Box<dyn Operator> as HasProcessTrace<
                LessThanColumn,
                LessThanTraceTable,
                RangeCheckLookup<1>,
            >>::has_process_trace(op)
            {
                less_than_ranges.push(compute_padded_range_from_srcs(&srcs));
            }

            // Execute
            let tensors = self.graph.node_weight_mut(*node).unwrap().process(srcs);
            for (i, tensor) in tensors.into_iter().enumerate() {
                self.tensors.insert((*node, i as u8), tensor);
            }

            // Bookkeep remaining consumers
            for (id, ind, _) in src_ids {
                *consumers.get_mut(&(*id, *ind)).unwrap() -= 1;
            }
        }

        self.reset();

        let sin_lookup = if !sin_ranges.is_empty() {
            let layout = LookupLayout::new(coalesce_ranges(sin_ranges));
            Some(SinLookup::new(&layout))
        } else {
            None
        };
        let exp2_lookup = if !exp2_ranges.is_empty() {
            let layout = LookupLayout::new(coalesce_ranges(exp2_ranges));
            Some(Exp2Lookup::new(&layout))
        } else {
            None
        };

        let range_check_lookup = if !less_than_ranges.is_empty() {
            // Coalesce ranges to get the overall min/max
            let coalesced_ranges = coalesce_ranges(less_than_ranges);
                        
            // Find the maximum absolute value to determine bit length needed
            let mut max_abs_value = 0i64;
            for range in &coalesced_ranges {
                let min_abs = range.0.0.abs();
                let max_abs = range.1.0.abs();
                max_abs_value = max_abs_value.max(min_abs).max(max_abs);
            }
                        
            // Calculate the maximum possible difference between any two values in the range
            // This is what we need to range check in the LessThan operation
            let max_diff = if !coalesced_ranges.is_empty() {
                let global_min = coalesced_ranges.iter().map(|r| r.0.0).min().unwrap();
                let global_max = coalesced_ranges.iter().map(|r| r.1.0).max().unwrap();
                (global_max - global_min).max(0)
            } else {
                0
            };
                        
            // Calculate required bit length: ceil(log2(max_diff + 1))
            let required_bits = if max_diff > 0 {
                (max_diff as u64 + 1).ilog2() + 1
            } else {
                1 // Minimum 1 bit
            };
                        
            // Ensure minimum bit length and cap at reasonable maximum
            let bit_length = required_bits.max(8).min(32);
            let log_size = bit_length;
                        
            Some(RangeCheckLookup::new(&RangeCheckLayout {
                ranges: [bit_length],
                log_size,
            }))
        } else {
            None
        };

        CircuitSettings {
            lookups: Lookups {
                sin: sin_lookup,
                exp2: exp2_lookup,
                range_check: range_check_lookup,
            },
        }
    }

    /// Generates the execution trace (witness) for the computation graph.
    ///
    /// Executes the graph operation by operation, collecting the inputs, outputs,
    /// and intermediate values for each supported AIR operation (e.g., add, mul, sin).
    /// It populates specific trace tables for each operation type and gathers
    /// metadata about the graph structure and execution flow.
    ///
    /// Returns a `LuminairPie` containing all the trace tables and execution resources.
    fn gen_trace(&mut self, settings: &mut CircuitSettings) -> Result<LuminairPie, LuminairError> {
        // Track the number of views pointing to each tensor so we know when to clear
        if self.linearized_graph.is_none() {
            self.toposort();
        }

        let mut consumers = self.consumers_map.as_ref().unwrap().clone();
        let mut dim_stack = Vec::new();

        // Initializes operator counter
        let mut op_counter = OpCounter::default();

        // Initializes table for each operator
        let mut add_table = AddTraceTable::new();
        let mut mul_table = MulTraceTable::new();
        let mut recip_table = RecipTraceTable::new();
        let mut sin_table = SinTraceTable::new();
        let mut sin_lookup_table = SinLookupTraceTable::new();
        let mut sum_reduce_table = SumReduceTraceTable::new();
        let mut max_reduce_table = MaxReduceTraceTable::new();
        let mut sqrt_table = SqrtTraceTable::new();
        let mut exp2_table = Exp2TraceTable::new();
        let mut exp2_lookup_table = Exp2LookupTraceTable::new();
        let mut less_than_table = LessThanTraceTable::new();
        let mut range_check_lookup_table = RangeCheckLookupTraceTable::new();

        for (node, src_ids) in self.linearized_graph.as_ref().unwrap() {
            if self.tensors.contains_key(&(*node, 0)) {
                continue;
            }

            let mut srcs =
                get_source_tensors(&self.no_delete, &mut self.tensors, src_ids, &consumers);

            // Substitute in the dyn dims
            for (_, st) in srcs.iter_mut() {
                st.resolve_global_dyn_dims_stack(&self.dyn_map, &mut dim_stack);
            }

            // Gather input source information
            let input_info: Vec<InputInfo> = src_ids
                .iter()
                .map(|(id, _, _)| {
                    let node_weight = self.node_weight(*id).unwrap();

                    let is_function = node_weight.as_any().is::<Function>();
                    let is_constant = node_weight.as_any().is::<LuminairConstant>()
                        || node_weight.as_any().is::<luminal::op::Constant>();
                    let is_copy_to = node_weight.as_any().is::<CopyToStwo>();

                    // Check if this is a CopyToStwo that wraps a Function node or a Constant
                    let is_copy_of_initializer = if is_copy_to {
                        self.get_sources(*id).iter().any(|(src_id, _, _)| {
                            let src_weight = self.node_weight(*src_id).unwrap();
                            src_weight.as_any().is::<Function>()
                                || src_weight.as_any().is::<LuminairConstant>()
                                || src_weight.as_any().is::<luminal::op::Constant>()
                        })
                    } else {
                        false
                    };

                    InputInfo {
                        is_initializer: is_function || is_constant || is_copy_of_initializer,
                        id: id.index() as u32,
                    }
                })
                .collect();

            // Determine output status
            let is_direct_output = self.to_retrieve.contains_key(&node);
            let is_output_via_copy = self
                .graph
                .edges_directed(*node, petgraph::Direction::Outgoing)
                .any(|e| {
                    let target = e.target();
                    self.to_retrieve.contains_key(&target)
                        && self
                            .node_weight(target)
                            .unwrap()
                            .as_any()
                            .is::<CopyFromStwo>()
                });

            // Calculate expansion-adjusted consumer count
            let base_consumers = *consumers.get(&(*node, 0)).unwrap_or(&0);
            let mut expansion_adjusted_consumers = 0u32;

            if base_consumers > 0 {
                // Iterate through each consumer edge to calculate expansion factors
                for edge in self
                    .graph
                    .edges_directed(*node, petgraph::Direction::Outgoing)
                {
                    if let Some((_, _, shape)) = edge.weight().as_data() {
                        // Calculate expansion factor for this consumer based on fake dimensions
                        let expansion_factor: u32 = (0..shape.len())
                            .map(|i| {
                                let dim_index = shape.indexes[i];
                                if shape.fake[dim_index] {
                                    // This dimension is fake (expanded), so count its size
                                    shape.dims[dim_index].to_usize().unwrap_or(1) as u32
                                } else {
                                    // This dimension is real, contributes factor of 1
                                    1
                                }
                            })
                            .product();

                        expansion_adjusted_consumers += expansion_factor;
                    }
                }
            } else {
                expansion_adjusted_consumers = base_consumers as u32;
            }

            let node_info = NodeInfo {
                inputs: input_info,
                output: OutputInfo {
                    is_final_output: is_direct_output || is_output_via_copy,
                },
                num_consumers: expansion_adjusted_consumers,
                id: node.index() as u32,
            };

            // Get operator and dispatch to appropriate process_trace handler
            let node_op = &mut *self.graph.node_weight_mut(*node).unwrap();

            let tensors =
                match () {
                    _
                        if <Box<dyn Operator> as HasProcessTrace<
                            AddColumn,
                            AddTraceTable,
                            (),
                        >>::has_process_trace(node_op) =>
                    {
                        op_counter.add += 1;
                        <Box<dyn Operator> as HasProcessTrace<AddColumn, AddTraceTable, ()>>::call_process_trace(
                        node_op, srcs, &mut add_table, &node_info, &mut ()
                    ).unwrap()
                    }
                    _
                        if <Box<dyn Operator> as HasProcessTrace<
                            MulColumn,
                            MulTraceTable,
                            (),
                        >>::has_process_trace(node_op) =>
                    {
                        op_counter.mul += 1;
                        <Box<dyn Operator> as HasProcessTrace<MulColumn, MulTraceTable, ()>>::call_process_trace(
                        node_op, srcs, &mut mul_table, &node_info, &mut ()
                    ).unwrap()
                    }
                    _ if <Box<dyn Operator> as HasProcessTrace<
                        RecipColumn,
                        RecipTraceTable,
                        (),
                    >>::has_process_trace(node_op) =>
                    {
                        op_counter.recip += 1;
                        <Box<dyn Operator> as HasProcessTrace<RecipColumn, RecipTraceTable, ()>>::call_process_trace(
                        node_op, srcs, &mut recip_table, &node_info, &mut ()
                    ).unwrap()
                    }
                    _ if <Box<dyn Operator> as HasProcessTrace<
                        SinColumn,
                        SinTraceTable,
                        SinLookup,
                    >>::has_process_trace(node_op) =>
                    {
                        op_counter.sin += 1;
                        match settings.lookups.sin.as_mut() {
                            Some(lookup) => <Box<dyn Operator> as HasProcessTrace<
                                SinColumn,
                                SinTraceTable,
                                SinLookup,
                            >>::call_process_trace(
                                node_op,
                                srcs,
                                &mut sin_table,
                                &node_info,
                                lookup,
                            )
                            .unwrap(),
                            None => unreachable!("Sin lookup table must be initialised"),
                        }
                    }
                    _ if <Box<dyn Operator> as HasProcessTrace<
                        SumReduceColumn,
                        SumReduceTraceTable,
                        (),
                    >>::has_process_trace(node_op) =>
                    {
                        op_counter.sum_reduce += 1;
                        <Box<dyn Operator> as HasProcessTrace<
                            SumReduceColumn,
                            SumReduceTraceTable,
                            (),
                        >>::call_process_trace(
                            node_op,
                            srcs,
                            &mut sum_reduce_table,
                            &node_info,
                            &mut (),
                        )
                        .unwrap()
                    }
                    _ if <Box<dyn Operator> as HasProcessTrace<
                        MaxReduceColumn,
                        MaxReduceTraceTable,
                        (),
                    >>::has_process_trace(node_op) =>
                    {
                        op_counter.max_reduce += 1;
                        <Box<dyn Operator> as HasProcessTrace<
                            MaxReduceColumn,
                            MaxReduceTraceTable,
                            (),
                        >>::call_process_trace(
                            node_op,
                            srcs,
                            &mut max_reduce_table,
                            &node_info,
                            &mut (),
                        )
                        .unwrap()
                    }
                    _ if <Box<dyn Operator> as HasProcessTrace<
                        SqrtColumn,
                        SqrtTraceTable,
                        (),
                    >>::has_process_trace(node_op) =>
                    {
                        op_counter.sqrt += 1;
                        <Box<dyn Operator> as HasProcessTrace<SqrtColumn, SqrtTraceTable, ()>>::call_process_trace(
                        node_op, srcs, &mut sqrt_table, &node_info, &mut ()
                    ).unwrap()
                    }
                    _ if <Box<dyn Operator> as HasProcessTrace<
                        Exp2Column,
                        Exp2TraceTable,
                        Exp2Lookup,
                    >>::has_process_trace(node_op) =>
                    {
                        op_counter.exp2 += 1;
                        match settings.lookups.exp2.as_mut() {
                            Some(lookup) => <Box<dyn Operator> as HasProcessTrace<
                                Exp2Column,
                                Exp2TraceTable,
                                Exp2Lookup,
                            >>::call_process_trace(
                                node_op,
                                srcs,
                                &mut exp2_table,
                                &node_info,
                                lookup,
                            )
                            .unwrap(),
                            None => unreachable!("Exp2 lookup table must be initialised"),
                        }
                    }
                    _ if <Box<dyn Operator> as HasProcessTrace<
                        LessThanColumn,
                        LessThanTraceTable,
                        RangeCheckLookup<1>,
                    >>::has_process_trace(node_op) =>
                    {
                        op_counter.less_than += 1;
                        match settings.lookups.range_check.as_mut() {
                            Some(lookup) => <Box<dyn Operator> as HasProcessTrace<
                                LessThanColumn,
                                LessThanTraceTable,
                                RangeCheckLookup<1>,
                            >>::call_process_trace(
                                node_op,
                                srcs,
                                &mut less_than_table,
                                &node_info,
                                lookup,
                            )
                            .unwrap(),
                            None => unreachable!("RangeCheck lookup table must be initialised"),
                        }
                    }

                    _ => node_op.process(srcs),
                };

            // Store output tensors
            for (i, tensor) in tensors.into_iter().enumerate() {
                self.tensors.insert((*node, i as u8), tensor);
            }

            // Update remaining consumers
            for (id, ind, _) in src_ids {
                *consumers.get_mut(&(*id, *ind)).unwrap() -= 1;
            }
        }

        self.reset();

        // Convert tables to traces - determine max log size while building
        let mut max_log_size = 0;
        let mut trace_tables = Vec::new();

        if !add_table.table.is_empty() {
            let log_size = calculate_log_size(add_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_add(add_table));
        }
        if !mul_table.table.is_empty() {
            let log_size = calculate_log_size(mul_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_mul(mul_table));
        }
        if !recip_table.table.is_empty() {
            let log_size = calculate_log_size(recip_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_recip(recip_table));
        }
        if !sin_table.table.is_empty() {
            let log_size = calculate_log_size(sin_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_sin(sin_table));

            if let Some(lookup) = settings.lookups.sin.as_ref() {
                lookup.add_multiplicities_to_table(&mut sin_lookup_table);
                max_log_size = max_log_size.max(lookup.layout.log_size);
                trace_tables.push(TraceTable::from_sin_lookup(sin_lookup_table))
            } // TODO (@raphaelDkhn): though error if LUT not present.
        }
        if !sum_reduce_table.table.is_empty() {
            let log_size = calculate_log_size(sum_reduce_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_sum_reduce(sum_reduce_table));
        }
        if !max_reduce_table.table.is_empty() {
            let log_size = calculate_log_size(max_reduce_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_max_reduce(max_reduce_table));
        }
        if !sqrt_table.table.is_empty() {
            let log_size = calculate_log_size(sqrt_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_sqrt(sqrt_table));
        }
        if !exp2_table.table.is_empty() {
            let log_size = calculate_log_size(exp2_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_exp2(exp2_table));

            if let Some(lookup) = settings.lookups.exp2.as_ref() {
                lookup.add_multiplicities_to_table(&mut exp2_lookup_table);
                max_log_size = max_log_size.max(lookup.layout.log_size);
                trace_tables.push(TraceTable::from_exp2_lookup(exp2_lookup_table))
            } // TODO (@raphaelDkhn): though error if LUT not present.
        }
        if !less_than_table.table.is_empty() {
            let log_size = calculate_log_size(less_than_table.table.len());
            max_log_size = max_log_size.max(log_size);
            trace_tables.push(TraceTable::from_less_than(less_than_table));

            if let Some(lookup) = settings.lookups.range_check.as_ref() {
                lookup.add_multiplicities_to_table(&mut range_check_lookup_table);
                max_log_size = max_log_size.max(lookup.layout.log_size);
                trace_tables.push(TraceTable::from_range_check_lookup(
                    range_check_lookup_table,
                ))
            } // TODO (@raphaelDkhn): though error if LUT not present.
        }

        Ok(LuminairPie {
            trace_tables,
            metadata: Metadata {
                execution_resources: ExecutionResources {
                    op_counter,
                    max_log_size,
                },
            },
        })
    }

    fn graph_viz(&self) -> String {
        let mut new_graph: StableGraph<String, u8> = StableGraph::default();
        let mut id_map = FxHashMap::default();
        for (id, node) in self.graph.node_indices().zip(self.graph.node_weights()) {
            id_map.insert(id, new_graph.add_node(format!("{node:?}")));
        }

        let mut schedule_edges = vec![];
        for node in self.graph.node_indices() {
            for edge in self
                .graph
                .edges_directed(node, Direction::Outgoing)
                .sorted_by_key(|e| {
                    if let Some(d) = e.weight().as_data() {
                        d.0
                    } else {
                        0
                    }
                })
            {
                let new_edge = new_graph.add_edge(
                    id_map[&edge.source()],
                    id_map[&edge.target()],
                    if let Some(d) = edge.weight().as_data() {
                        d.0
                    } else {
                        0
                    },
                );
                if edge.weight().is_schedule() {
                    schedule_edges.push(new_edge);
                }
            }
        }

        let mut graph_string =
            petgraph::dot::Dot::with_config(&new_graph, &[petgraph::dot::Config::EdgeIndexLabel])
                .to_string();
        let re = Regex::new(r#"label\s*=\s*"\d+""#).unwrap();
        for e in schedule_edges {
            graph_string =
                graph_string.replace(&format!("label = \"{}\"", e.index()), "color=\"green\"");
        }
        graph_string = re.replace_all(&graph_string, "").to_string();
        let mark_nodes: &[NodeIndex] = &[];
        for n in mark_nodes {
            graph_string = graph_string.replace(
                &format!("    {} [ label =", n.index()),
                &format!(
                    "    {} [ style=\"filled\" fillcolor=\"yellow\" label =",
                    n.index()
                ),
            );
        }

        graph_string.to_owned()
    }
}

/// Merges overlapping or adjacent ranges into a minimal set of disjoint ranges.
///
/// Used to consolidate the input ranges identified for lookup operations during
/// the `gen_circuit_settings` phase, optimizing the lookup table structure.
fn coalesce_ranges(mut ranges: Vec<Range>) -> Vec<Range> {
    if ranges.is_empty() {
        return Vec::new();
    }

    // Sort by lower bound
    ranges.sort_unstable_by_key(|r| r.0 .0);

    // Use the first element as the starting point
    let mut result = Vec::with_capacity(ranges.len());
    let mut current_range = ranges[0].clone();

    // Merge overlapping or adjacent ranges
    for range in ranges.into_iter().skip(1) {
        if range.0 .0 <= current_range.1 .0 + 1 {
            // Merge ranges if they overlap or are adjacent
            current_range.1 = Fixed(current_range.1 .0.max(range.1 .0));
        } else {
            // No overlap, push the current range and start a new one
            result.push(current_range);
            current_range = range;
        }
    }

    result.push(current_range);
    result
}
