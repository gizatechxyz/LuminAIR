use crate::{
    op::{
        prim::{CopyFromStwo, CopyToStwo, LuminairConstant},
        HasProcessTrace,
    },
    utils::compute_padded_range_from_srcs,
};
use luminair_air::{
    components::{
        add::table::{AddColumn, AddTraceTable},
        lookups::{
            sin::{table::SinLookupTraceTable, SinLookup},
            Lookups,
        },
        max_reduce::table::{MaxReduceColumn, MaxReduceTraceTable},
        mul::table::{MulColumn, MulTraceTable},
        recip::table::{RecipColumn, RecipTraceTable},
        sin::table::{SinColumn, SinTraceTable},
        sqrt::table::{SqrtColumn, SqrtTraceTable},
        sum_reduce::table::{SumReduceColumn, SumReduceTraceTable},
        LuminairComponents, LuminairInteractionElements,
    },
    pie::{
        ExecutionResources, InputInfo, LuminairPie, NodeInfo, OpCounter, OutputInfo, TraceTable,
    },
    preprocessed::{lookups_to_preprocessed_column, LookupLayout, PreProcessedTrace, Range},
    settings::CircuitSettings,
    utils::{calculate_log_size, log_sum_valid},
    LuminairProof,
};
use luminair_utils::LuminairError;
use luminal::{
    op::*,
    prelude::{petgraph::visit::EdgeRef, *},
};
use numerair::Fixed;
use stwo_prover::{
    constraint_framework::{INTERACTION_TRACE_IDX, ORIGINAL_TRACE_IDX, PREPROCESSED_TRACE_IDX},
    core::{
        channel::Blake2sChannel,
        pcs::{CommitmentSchemeVerifier, PcsConfig},
        prover::verify,
        vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
    },
};

/// Trait defining the core functionality of a LuminAIR computation graph.
///
/// Provides methods to generate execution traces, retrieve outputs, and handle proof
/// generation and verification using Stwo.
pub trait LuminairGraph {
    /// Infers circuit settings using simulated representative inputs.
    fn gen_circuit_settings(&mut self) -> CircuitSettings;

    /// Generates an execution trace for the graph's computation.
    fn gen_trace(&mut self, settings: &mut CircuitSettings) -> Result<LuminairPie, LuminairError>;

    /// Verifies a proof to ensure integrity of graph's computation.
    fn verify(
        &self,
        proof: LuminairProof<Blake2sMerkleHasher>,
        settings: CircuitSettings,
    ) -> Result<(), LuminairError>;
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

        CircuitSettings {
            lookups: Lookups { sin: sin_lookup },
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

        Ok(LuminairPie {
            trace_tables,
            execution_resources: ExecutionResources {
                op_counter,
                max_log_size,
            },
        })
    }

    /// Verifies a STWO proof.
    ///
    /// Takes a `LuminairProof` and `CircuitSettings` as input.
    /// It orchestrates the STWO verification protocol:
    /// 1. Sets up the verifier, channel, and commitment scheme.
    /// 2. Reads commitments for preprocessed, main, and interaction traces from the proof.
    /// 3. Derives interaction elements using Fiat-Shamir.
    /// 4. Constructs the AIR components (constraints) based on the claims and interaction elements.
    /// 5. Verifies the STARK proof.
    /// Returns `Ok(())` if the proof is valid, otherwise returns a `LuminairError`.
    fn verify(
        &self,
        LuminairProof {
            claim,
            interaction_claim,
            proof,
        }: LuminairProof<Blake2sMerkleHasher>,
        settings: CircuitSettings,
    ) -> Result<(), LuminairError> {
        // Convert lookups in circuit settings to preprocessed column.
        let lut_cols = lookups_to_preprocessed_column(&settings.lookups);
        let preprocessed_trace = PreProcessedTrace::new(lut_cols);

        // ┌──────────────────────────┐
        // │     Protocol Setup       │
        // └──────────────────────────┘
        let config = PcsConfig::default();
        let channel = &mut Blake2sChannel::default();
        let commitment_scheme_verifier =
            &mut CommitmentSchemeVerifier::<Blake2sMerkleChannel>::new(config);

        // Prepare log sizes for each phase
        let mut log_sizes = claim.log_sizes();
        log_sizes[PREPROCESSED_TRACE_IDX] = preprocessed_trace.log_sizes();

        // ┌───────────────────────────────────────────────┐
        // │   Interaction Phase 0 - Preprocessed Trace    │
        // └───────────────────────────────────────────────┘
        commitment_scheme_verifier.commit(
            proof.commitments[PREPROCESSED_TRACE_IDX],
            &log_sizes[PREPROCESSED_TRACE_IDX],
            channel,
        );

        // ┌───────────────────────────────────────┐
        // │    Interaction Phase 1 - Main Trace   │
        // └───────────────────────────────────────┘
        claim.mix_into(channel);
        commitment_scheme_verifier.commit(
            proof.commitments[ORIGINAL_TRACE_IDX],
            &log_sizes[ORIGINAL_TRACE_IDX],
            channel,
        );

        // ┌───────────────────────────────────────────────┐
        // │    Interaction Phase 2 - Interaction Trace    │
        // └───────────────────────────────────────────────┘
        let interaction_elements = LuminairInteractionElements::draw(channel);

        // Validate LogUp sum
        if !log_sum_valid(&interaction_claim) {
            return Err(LuminairError::InvalidLogUp("Invalid LogUp sum".to_string()));
        }

        interaction_claim.mix_into(channel);
        commitment_scheme_verifier.commit(
            proof.commitments[INTERACTION_TRACE_IDX],
            &log_sizes[INTERACTION_TRACE_IDX],
            channel,
        );

        // ┌──────────────────────────┐
        // │    Proof Verification    │
        // └──────────────────────────┘
        let component_builder = LuminairComponents::new(
            &claim,
            &interaction_elements,
            &interaction_claim,
            &preprocessed_trace,
            &settings.lookups,
        );
        let components = component_builder.components();

        verify(&components, channel, commitment_scheme_verifier, proof)
            .map_err(LuminairError::StwoVerifierError)
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
