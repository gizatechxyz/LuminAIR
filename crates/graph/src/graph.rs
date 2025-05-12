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
            table::{AddColumn, AddTable},
        },
        lookups::{self, sin::{table::SinLookupTable, SinLookup}, Lookups},
        max_reduce::{
            self,
            table::{MaxReduceColumn, MaxReduceTable},
        },
        mul::{
            self,
            table::{MulColumn, MulTable},
        },
        recip::{
            self,
            table::{RecipColumn, RecipTable},
        },
        sin::{
            self,
            table::{SinColumn, SinTable},
        },
        sum_reduce::{
            self,
            table::{SumReduceColumn, SumReduceTable},
        },
        LuminairComponents, LuminairInteractionElements, TraceError,
    },
    pie::{
        ExecutionResources, InputInfo, LuminairPie, NodeInfo, OpCounter, OutputInfo, TableTrace,
    },
    preprocessed::{
        lookups_to_preprocessed_column,
        LookupLayout,
        PreProcessedTrace,
        Range, SinPreProcessed, // SinLUT
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
        backend::simd::SimdBackend, channel::Blake2sChannel,pcs::{CommitmentSchemeProver, CommitmentSchemeVerifier, PcsConfig}, poly::circle::{CanonicCoset, PolyOps}, prover::{self, verify, ProvingError, VerificationError}, vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher}
    },
};
use thiserror::Error;

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

