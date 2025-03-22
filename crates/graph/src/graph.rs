use crate::op::{
    prim::{CopyFromStwo, CopyToStwo, LuminairConstant},
    HasProcessTrace,
};
use luminair_air::{
    components::{
        add::{
            self,
            table::{AddColumn, AddTable},
        },
        max_reduce::{
            self,
            table::{MaxReduceColumn, MaxReduceTable},
        },
        mul::{
            self,
            table::{MulColumn, MulTable},
        },
        ClaimType, LuminairComponents, LuminairInteractionElements, TraceError, TraceEval,
    },
    pie::{ExecutionResources, InputInfo, LuminairPie, NodeInfo, OpCounter, OutputInfo, Trace},
    serde::SerializableTrace,
    utils::{get_is_first_log_sizes, lookup_sum_valid},
    LuminairClaim, LuminairInteractionClaim, LuminairProof,
};
use luminal::{
    op::*,
    prelude::{petgraph::visit::EdgeRef, *},
};
use stwo_prover::{
    constraint_framework::{
        preprocessed_columns::IsFirst, INTERACTION_TRACE_IDX, ORIGINAL_TRACE_IDX,
        PREPROCESSED_TRACE_IDX,
    },
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

#[derive(Clone, Debug, Error)]
pub enum LuminairError {
    #[error(transparent)]
    StwoVerifierError(#[from] VerificationError),

    #[error("{0} lookup values do not match.")]
    InvalidLookup(String),
}

/// Trait defining the core functionality of a LuminAIR computation graph.
///
/// Provides methods to generate execution traces, retrieve outputs, and handle proof
/// generation and verification using Stwo.
pub trait LuminairGraph {
    /// Generates an execution trace for the graph’s computation.
    fn gen_trace(&mut self) -> Result<LuminairPie, TraceError>;

    /// Generates a proof of the graph’s execution using the provided trace.
    fn prove(
        &mut self,
        pie: LuminairPie,
    ) -> Result<LuminairProof<Blake2sMerkleHasher>, ProvingError>;

    /// Verifies a proof to ensure integrity of graph’s computation.
    fn verify(&self, proof: LuminairProof<Blake2sMerkleHasher>) -> Result<(), LuminairError>;
}

impl LuminairGraph for Graph {
    fn gen_trace(&mut self) -> Result<LuminairPie, TraceError> {
        // Track the number of views pointing to each tensor so we know when to clear
        if self.linearized_graph.is_none() {
            self.toposort();
        }

        let mut consumers = self.consumers_map.as_ref().unwrap().clone();
        let mut dim_stack = Vec::new();

        // Initialize trace collectors for different operators
        let mut traces = Vec::new();
        // Initializes operator counter
        let mut op_counter = OpCounter::default();

        // Initilializes table for each operator
        let mut add_table: AddTable = AddTable::new();
        let mut mul_table: MulTable = MulTable::new();
        let mut max_reduce_table: MaxReduceTable = MaxReduceTable::new();

        for (node, src_ids) in self.linearized_graph.as_ref().unwrap() {
            if self.tensors.contains_key(&(*node, 0)) {
                continue;
            }

            let mut srcs =
                get_source_tensors(&self.no_delete, &mut self.tensors, src_ids, &consumers);

            // Gather input source information
            let input_info = src_ids
                .iter()
                .map(|(id, _, _)| {
                    let node_weight = self.node_weight(*id).unwrap();
                    let node_is_function = node_weight.as_any().is::<Function>();
                    let node_is_constant = node_weight.as_any().is::<LuminairConstant>()
                        || node_weight.as_any().is::<luminal::op::Constant>();
                    let node_is_copy_to = node_weight.as_any().is::<CopyToStwo>();

                    // Check if this is a CopyToStwo that wraps a Function node or a Constant
                    let is_copy_of_initializer = if node_is_copy_to {
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
                        is_initializer: node_is_function
                            || node_is_constant
                            || is_copy_of_initializer,
                        id: id.index() as u32,
                    }
                })
                .collect::<Vec<_>>();

            // Get output source information - check if this node is a final output
            // or if it feeds into a CopyFromStwo that's a final output
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

            let output_info = OutputInfo {
                is_final_output: is_direct_output || is_output_via_copy,
            };

            let node_info = NodeInfo {
                inputs: input_info,
                output: output_info,
                num_consumers: *consumers.get(&(*node, 0)).unwrap_or(&0) as u32,
                id: node.index() as u32,
            };

            // Substitute in the dyn dims
            for (_, st) in srcs.iter_mut() {
                st.resolve_global_dyn_dims_stack(&self.dyn_map, &mut dim_stack);
            }

            // Get operator and try to use process_trace if available
            let node_op = &mut *self.graph.node_weight_mut(*node).unwrap();

            let tensors =
                if <Box<dyn Operator> as HasProcessTrace<AddColumn, AddTable>>::has_process_trace(
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
                } else if <Box<dyn Operator> as HasProcessTrace<MulColumn, MulTable>>::has_process_trace(
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
                } else {
                    // Handle other operators or fallback
                    node_op.process(srcs)
                };

            for (i, tensor) in tensors.into_iter().enumerate() {
                self.tensors.insert((*node, i as u8), tensor);
            }

            // Bookkeep remaining consumers
            for (id, ind, _) in src_ids {
                *consumers.get_mut(&(*id, *ind)).unwrap() -= 1;
            }
        }

        self.reset();

        // Convert tables to traces
        let mut max_log_size = 0;

        if !add_table.table.is_empty() {
            let (trace, claim) = add_table.trace_evaluation()?;
            max_log_size = max_log_size.max(claim.log_size);

            traces.push(Trace::new(
                SerializableTrace::from(&trace),
                ClaimType::Add(claim),
            ));
        }
        if !mul_table.table.is_empty() {
            let (trace, claim) = mul_table.trace_evaluation()?;
            max_log_size = max_log_size.max(claim.log_size);

            traces.push(Trace::new(
                SerializableTrace::from(&trace),
                ClaimType::Mul(claim),
            ));
        }
        if !max_reduce_table.table.is_empty() {
            let (trace, claim) = max_reduce_table.trace_evaluation()?;
            max_log_size = max_log_size.max(claim.log_size);

            traces.push(Trace::new(
                SerializableTrace::from(&trace),
                ClaimType::MaxReduce(claim),
            ));
        }

        Ok(LuminairPie {
            traces,
            execution_resources: ExecutionResources {
                op_counter,
                max_log_size,
            },
        })
    }

    fn prove(
        &mut self,
        pie: LuminairPie,
    ) -> Result<LuminairProof<Blake2sMerkleHasher>, ProvingError> {
        // ┌──────────────────────────┐
        // │     Protocol Setup       │
        // └──────────────────────────┘
        tracing::info!("Protocol Setup");
        let config = PcsConfig::default();
        let max_log_size = pie.execution_resources.max_log_size;
        let is_first_log_sizes = get_is_first_log_sizes(max_log_size);
        let twiddles = SimdBackend::precompute_twiddles(
            CanonicCoset::new(max_log_size + config.fri_config.log_blowup_factor + 2)
                .circle_domain()
                .half_coset,
        );
        let channel = &mut Blake2sChannel::default();
        let mut commitment_scheme =
            CommitmentSchemeProver::<_, Blake2sMerkleChannel>::new(config, &twiddles);

        // ┌───────────────────────────────────────────────┐
        // │   Interaction Phase 0 - Preprocessed Trace    │
        // └───────────────────────────────────────────────┘

        tracing::info!("Preprocessed Trace");
        // Generate all preprocessed columns
        let mut tree_builder = commitment_scheme.tree_builder();

        tree_builder.extend_evals(
            is_first_log_sizes
                .iter()
                .copied()
                .map(|log_size| IsFirst::new(log_size).gen_column_simd()),
        );

        // Commit the preprocessed trace
        tree_builder.commit(channel);

        // ┌───────────────────────────────────────┐
        // │    Interaction Phase 1 - Main Trace   │
        // └───────────────────────────────────────┘

        tracing::info!("Main Trace");
        let mut tree_builder = commitment_scheme.tree_builder();
        let mut main_claim = LuminairClaim::new(is_first_log_sizes.clone());

        for trace in pie.traces.clone().into_iter() {
            // Add the components' trace evaluation to the commit tree.
            tree_builder.extend_evals(trace.eval.to_trace());

            match trace.claim {
                ClaimType::Add(claim) => main_claim.add = Some(claim),
                ClaimType::Mul(claim) => main_claim.mul = Some(claim),
                ClaimType::MaxReduce(claim) => main_claim.max_reduce = Some(claim),
            }
        }

        // Mix the claim into the Fiat-Shamir channel.
        main_claim.mix_into(channel);
        // Commit the main trace.
        tree_builder.commit(channel);

        // ┌───────────────────────────────────────────────┐
        // │    Interaction Phase 2 - Interaction Trace    │
        // └───────────────────────────────────────────────┘

        // Draw interaction elements
        let interaction_elements = LuminairInteractionElements::draw(channel);
        // Generate the interaction trace from the main trace, and compute the logUp sum.
        let mut tree_builder = commitment_scheme.tree_builder();
        let mut interaction_claim = LuminairInteractionClaim::default();

        for trace in pie.traces.into_iter() {
            let claim = trace.claim;
            let trace: TraceEval = trace.eval.to_trace();
            let lookup_elements = &interaction_elements.node_lookup_elements;

            match claim {
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
                ClaimType::MaxReduce(_) => {
                    let (tr, cl) =
                        max_reduce::table::interaction_trace_evaluation(&trace, lookup_elements).unwrap();
                    tree_builder.extend_evals(tr);
                    interaction_claim.max_reduce = Some(cl);
                }
            }
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
            &is_first_log_sizes,
        );
        let components = component_builder.provers();
        let proof = prover::prove::<SimdBackend, _>(&components, channel, commitment_scheme)?;

        Ok(LuminairProof {
            claim: main_claim,
            interaction_claim,
            proof,
            execution_resources: pie.execution_resources,
        })
    }

    fn verify(
        &self,
        LuminairProof {
            claim,
            interaction_claim,
            proof,
            execution_resources,
        }: LuminairProof<Blake2sMerkleHasher>,
    ) -> Result<(), LuminairError> {
        // ┌──────────────────────────┐
        // │     Protocol Setup       │
        // └──────────────────────────┘
        let config = PcsConfig::default();
        let channel = &mut Blake2sChannel::default();
        let commitment_scheme_verifier =
            &mut CommitmentSchemeVerifier::<Blake2sMerkleChannel>::new(config);
        let log_sizes = &claim.log_sizes();
        let is_first_log_sizes = get_is_first_log_sizes(execution_resources.max_log_size);

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

        // Check that the lookup sum is valid, otherwise throw
        if !lookup_sum_valid(&interaction_claim) {
            return Err(LuminairError::InvalidLookup(
                "Invalid LogUp sum".to_string(),
            ));
        };

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
            &is_first_log_sizes,
        );
        let components = component_builder.components();
        verify(&components, channel, commitment_scheme_verifier, proof)?;

        Ok(())
    }
}
