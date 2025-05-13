use crate::{
    op::{
        prim::{CopyFromStwo, CopyToStwo, LuminairConstant},
        HasProcessTrace,
    },
    settings::CircuitSettings,
    utils::compute_padded_range_from_srcs,
};
use luminair_air::{
    components::{
        add::{
            self,
            table::{AddColumn, AddTraceTable},
        },
        lookups::{
            self,
            sin::{table::SinLookupTraceTable, SinLookup},
            Lookups,
        },
        lessthan::{
            self,
            table::{LessThanColumn, LessThanTable},
        max_reduce::{
            self,
            table::{MaxReduceColumn, MaxReduceTraceTable},
        },
        mul::{
            self,
            table::{MulColumn, MulTraceTable},
        },
        recip::{
            self,
            table::{RecipColumn, RecipTraceTable},
        },
        sin::{
            self,
            table::{SinColumn, SinTraceTable},
        },
        sum_reduce::{
            self,
            table::{SumReduceColumn, SumReduceTraceTable},
        },
        LuminairComponents, LuminairInteractionElements, TraceError,
    },
    pie::{
        ExecutionResources, InputInfo, LuminairPie, NodeInfo, OpCounter, OutputInfo, TraceTable,
    },
    preprocessed::{
        lookups_to_preprocessed_column,
        LookupLayout,
        PreProcessedTrace,
        Range,
        SinPreProcessed, // SinLUT
    },
    utils::{calculate_log_size, log_sum_valid},
    LuminairClaim, LuminairInteractionClaim, LuminairInteractionClaimGenerator, LuminairProof,
};
use luminal::{
    op::*,
    prelude::{petgraph::visit::EdgeRef, *},
};
use numerair::Fixed;
use stwo_prover::{
    constraint_framework::{INTERACTION_TRACE_IDX, ORIGINAL_TRACE_IDX, PREPROCESSED_TRACE_IDX},
    core::{
        backend::simd::SimdBackend,
        channel::Blake2sChannel,
        pcs::{CommitmentSchemeProver, CommitmentSchemeVerifier, PcsConfig},
        poly::circle::{CanonicCoset, PolyOps},
        prover::{self, verify, ProvingError, VerificationError},
        vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
    },
};
use thiserror::Error;

/// Errors that can occur during LuminAIR graph processing, proof generation, or verification.
#[derive(Clone, Debug, Error)]
pub enum LuminairError {
    #[error(transparent)]
    TraceError(#[from] TraceError),

    #[error("Main trace generation failed.")]
    MainTraceEvalGenError,

    #[error("Interaction trace generation failed.")]
    InteractionTraceEvalGenError,

    #[error(transparent)]
    ProverError(#[from] ProvingError),

    #[error(transparent)]
    StwoVerifierError(#[from] VerificationError),

    #[error("{0} LogUp values do not match.")]
    InvalidLogUp(String),
}

/// Trait defining the core functionality of a LuminAIR computation graph.
///
/// Provides methods to generate execution traces, retrieve outputs, and handle proof
/// generation and verification using Stwo.
pub trait LuminairGraph {
    /// Infers circuit settings using simulated representative inputs.
    fn gen_circuit_settings(&mut self) -> CircuitSettings;

    /// Generates an execution trace for the graph's computation.
    fn gen_trace(&mut self, settings: &mut CircuitSettings) -> Result<LuminairPie, TraceError>;

    /// Generates a proof of the graph's execution using the provided trace.
    fn prove(
        &mut self,
        pie: LuminairPie,
        settings: CircuitSettings,
    ) -> Result<LuminairProof<Blake2sMerkleHasher>, LuminairError>;

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
    fn gen_trace(&mut self, settings: &mut CircuitSettings) -> Result<LuminairPie, TraceError> {
        // Track the number of views pointing to each tensor so we know when to clear
        if self.linearized_graph.is_none() {
            self.toposort();
        }

        let mut consumers = self.consumers_map.as_ref().unwrap().clone();
        let mut dim_stack = Vec::new();

        // Initializes operator counter
        let mut op_counter = OpCounter::default();

        // Initializes table for each operator
        let mut add_table = AddTable::new();
        let mut mul_table = MulTable::new();
        let mut lessthan_table = LessThanTable::new();
        let mut recip_table = RecipTable::new();
        let mut sum_reduce_table = SumReduceTable::new();
        let mut max_reduce_table = MaxReduceTable::new();
        let mut add_table = AddTraceTable::new();
        let mut mul_table = MulTraceTable::new();
        let mut recip_table = RecipTraceTable::new();
        let mut sin_table = SinTraceTable::new();
        let mut sin_lookup_table = SinLookupTraceTable::new();
        let mut sum_reduce_table = SumReduceTraceTable::new();
        let mut max_reduce_table = MaxReduceTraceTable::new();

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

            let node_info = NodeInfo {
                inputs: input_info,
                output: OutputInfo {
                    is_final_output: is_direct_output || is_output_via_copy,
                },
                num_consumers: *consumers.get(&(*node, 0)).unwrap_or(&0) as u32,
                id: node.index() as u32,
            };

            // Get operator and dispatch to appropriate process_trace handler
            let node_op = &mut *self.graph.node_weight_mut(*node).unwrap();

            let tensors =
                if <Box<dyn Operator> as HasProcessTrace<AddColumn,
        AddTable>>::has_process_trace(
                    node_op,
                ) {
                    let tensors = <Box<dyn Operator> as HasProcessTrace<
                        AddColumn,
                        AddTable,
                    >>::call_process_trace(
                        node_op, srcs, &mut add_table, &node_info
                    )
                    .unwrap();
                    *op_counter.add.get_or_insert(0) += 1;

                    tensors
                }  else if <Box<dyn Operator> as HasProcessTrace<MulColumn,
        MulTable>>::has_process_trace(
                    node_op,
                ) {
                    let tensors = <Box<dyn Operator> as HasProcessTrace<
                        MulColumn,
                        MulTable,
                    >>::call_process_trace(
                        node_op, srcs, &mut mul_table, &node_info
                    )
                    .unwrap();
                    *op_counter.mul.get_or_insert(0) += 1;

                    tensors
                } else if <Box<dyn Operator> as HasProcessTrace<LessThanColumn,
        LessThanTable>>::has_process_trace(
                    node_op,
                ) {
                    let tensors = <Box<dyn Operator> as HasProcessTrace<
                        LessThanColumn,
                        LessThanTable,
                    >>::call_process_trace(
                        node_op, srcs, &mut lessthan_table, &node_info
                    )
                    .unwrap();
                    *op_counter.lessthan.get_or_insert(0) += 1;

                    tensors
                } else if <Box<dyn Operator> as HasProcessTrace<SumReduceColumn,
        SumReduceTable>>::has_process_trace(
                    node_op,
                ) {
                    let tensors = <Box<dyn Operator> as HasProcessTrace<
                        SumReduceColumn,
                        SumReduceTable,
                    >>::call_process_trace(
                        node_op, srcs, &mut sum_reduce_table, &node_info
                    )
                    .unwrap();
                    *op_counter.sum_reduce.get_or_insert(0) += 1;

                    tensors
                } else if <Box<dyn Operator> as HasProcessTrace<RecipColumn,
        RecipTable>>::has_process_trace(
                    node_op,
                ) {
                    let tensors = <Box<dyn Operator> as HasProcessTrace<
                    RecipColumn,
                    RecipTable,
                    >>::call_process_trace(
                        node_op, srcs, &mut recip_table, &node_info
                    )
                    .unwrap();
                    *op_counter.recip.get_or_insert(0) += 1;

                    tensors
                } else if <Box<dyn Operator> as HasProcessTrace<MaxReduceColumn, MaxReduceTable>>::has_process_trace(
                    node_op,
                ) {
                    let tensors = <Box<dyn Operator> as HasProcessTrace<
                    MaxReduceColumn,
                    MaxReduceTable,
                    >>::call_process_trace(
                        node_op, srcs, &mut max_reduce_table, &node_info
                    )
                    .unwrap();
                    *op_counter.max_reduce.get_or_insert(0) += 1;

                    tensors
                }
                else {
                    // Handle other operators or fallback
                    node_op.process(srcs)
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
                        op_counter.mul += 1;
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
                        op_counter.mul += 1;
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
                        op_counter.mul += 1;
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
                        op_counter.mul += 1;
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

        if !lessthan_table.table.is_empty() {
            let log_size = calculate_log_size(lessthan_table.table.len());
            max_log_size = max_log_size.max(log_size);

            table_traces.push(TableTrace::from_lessthan(lessthan_table));
        }

        Ok(LuminairPie {
            trace_tables,
            execution_resources: ExecutionResources {
                op_counter,
                max_log_size,
            },
        })
    }

    /// Generates a STWO proof for the computation graph execution.
    ///
    /// Takes the `LuminairPie` (containing execution traces) and `CircuitSettings`.
    /// It orchestrates the STWO proving protocol:
    /// 1. Sets up the prover, channel, and commitment scheme.
    /// 2. Commits to the preprocessed trace.
    /// 3. Commits to the main execution trace components (add, mul, sin, etc.).
    /// 4. Commits to the interaction trace.
    /// 5. Executes the Stwo prover.
    /// Returns a `LuminairProof` containing the claims and the STARK proof.
    fn prove(
        &mut self,
        pie: LuminairPie,
        settings: CircuitSettings,
    ) -> Result<LuminairProof<Blake2sMerkleHasher>, LuminairError> {
        // ┌──────────────────────────┐
        // │     Protocol Setup       │
        // └──────────────────────────┘
        tracing::info!("Protocol Setup");
        let config: PcsConfig = PcsConfig::default();
        let max_log_size = pie.execution_resources.max_log_size;
        let twiddles = SimdBackend::precompute_twiddles(
            CanonicCoset::new(max_log_size + config.fri_config.log_blowup_factor + 2)
                .circle_domain()
                .half_coset,
        );
        // Setup protocol.
        let channel = &mut Blake2sChannel::default();
        let mut commitment_scheme =
            CommitmentSchemeProver::<_, Blake2sMerkleChannel>::new(config, &twiddles);

        // ┌───────────────────────────────────────────────┐
        // │   Interaction Phase 0 - Preprocessed Trace    │
        // └───────────────────────────────────────────────┘

        tracing::info!("Preprocessed Trace");
        // Convert lookups in circuit settings to preprocessed column.
        let lut_cols = lookups_to_preprocessed_column(&settings.lookups);
        let preprocessed_trace = PreProcessedTrace::new(lut_cols);
        let mut tree_builder = commitment_scheme.tree_builder();
        tree_builder.extend_evals(preprocessed_trace.gen_trace());
        // Commit the preprocessed trace
        tree_builder.commit(channel);

        // ┌───────────────────────────────────────┐
        // │    Interaction Phase 1 - Main Trace   │
        // └───────────────────────────────────────┘

        tracing::info!("Main Trace");
        let mut main_claim = LuminairClaim::default();
        let mut interaction_claim_gen = LuminairInteractionClaimGenerator::default();
        let mut tree_builder = commitment_scheme.tree_builder();

            processed_traces.push((trace.clone(), claim_type.clone()));

            // Add the trace to the commit tree.
            tree_builder.extend_evals(trace.clone());

            // Update the main claim based the correct claim type
            match claim_type {
                ClaimType::Add(claim) => main_claim.add = Some(claim),
                ClaimType::Mul(claim) => main_claim.mul = Some(claim),
                ClaimType::LessThan(claim) => main_claim.lessthan = Some(claim),
                ClaimType::SumReduce(claim) => main_claim.sum_reduce = Some(claim),
                ClaimType::Recip(claim) => main_claim.recip = Some(claim),
                ClaimType::MaxReduce(claim) => main_claim.max_reduce = Some(claim),
        for table in pie.trace_tables.clone() {
            match table {
                TraceTable::Add { table } => {
                    let claim_gen = add::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.add = Some(cl.clone());
                    interaction_claim_gen.add = Some(in_cl_gen);
                }
                TraceTable::Mul { table } => {
                    let claim_gen = mul::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.mul = Some(cl.clone());
                    interaction_claim_gen.mul = Some(in_cl_gen);
                }
                TraceTable::Recip { table } => {
                    let claim_gen = recip::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.recip = Some(cl.clone());
                    interaction_claim_gen.recip = Some(in_cl_gen);
                }
                TraceTable::Sin { table } => {
                    let claim_gen = sin::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.sin = Some(cl.clone());
                    interaction_claim_gen.sin = Some(in_cl_gen);
                }
                TraceTable::SinLookup { table } => {
                    let claim_gen = lookups::sin::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.sin_lookup = Some(cl.clone());
                    interaction_claim_gen.sin_lookup = Some(in_cl_gen);
                }
                TraceTable::SumReduce { table } => {
                    let claim_gen = sum_reduce::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.sum_reduce = Some(cl.clone());
                    interaction_claim_gen.sum_reduce = Some(in_cl_gen);
                }
                TraceTable::MaxReduce { table } => {
                    let claim_gen = max_reduce::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.max_reduce = Some(cl.clone());
                    interaction_claim_gen.max_reduce = Some(in_cl_gen);
                }
            }
        }
        // Mix the claim into the Fiat-Shamir channel.
        main_claim.mix_into(channel);
        // Commit the main trace.
        tree_builder.commit(channel);

        // ┌───────────────────────────────────────────────┐
        // │    Interaction Phase 2 - Interaction Trace    │
        // └───────────────────────────────────────────────┘

        tracing::info!("Interaction Trace");
        let interaction_elements = LuminairInteractionElements::draw(channel);
        let mut interaction_claim = LuminairInteractionClaim::default();

        let lookup_elements = &interaction_elements.node_lookup_elements;

        for (trace, claim_type) in processed_traces {
            match claim_type {
                ClaimType::Add(_) => {
                    let (tr, cl) =
                        add::table::interaction_trace_evaluation(&trace, lookup_elements).unwrap();
                    tree_builder.extend_evals(tr);
                    interaction_claim.add = Some(cl);
                }
                ClaimType::Mul(_) => {
                    let (tr, cl) =
                        mul::table::interaction_trace_evaluation(&trace, lookup_elements).unwrap();
                    tree_builder.extend_evals(tr);
                    interaction_claim.mul = Some(cl);
                }
                ClaimType::LessThan(_) => {
                    let (tr, cl) =
                        lessthan::table::interaction_trace_evaluation(&trace, lookup_elements)
                            .unwrap();
                    tree_builder.extend_evals(tr);
                    interaction_claim.lessthan = Some(cl);
                }
                ClaimType::SumReduce(_) => {
                    let (tr, cl) =
                        sum_reduce::table::interaction_trace_evaluation(&trace, lookup_elements)
                            .unwrap();
                    tree_builder.extend_evals(tr);
                    interaction_claim.sum_reduce = Some(cl);
                }
                ClaimType::Recip(_) => {
                    let (tr, cl) =
                        recip::table::interaction_trace_evaluation(&trace, lookup_elements)
                            .unwrap();
                    tree_builder.extend_evals(tr);
                    interaction_claim.recip = Some(cl);
                }
                ClaimType::MaxReduce(_) => {
                    let (tr, cl) =
                        max_reduce::table::interaction_trace_evaluation(&trace, lookup_elements)
                            .unwrap();
                    tree_builder.extend_evals(tr);
                    interaction_claim.max_reduce = Some(cl);
                }
            }
        let mut tree_builder = commitment_scheme.tree_builder();
        let node_elements = &interaction_elements.node_elements;
        let lookup_elements = &interaction_elements.lookup_elements;
        if let Some(claim_gen) = interaction_claim_gen.add {
            let claim = claim_gen.write_interaction_trace(&mut tree_builder, node_elements);
            interaction_claim.add = Some(claim)
        }
        if let Some(claim_gen) = interaction_claim_gen.mul {
            let claim = claim_gen.write_interaction_trace(&mut tree_builder, node_elements);
            interaction_claim.mul = Some(claim)
        }
        if let Some(claim_gen) = interaction_claim_gen.recip {
            let claim = claim_gen.write_interaction_trace(&mut tree_builder, node_elements);
            interaction_claim.recip = Some(claim)
        }
        if let Some(claim_gen) = interaction_claim_gen.sin {
            let claim = claim_gen.write_interaction_trace(
                &mut tree_builder,
                node_elements,
                &lookup_elements.sin,
            );
            interaction_claim.sin = Some(claim)
        }
        if let Some(claim_gen) = interaction_claim_gen.sin_lookup {
            let mut sin_luts = preprocessed_trace.columns_of::<SinPreProcessed>();
            sin_luts.sort_by_key(|c| c.col_index);

            let claim = claim_gen.write_interaction_trace(
                &mut tree_builder,
                &lookup_elements.sin,
                &sin_luts,
            );
            interaction_claim.sin_lookup = Some(claim)
        }
        if let Some(claim_gen) = interaction_claim_gen.sum_reduce {
            let claim = claim_gen.write_interaction_trace(&mut tree_builder, node_elements);
            interaction_claim.sum_reduce = Some(claim)
        }
        if let Some(claim_gen) = interaction_claim_gen.max_reduce {
            let claim = claim_gen.write_interaction_trace(&mut tree_builder, node_elements);
            interaction_claim.max_reduce = Some(claim)
        }
        // Mix the interaction claim into the Fiat-Shamir channel.
        interaction_claim.mix_into(channel);
        // Commit the interaction trace.
        tree_builder.commit(channel);

        // ┌──────────────────────────┐
        // │     Proof Generation     │
        // └──────────────────────────┘
        tracing::info!("Proof Generation");
        let component_builder = LuminairComponents::new(
            &main_claim,
            &interaction_elements,
            &interaction_claim,
            &preprocessed_trace,
            &settings.lookups,
        );
        let components = component_builder.provers();
        let proof = prover::prove::<SimdBackend, _>(&components, channel, commitment_scheme)?;

        Ok(LuminairProof {
            claim: main_claim,
            interaction_claim,
            proof,
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

#[test]
fn test_direct_table_trace_processing() {
    use crate::StwoCompiler;

    let mut cx = Graph::new();
    let a = cx.tensor((10, 10)).set(vec![1.0; 100]);
    let b = cx.tensor((10, 10)).set(vec![2.0; 100]);
    let c = a * b;
    let less = a.less_than(b);
    let mut d = (c + a).retrieve();
    let mut l = less.retrieve();

    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut d);
    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut l);
    let _e = a.sum_reduce(0).retrieve();
    let _f = a.max_reduce(0).retrieve();

    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut d);

    // Generate trace with direct table storage
    let trace = cx.gen_trace().expect("Trace generation failed");

    // Verify that table traces contain both operation types

    let has_add = trace
        .table_traces
        .iter()
        .any(|t| matches!(t, TableTrace::Add { .. }));
    let has_mul = trace
        .table_traces
        .iter()
        .any(|t| matches!(t, TableTrace::Mul { .. }));
    let has_lessthan = trace
        .table_traces
        .iter()
        .any(|t| matches!(t, TableTrace::LessThan { .. }));
    let has_sum_reduce = trace
        .table_traces
        .iter()
        .any(|t| matches!(t, TableTrace::SumReduce { .. }));
    let has_max_reduce = trace
        .table_traces
        .iter()
        .any(|t| matches!(t, TableTrace::MaxReduce { .. }));

    assert!(has_add, "Should contain Add table traces");
    assert!(has_mul, "Should contain Mul table traces");
    assert!(has_lessthan, "Should contain LessThan table traces");
    assert!(has_sum_reduce, "Should contain SumReduce table traces");
    assert!(has_max_reduce, "Should contain MaxReduce table traces");

    // Verify the end-to-end proof pipeline
    let proof = cx.prove(trace).expect("Proof generation failed");
    assert!(
        cx.verify(proof).is_ok(),
        "Proof verification should succeed"
    );
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
