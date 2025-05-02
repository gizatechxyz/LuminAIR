use std::sync::atomic::Ordering;

use crate::{
    op::{
        prim::{CopyFromStwo, CopyToStwo, LuminairConstant},
        HasProcessTrace, Lookup,
    },
    settings::{CircuitSettings, LookupLayouts},
    utils::compute_padded_range_from_srcs,
};
use luminair_air::{
    components::{
        add::{
            self,
            table::{AddColumn, AddTable},
        },
        lookups::{
            sin::table::{SinLookupData, SinLookupTable, SinLookupTableRow},
            Layout, Range,
        },
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
        ClaimType, LuminairComponents, LuminairInteractionElements, TraceError,
    },
    pie::{
        ExecutionResources, InputInfo, LUTMultiplicities, LuminairPie, NodeInfo, OpCounter,
        OutputInfo, TableTrace,
    },
    preprocessed::{PreProcessedColumn, PreProcessedTrace, SinLUT},
    utils::{calculate_log_size, log_sum_valid, AtomicMultiplicityColumn},
    LuminairClaim, LuminairInteractionClaim, LuminairProof,
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
        fields::m31::BaseField,
        pcs::{CommitmentSchemeProver, CommitmentSchemeVerifier, PcsConfig},
        poly::circle::{CanonicCoset, PolyOps},
        prover::{self, verify, ProvingError, VerificationError},
        vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
    },
};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum LuminairError {
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
    fn gen_trace(&mut self, settings: CircuitSettings) -> Result<LuminairPie, TraceError>;

    /// Generates a proof of the graph's execution using the provided trace.
    fn prove(
        &mut self,
        pie: LuminairPie,
        settings: CircuitSettings,
    ) -> Result<LuminairProof<Blake2sMerkleHasher>, ProvingError>;

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

            // Add range
            let op = &*self.graph.node_weight(*node).unwrap();
            if <Box<dyn Operator> as HasProcessTrace<SinColumn, SinTable>>::has_process_trace(op) {
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

        let sin_lookup_layout = if !sin_ranges.is_empty() {
            let layout = Layout::new(coalesce_ranges(sin_ranges));
            Some(layout)
        } else {
            None
        };

        let settings = CircuitSettings {
            lookup_layouts: LookupLayouts {
                sin: sin_lookup_layout,
            },
        };
        settings
    }

    fn gen_trace(&mut self, settings: CircuitSettings) -> Result<LuminairPie, TraceError> {
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
        let mut sum_reduce_table = SumReduceTable::new();
        let mut max_reduce_table = MaxReduceTable::new();
        let mut sin_table = SinTable::new();
        let mut sin_lookup_table = SinLookupTable::new();

        // Initializes lookups
        let mut sin_lookup = if let Some(layout) = settings.lookup_layouts.sin {
            Some(Lookup {
                layout: layout.clone(),
                multiplicities: AtomicMultiplicityColumn::new(1 << layout.log_size),
            })
        } else {
            None
        };

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
                _ if <Box<dyn Operator> as HasProcessTrace<AddColumn, AddTable>>::has_process_trace(node_op) => {
                    op_counter.add += 1;
                    <Box<dyn Operator> as HasProcessTrace<AddColumn, AddTable>>::call_process_trace(
                        node_op, srcs, &mut add_table, &node_info, &mut None
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<MulColumn, MulTable>>::has_process_trace(node_op) => {
                    op_counter.mul += 1;
                    <Box<dyn Operator> as HasProcessTrace<MulColumn, MulTable>>::call_process_trace(
                        node_op, srcs, &mut mul_table, &node_info, &mut None
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<SumReduceColumn, SumReduceTable>>::has_process_trace(node_op) => {
                    op_counter.sum_reduce += 1;
                    <Box<dyn Operator> as HasProcessTrace<SumReduceColumn, SumReduceTable>>::call_process_trace(
                        node_op, srcs, &mut sum_reduce_table, &node_info, &mut None
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<RecipColumn, RecipTable>>::has_process_trace(node_op) => {
                    op_counter.recip += 1;
                    <Box<dyn Operator> as HasProcessTrace<RecipColumn, RecipTable>>::call_process_trace(
                        node_op, srcs, &mut recip_table, &node_info, &mut None
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<MaxReduceColumn, MaxReduceTable>>::has_process_trace(node_op) => {
                    op_counter.max_reduce += 1;
                    <Box<dyn Operator> as HasProcessTrace<MaxReduceColumn, MaxReduceTable>>::call_process_trace(
                        node_op, srcs, &mut max_reduce_table, &node_info, &mut None
                    ).unwrap()
                }
                _ if <Box<dyn Operator> as HasProcessTrace<SinColumn, SinTable>>::has_process_trace(node_op) => {
                    op_counter.sin += 1;
                    <Box<dyn Operator> as HasProcessTrace<SinColumn, SinTable>>::call_process_trace(
                        node_op, srcs, &mut sin_table, &node_info, &mut sin_lookup
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
        if !sin_table.table.is_empty() {
            let log_size = calculate_log_size(sin_table.table.len());
            max_log_size = max_log_size.max(log_size);
            table_traces.push(TableTrace::from_sin(sin_table));
        }

        // Create Lookups
        if let Some(lookup) = sin_lookup {
            for mult in lookup.multiplicities.data {
                sin_lookup_table.add_row(SinLookupTableRow {
                    multiplicity: BaseField::from_u32_unchecked(mult.load(Ordering::Relaxed)),
                });
            }
            table_traces.push(TableTrace::from_sin_lookup(sin_lookup_table));
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
    ) -> Result<LuminairProof<Blake2sMerkleHasher>, ProvingError> {
        // ┌──────────────────────────┐
        // │     Protocol Setup       │
        // └──────────────────────────┘
        tracing::info!("Protocol Setup");
        let config: PcsConfig = PcsConfig::default();

        let mut lut_cols: Vec<Box<dyn PreProcessedColumn>> = Vec::new();
        if pie.execution_resources.op_counter.sin > 0 {
            let col_0 = SinLUT::new(settings.lookup_layouts.sin.clone().unwrap(), 0);
            let col_1 = SinLUT::new(settings.lookup_layouts.sin.unwrap(), 1);
            lut_cols.push(Box::new(col_0.clone()));
            lut_cols.push(Box::new(col_1));
        }
        let preprocessed_trace = PreProcessedTrace::new(lut_cols);
        let max_log_size = preprocessed_trace
            .log_sizes()
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .max(pie.execution_resources.max_log_size);
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
        {
            let mut tree_builder = commitment_scheme.tree_builder();
            tree_builder.extend_evals(preprocessed_trace.gen_trace());
            // Commit the preprocessed trace
            tree_builder.commit(channel);
        }

        // ┌───────────────────────────────────────┐
        // │    Interaction Phase 1 - Main Trace   │
        // └───────────────────────────────────────┘

        tracing::info!("Main Trace");
        let mut main_claim = LuminairClaim::new();
        let mut processed_traces: Vec<(_, ClaimType)> = Vec::with_capacity(pie.table_traces.len());

        {
            let mut tree_builder = commitment_scheme.tree_builder();

            for table_trace in pie.table_traces {
                let (trace, claim_type) = match table_trace.to_trace() {
                    Ok(result) => result,
                    Err(err) => {
                        tracing::error!("Trace evaluation failed: {:?}", err);
                        return Err(ProvingError::ConstraintsNotSatisfied);
                    }
                };

                // Add the trace to the commit tree
                tree_builder.extend_evals(trace.clone());

                // Store trace and claim type for interaction phase
                processed_traces.push((trace, claim_type.clone()));

                // Update the main claim based on the claim type
                match claim_type {
                    ClaimType::Add(claim) => main_claim.add = Some(claim),
                    ClaimType::Mul(claim) => main_claim.mul = Some(claim),
                    ClaimType::SumReduce(claim) => main_claim.sum_reduce = Some(claim),
                    ClaimType::Recip(claim) => main_claim.recip = Some(claim),
                    ClaimType::MaxReduce(claim) => main_claim.max_reduce = Some(claim),
                    ClaimType::Sin(claim) => main_claim.sin = Some(claim),
                    ClaimType::SinLookup(claim) => main_claim.sin_lookup = Some(claim),
                }
            }

            // Mix the claim into the Fiat-Shamir channel.
            main_claim.mix_into(channel);
            // Commit the main trace.
            tree_builder.commit(channel);
        }

        // ┌───────────────────────────────────────────────┐
        // │    Interaction Phase 2 - Interaction Trace    │
        // └───────────────────────────────────────────────┘

        // Draw interaction elements
        let interaction_elements = LuminairInteractionElements::draw(channel);
        let mut interaction_claim = LuminairInteractionClaim::default();
        let node_elements = &interaction_elements.node_elements;

        {
            // Generate the interaction trace from the main trace, and compute the logUp sum.
            let mut tree_builder = commitment_scheme.tree_builder();

            for (trace, claim_type) in processed_traces {
                let (interaction_trace, claim) = match claim_type {
                    ClaimType::Add(_) => {
                        add::table::interaction_trace_evaluation(&trace, node_elements)
                            .map_err(|_| ProvingError::ConstraintsNotSatisfied)?
                    }
                    ClaimType::Mul(_) => {
                        mul::table::interaction_trace_evaluation(&trace, node_elements)
                            .map_err(|_| ProvingError::ConstraintsNotSatisfied)?
                    }
                    ClaimType::SumReduce(_) => {
                        sum_reduce::table::interaction_trace_evaluation(&trace, node_elements)
                            .map_err(|_| ProvingError::ConstraintsNotSatisfied)?
                    }
                    ClaimType::Recip(_) => {
                        recip::table::interaction_trace_evaluation(&trace, node_elements)
                            .map_err(|_| ProvingError::ConstraintsNotSatisfied)?
                    }
                    ClaimType::MaxReduce(_) => {
                        max_reduce::table::interaction_trace_evaluation(&trace, node_elements)
                            .map_err(|_| ProvingError::ConstraintsNotSatisfied)?
                    }
                    ClaimType::Sin(_) => {
                        sin::table::interaction_trace_evaluation(&trace, node_elements)
                            .map_err(|_| ProvingError::ConstraintsNotSatisfied)?
                    }
                    ClaimType::SinLookup(_) => {
                        todo!()
                    }
                };

                // Add trace to the commit tree
                tree_builder.extend_evals(interaction_trace);

                // Update the interaction claim
                match claim_type {
                    ClaimType::Add(_) => interaction_claim.add = Some(claim),
                    ClaimType::Mul(_) => interaction_claim.mul = Some(claim),
                    ClaimType::SumReduce(_) => interaction_claim.sum_reduce = Some(claim),
                    ClaimType::Recip(_) => interaction_claim.recip = Some(claim),
                    ClaimType::MaxReduce(_) => interaction_claim.max_reduce = Some(claim),
                    ClaimType::Sin(_) => interaction_claim.sin = Some(claim),
                    ClaimType::SinLookup(_) => interaction_claim.sin_lookup = Some(claim),
                }
            }

            // Mix the interaction claim into the Fiat-Shamir channel.
            interaction_claim.mix_into(channel);
            // Commit the interaction trace.
            tree_builder.commit(channel);
        }

        // ┌──────────────────────────┐
        // │     Proof Generation     │
        // └──────────────────────────┘
        tracing::info!("Proof Generation");
        let component_builder = LuminairComponents::new(
            &main_claim,
            &interaction_elements,
            &interaction_claim,
            &preprocessed_trace,
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
        // Create preprocessed trace
        // TODO: move preprocessed_trace to function param so we don't regenerate.
        let mut lut_cols: Vec<Box<dyn PreProcessedColumn>> = Vec::new();
        if let Some(layout) = settings.lookup_layouts.sin {
            let col_0 = SinLUT::new(layout.clone(), 0);
            let col_1 = SinLUT::new(layout, 1);
            lut_cols.push(Box::new(col_0.clone()));
            lut_cols.push(Box::new(col_1));
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StwoCompiler;

    #[test]
    fn test_direct_table_trace_processing() {
        let mut cx = Graph::new();
        let a = cx.tensor((10, 10)).set(vec![1.0; 100]);
        let b = cx.tensor((10, 10)).set(vec![2.0; 100]);
        let c = a * b;
        let mut d = (c + a).retrieve();
        let _e = a.sum_reduce(0).retrieve();
        let _f = a.max_reduce(0).retrieve();

        cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut d);

        // Generate circuit settings
        let settings = cx.gen_circuit_settings();

        // Generate trace with direct table storage
        let trace = cx
            .gen_trace(settings.clone())
            .expect("Trace generation failed");

        // Verify that table traces contain both operation types
        let has_add = trace
            .table_traces
            .iter()
            .any(|t| matches!(t, TableTrace::Add { .. }));
        let has_mul = trace
            .table_traces
            .iter()
            .any(|t| matches!(t, TableTrace::Mul { .. }));
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
        assert!(has_sum_reduce, "Should contain SumReduce table traces");
        assert!(has_max_reduce, "Should contain MaxReduce table traces");

        // Verify the end-to-end proof pipeline
        let proof = cx
            .prove(trace, settings.clone())
            .expect("Proof generation failed");
        assert!(
            cx.verify(proof, settings).is_ok(),
            "Proof verification should succeed"
        );
    }

    #[test]
    fn gen_circuit_settings_merges_luts() {
        let mut g = Graph::new();

        // tensor with values in [-100, -1]
        let t1_vals: Vec<f32> = (-100..0).map(|x| x as f32).collect();
        let t1 = g.tensor((t1_vals.len(),)).set(t1_vals);
        // tensor with values in [200, 299]
        let t2_vals: Vec<f32> = (200..300).map(|x| x as f32).collect();
        let t2 = g.tensor((t2_vals.len(),)).set(t2_vals);

        // two distinct Sin nodes
        let r1 = t1.sin();
        let r2 = t2.sin();
        let mut out = (r1 + r2).retrieve();

        g.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut out);

        let settings = g.gen_circuit_settings();

        // assert_eq!(
        //     settings.lut_cols.len(),
        //     2,
        //     "Expect exactly one (x, 1/x) column pair for Sin"
        // );

        // let preprocessed = PreProcessedTrace::new(settings.lut_cols);
        // let ids = preprocessed.ids();

        // assert_eq!(ids[0].id, "sin_lut_0");
        // assert_eq!(ids[1].id, "sin_lut_1");
    }
}
