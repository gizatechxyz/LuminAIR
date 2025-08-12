use luminair_air::{
    components::{LuminairComponents, LuminairInteractionElements},
    preprocessed::{lookups_to_preprocessed_column, PreProcessedTrace},
    settings::CircuitSettings,
    utils::log_sum_valid,
};
use luminair_prover::LuminairProof;
use luminair_utils::LuminairError;
use tracing::{info, span, Level};

use stwo::{
    constraint_framework::{INTERACTION_TRACE_IDX, ORIGINAL_TRACE_IDX, PREPROCESSED_TRACE_IDX},
    core::{
        channel::Blake2sChannel,
        pcs::{CommitmentSchemeVerifier, PcsConfig},
        prover,
        vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
    },
};

/// Verifies a LuminAIR proof using the given circuit settings
pub fn verify(
    LuminairProof {
        claim,
        interaction_claim,
        proof,
    }: LuminairProof<Blake2sMerkleHasher>,
    settings: CircuitSettings,
) -> Result<(), LuminairError> {
    let _span = span!(Level::INFO, "luminair_verification").entered();
    info!("🚀 Starting LuminAIR proof verification");

    // Convert lookups in circuit settings to preprocessed column.
    let lut_cols = lookups_to_preprocessed_column(&settings.lookups);
    let preprocessed_trace = PreProcessedTrace::new(lut_cols);

    // ┌──────────────────────────┐
    // │     Protocol Setup       │
    // └──────────────────────────┘
    {
        let _span = span!(Level::INFO, "protocol_setup").entered();
        info!("⚙️  Protocol Setup: Initializing verifier components");
        
        let config = PcsConfig::default();
        let channel = &mut Blake2sChannel::default();
        let commitment_scheme_verifier =
            &mut CommitmentSchemeVerifier::<Blake2sMerkleChannel>::new(config);

        // Prepare log sizes for each phase
        let mut log_sizes = claim.log_sizes();
        log_sizes[PREPROCESSED_TRACE_IDX] = preprocessed_trace.log_sizes();
        
        info!("✅ Protocol Setup: Configuration complete");

        // ┌───────────────────────────────────────────────┐
        // │   Interaction Phase 0 - Preprocessed Trace    │
        // └───────────────────────────────────────────────┘
        {
            let _span = span!(Level::INFO, "interaction_phase_0").entered();
            info!("🔄 Interaction Phase 0: Processing preprocessed trace");
            
            commitment_scheme_verifier.commit(
                proof.commitments[PREPROCESSED_TRACE_IDX],
                &log_sizes[PREPROCESSED_TRACE_IDX],
                channel,
            );
            
            info!("✅ Interaction Phase 0: Preprocessed trace committed");
        }

        // ┌───────────────────────────────────────┐
        // │    Interaction Phase 1 - Main Trace   │
        // └───────────────────────────────────────┘
        {
            let _span = span!(Level::INFO, "interaction_phase_1").entered();
            info!("🔄 Interaction Phase 1: Processing main trace");
            
            claim.mix_into(channel);
            commitment_scheme_verifier.commit(
                proof.commitments[ORIGINAL_TRACE_IDX],
                &log_sizes[ORIGINAL_TRACE_IDX],
                channel,
            );
            
            info!("✅ Interaction Phase 1: Main trace committed");
        }

        // ┌───────────────────────────────────────────────┐
        // │    Interaction Phase 2 - Interaction Trace    │
        // └───────────────────────────────────────────────┘
        {
            let _span = span!(Level::INFO, "interaction_phase_2").entered();
            info!("🔄 Interaction Phase 2: Processing interaction trace");
            
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
            
            info!("✅ Interaction Phase 2: Interaction trace committed");

            // ┌──────────────────────────┐
            // │    Proof Verification    │
            // └──────────────────────────┘
            {
                let _span = span!(Level::INFO, "proof_verification").entered();
                info!("🔍 Proof Verification: Verifying STARK proof");
                
                let component_builder = LuminairComponents::new(
                    &claim,
                    &interaction_elements,
                    &interaction_claim,
                    &preprocessed_trace,
                    &settings.lookups,
                );
                let components = component_builder.components();

                let result = prover::verify(&components, channel, commitment_scheme_verifier, proof)
                    .map_err(LuminairError::StwoVerifierError);
                
                match &result {
                    Ok(()) => info!("✅ Proof Verification: STARK proof is valid"),
                    Err(e) => info!("❌ Proof Verification: STARK proof is invalid - {}", e),
                }
                
                result
            }
        }
    }
}