impl LuminairGraph for Graph {
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
            if <Box<dyn Operator> as HasProcessTrace<SinColumn, SinTable, SinLookup>>::has_process_trace(op) {
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
        let mut recip_table = RecipTable::new();
        let mut sin_table = SinTable::new();
        let mut sin_lookup_table = SinLookupTable::new();
        let mut sum_reduce_table = SumReduceTable::new();
        let mut max_reduce_table = MaxReduceTable::new();

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

            let tensors = match () {
                _ if <Box<dyn Operator> as HasProcessTrace<AddColumn, AddTable, ()>>::has_process_trace(node_op) => {
                    op_counter.add += 1;
                    <Box<dyn Operator> as HasProcessTrace<AddColumn, AddTable, ()>>::call_process_trace(
                        node_op, srcs, &mut add_table, &node_info, &mut ()
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<MulColumn, MulTable, ()>>::has_process_trace(node_op) => {
                    op_counter.mul += 1;
                    <Box<dyn Operator> as HasProcessTrace<MulColumn, MulTable, ()>>::call_process_trace(
                        node_op, srcs, &mut mul_table, &node_info, &mut ()
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<RecipColumn, RecipTable, ()>>::has_process_trace(node_op) => {
                    op_counter.mul += 1;
                    <Box<dyn Operator> as HasProcessTrace<RecipColumn, RecipTable, ()>>::call_process_trace(
                        node_op, srcs, &mut recip_table, &node_info, &mut ()
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<SinColumn, SinTable, SinLookup>>::has_process_trace(node_op) => {
                    op_counter.mul += 1;
                    match settings.lookups.sin.as_mut() {
                        Some(lookup) => {
                            <Box<dyn Operator> as HasProcessTrace<
                                SinColumn,
                                SinTable,
                                SinLookup,
                            >>::call_process_trace(
                                node_op,
                                srcs,
                                &mut sin_table,
                                &node_info,
                                lookup,
                            )
                            .unwrap()
                        }                
                        None =>  unreachable!("Sin lookup table must be initialised"),
                    }

                }
                _ if <Box<dyn Operator> as HasProcessTrace<SumReduceColumn, SumReduceTable, ()>>::has_process_trace(node_op) => {
                    op_counter.mul += 1;
                    <Box<dyn Operator> as HasProcessTrace<SumReduceColumn, SumReduceTable, ()>>::call_process_trace(
                        node_op, srcs, &mut sum_reduce_table, &node_info, &mut ()
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<MaxReduceColumn, MaxReduceTable, ()>>::has_process_trace(node_op) => {
                    op_counter.mul += 1;
                    <Box<dyn Operator> as HasProcessTrace<MaxReduceColumn, MaxReduceTable, ()>>::call_process_trace(
                        node_op, srcs, &mut max_reduce_table, &node_info, &mut ()
                    ).unwrap()
                }
                _ => node_op.process(srcs)
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
        let mut table_traces = Vec::new();

        if !add_table.table.is_empty() {
            let log_size = calculate_log_size(add_table.table.len());
            max_log_size = max_log_size.max(log_size);
            table_traces.push(TableTrace::from_add(add_table));
        }
        if !mul_table.table.is_empty() {
            let log_size = calculate_log_size(mul_table.table.len());
            max_log_size = max_log_size.max(log_size);
            table_traces.push(TableTrace::from_mul(mul_table));
        }
        if !recip_table.table.is_empty() {
            let log_size = calculate_log_size(recip_table.table.len());
            max_log_size = max_log_size.max(log_size);
            table_traces.push(TableTrace::from_recip(recip_table));
        }
        if !sin_table.table.is_empty() {
            let log_size = calculate_log_size(sin_table.table.len());
            max_log_size = max_log_size.max(log_size);
            table_traces.push(TableTrace::from_sin(sin_table));

            if let Some(lookup) = settings.lookups.sin.as_ref() {
               lookup.add_multiplicities_to_table(&mut sin_lookup_table);
                max_log_size = max_log_size.max(lookup.layout.log_size);
                table_traces.push(TableTrace::from_sin_lookup(sin_lookup_table))
            } // TODO (@raphaelDkhn): though error if LUT not present.
        }
        if !sum_reduce_table.table.is_empty() {
            let log_size = calculate_log_size(sum_reduce_table.table.len());
            max_log_size = max_log_size.max(log_size);
            table_traces.push(TableTrace::from_sum_reduce(sum_reduce_table));
        }
        if !max_reduce_table.table.is_empty() {
            let log_size = calculate_log_size(max_reduce_table.table.len());
            max_log_size = max_log_size.max(log_size);
            table_traces.push(TableTrace::from_max_reduce(max_reduce_table));
        }

        Ok(LuminairPie {
            table_traces,
            execution_resources: ExecutionResources {
                op_counter,
                max_log_size,
            },
        })
    }

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

        for table in pie.table_traces.clone() {
            match table {
                TableTrace::Add { table } => {
                    let claim_gen = add::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.add = Some(cl.clone());
                    interaction_claim_gen.add = Some(in_cl_gen);
                }
                TableTrace::Mul { table } => {
                    let claim_gen = mul::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.mul = Some(cl.clone());
                    interaction_claim_gen.mul = Some(in_cl_gen);
                }
                TableTrace::Recip { table } => {
                    let claim_gen = recip::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.recip = Some(cl.clone());
                    interaction_claim_gen.recip = Some(in_cl_gen);
                }
                TableTrace::Sin { table } => {
                    let claim_gen = sin::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.sin = Some(cl.clone());
                    interaction_claim_gen.sin = Some(in_cl_gen);
                }
                TableTrace::SinLookup { table } => {
                    let claim_gen = lookups::sin::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.sin_lookup = Some(cl.clone());
                    interaction_claim_gen.sin_lookup = Some(in_cl_gen);
                }
                TableTrace::SumReduce { table } => {
                    let claim_gen = sum_reduce::witness::ClaimGenerator::new(table);
                    let (cl, in_cl_gen) = claim_gen.write_trace(&mut tree_builder)?;
                    main_claim.sum_reduce = Some(cl.clone());
                    interaction_claim_gen.sum_reduce = Some(in_cl_gen);
                }
                TableTrace::MaxReduce { table } => {
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
            let claim = claim_gen.write_interaction_trace(&mut tree_builder, node_elements);
            interaction_claim.sin = Some(claim)
        }
        if let Some(claim_gen) = interaction_claim_gen.sin_lookup {
            let mut sin_luts = preprocessed_trace.columns_of::<SinPreProcessed>();
            sin_luts.sort_by_key(|c| c.col_index);

            let claim = claim_gen.write_interaction_trace(&mut tree_builder, &lookup_elements.sin, &sin_luts);
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
