use crate::{
    data::{OutputConverter, StwoData},
    op::HasProcessTrace,
};
use luminair_air::{
    components::{
        add::{self, table::AddColumn},
        mul::{self, table::MulColumn},
        ClaimType, LuminairComponents, LuminairInteractionElements, TraceEval,
    },
    pie::{ExecutionResources, InputInfo, LuminairPie, NodeInfo, OpCounter, OutputInfo, Trace},
    serde::SerializableTrace,
    utils::{get_is_first_log_sizes, lookup_sum_valid},
    LuminairClaim, LuminairInteractionClaim, LuminairProof,
};
use luminal::prelude::*;
use stwo_prover::{
    constraint_framework::{
        preprocessed_columns::gen_is_first, INTERACTION_TRACE_IDX, ORIGINAL_TRACE_IDX,
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

/// Trait defining the core functionality of a LuminAIR computation graph.
///
/// Provides methods to generate execution traces, retrieve outputs, and handle proof
/// generation and verification using Stwo.
pub trait LuminairGraph {
    /// Generates an execution trace for the graph’s computation.
    fn gen_trace(&mut self) -> LuminairPie;

    /// Retrieves the output of a node as a vector of `f32` values.
    fn get_output(&mut self, id: NodeIndex) -> Vec<f32>;

    /// Generates a proof of the graph’s execution using the provided trace.
    fn prove(
        &mut self,
        pie: LuminairPie,
    ) -> Result<LuminairProof<Blake2sMerkleHasher>, ProvingError>;

    /// Verifies a proof to ensure integrity of graph’s computation.
    fn verify(&self, proof: LuminairProof<Blake2sMerkleHasher>) -> Result<(), VerificationError>;
}

impl LuminairGraph for Graph {
    fn gen_trace(&mut self) -> LuminairPie {
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

        let mut max_log_size = 0;

        for (node, src_ids) in self.linearized_graph.as_ref().unwrap() {
            if self.tensors.contains_key(&(*node, 0)) {
                continue;
            }

            let mut srcs =
                get_source_tensors(&self.no_delete, &mut self.tensors, src_ids, &consumers);

            // Gather input source information
            let input_info = src_ids
                .iter()
                .map(|(id, _, _)| InputInfo {
                    is_initializer: self.node_weight(*id).unwrap().as_any().is::<Function>(),
                })
                .collect::<Vec<_>>();

            // Get output source information
            let output_info = OutputInfo {
                is_final_output: self.to_retrieve.contains_key(&node),
            };

            let node_info = NodeInfo {
                inputs: input_info,
                output: output_info,
                num_consumers: *consumers.get(&(*node, 0)).unwrap_or(&0),
            };

            // Substitute in the dyn dims
            for (_, st) in srcs.iter_mut() {
                st.resolve_global_dyn_dims_stack(&self.dyn_map, &mut dim_stack);
            }

            // Get operator and try to use process_trace if available
            let node_op = &mut *self.graph.node_weight_mut(*node).unwrap();

            let tensors =
                if <Box<dyn Operator> as HasProcessTrace<AddColumn>>::has_process_trace(node_op) {
                    let (trace, claim, tensors) =
                        <Box<dyn Operator> as HasProcessTrace<AddColumn>>::call_process_trace(
                            node_op, srcs, &node_info,
                        )
                        .unwrap();

                    max_log_size = max_log_size.max(claim.log_size);

                    traces.push(Trace {
                        eval: SerializableTrace::from(&trace),
                        claim: ClaimType::Add(claim),
                        node_info,
                    });
                    *op_counter.add.get_or_insert(0) += 1;

                    tensors
                } else if <Box<dyn Operator> as HasProcessTrace<MulColumn>>::has_process_trace(
                    node_op,
                ) {
                    let (trace, claim, tensors) =
                        <Box<dyn Operator> as HasProcessTrace<MulColumn>>::call_process_trace(
                            node_op, srcs, &node_info,
                        )
                        .unwrap();

                    max_log_size = max_log_size.max(claim.log_size);

                    traces.push(Trace {
                        eval: SerializableTrace::from(&trace),
                        claim: ClaimType::Mul(claim),
                        node_info,
                    });
                    *op_counter.mul.get_or_insert(0) += 1;

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

        LuminairPie {
            traces,
            execution_resources: ExecutionResources {
                op_counter,
                max_log_size,
            },
        }
    }

    fn get_output(&mut self, id: NodeIndex) -> Vec<f32> {
        // Get the shape from the graph edges
        let output_size = if let Some((_, shape)) = self.to_retrieve.get(&id) {
            shape
                .n_elements()
                .to_usize()
                .expect("Failed to get tensor size")
        } else {
            // Fallback to checking graph edges if not in to_retrieve
            self.graph
                .edges_directed(id, petgraph::Direction::Incoming)
                .find_map(|e| e.weight().as_data())
                .map(|(_, _, shape)| {
                    shape
                        .n_elements()
                        .to_usize()
                        .expect("Failed to get tensor size")
                })
                .expect("Could not determine tensor shape")
        };

        if let Some(tensor) = self.tensors.remove(&(id, 0)) {
            if let Some(data) = tensor.downcast_ref::<StwoData>() {
                let converter = OutputConverter::new(data.clone(), output_size);
                return converter.to_f32();
            }
        }
        panic!("No StwoData found for final output conversion");
    }

    fn prove(
        &mut self,
        pie: LuminairPie,
    ) -> Result<LuminairProof<Blake2sMerkleHasher>, ProvingError> {
        // Track the number of views pointing to each tensor so we know when to clear
        if self.linearized_graph.is_none() {
            self.toposort();
        }

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
                .map(gen_is_first::<SimdBackend>),
        );

        // Commit the preprocessed trace
        tree_builder.commit(channel);

        // ┌───────────────────────────────────────┐
        // │    Interaction Phase 1 - Main Trace   │
        // └───────────────────────────────────────┘

        tracing::info!("Main Trace");
        let mut tree_builder = commitment_scheme.tree_builder();
        let mut main_claim = LuminairClaim::init(is_first_log_sizes.clone());

        for trace in pie.traces.clone().into_iter() {
            tree_builder.extend_evals(trace.eval.to_trace());

            match trace.claim {
                ClaimType::Add(claim) => {
                    main_claim.add.push(claim);
                }
                ClaimType::Mul(claim) => {
                    main_claim.mul.push(claim);
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

        // Draw interaction elements
        let interaction_elements =
            LuminairInteractionElements::draw(channel, &pie.execution_resources.op_counter);

        // Generate the interaction trace from the main trace, and compute the logUp sum.
        let mut tree_builder = commitment_scheme.tree_builder();
        let mut interaction_claim = LuminairInteractionClaim::init();

        for trace in pie.traces.into_iter() {
            let node_info = trace.node_info;
            let trace_eval: TraceEval = trace.eval.to_trace();

            match trace.claim {
                ClaimType::Add(_) => {
                    let lookup_elements = &interaction_elements.add_lookup_elements;

                    let (t, c) = add::table::interaction_trace_evaluation(
                        &trace_eval,
                        lookup_elements,
                        &node_info,
                    )
                    .unwrap();

                    tree_builder.extend_evals(t);
                    interaction_claim.add.push(c);
                }
                ClaimType::Mul(_) => {
                    let lookup_elements = &interaction_elements.mul_lookup_elements;

                    let (t, c) = mul::table::interaction_trace_evaluation(
                        &trace_eval,
                        lookup_elements,
                        &node_info,
                    )
                    .unwrap();

                    tree_builder.extend_evals(t);
                    interaction_claim.mul.push(c);
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

        self.reset();

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
    ) -> Result<(), VerificationError> {
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

        let interaction_elements =
            LuminairInteractionElements::draw(channel, &execution_resources.op_counter);

        // Check that the lookup sum is valid, otherwise throw
        if !lookup_sum_valid(&interaction_claim) {
            return Err(VerificationError::InvalidLookup(
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

        verify(&components, channel, commitment_scheme_verifier, proof)
    }
}
