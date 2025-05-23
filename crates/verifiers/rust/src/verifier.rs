use luminair_air::{
    components::{LuminairComponents, LuminairInteractionElements},
    preprocessed::{lookups_to_preprocessed_column, PreProcessedTrace},
    settings::CircuitSettings,
    utils::log_sum_valid,
};
use luminair_prover::LuminairProof;
use luminair_utils::LuminairError;

use stwo_prover::{
    constraint_framework::{INTERACTION_TRACE_IDX, ORIGINAL_TRACE_IDX, PREPROCESSED_TRACE_IDX},
    core::{
        channel::Blake2sChannel,
        pcs::{CommitmentSchemeVerifier, PcsConfig},
        prover,
        vcs::blake2_merkle::{Blake2sMerkleChannel, Blake2sMerkleHasher},
    },
};

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
pub fn verify(
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

    prover::verify(&components, channel, commitment_scheme_verifier, proof)
        .map_err(LuminairError::StwoVerifierError)
}
