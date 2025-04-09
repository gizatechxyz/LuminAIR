use add::{
    component::{AddComponent, AddEval},
    table::AddColumn,
};
use lessthan::{
    component::{LessThanComponent, LessThanEval},
    table::LessThanColumn,
};
use mul::{
    component::{MulComponent, MulEval},
    table::MulColumn,
};
use sum_reduce::{
    component::{SumReduceComponent, SumReduceEval},
    table::SumReduceColumn,
};
use recip::{
    component::{RecipComponent, RecipEval},
    table::RecipColumn,
};
use serde::{Deserialize, Serialize};
use stwo_prover::{
    constraint_framework::{preprocessed_columns::IsFirst, TraceLocationAllocator},
    core::{
        air::{Component, ComponentProver},
        backend::simd::SimdBackend,
        channel::Channel,
        fields::{m31::BaseField, qm31::SecureField, secure_column::SECURE_EXTENSION_DEGREE},
        pcs::TreeVec,
        poly::{circle::CircleEvaluation, BitReversedOrder},
        ColumnVec,
    },
    relation,
};
use thiserror::Error;

use crate::{LuminairClaim, LuminairInteractionClaim};

pub mod add;
pub mod lessthan;
pub mod mul;
pub mod sum_reduce;
pub mod recip;

/// Errors related to trace operations.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum TraceError {
    /// The component trace is empty.
    #[error("The trace is empty.")]
    EmptyTrace,
}

/// Alias for trace evaluation columns used in Stwo.
pub type TraceEval = ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>;

/// Claim for the Add trace.
pub type AddClaim = Claim<AddColumn>;
/// Claim for the Mul trace.
pub type MulClaim = Claim<MulColumn>;
/// Claim for the LessThan trace.
pub type LessThanClaim = Claim<LessThanColumn>;
/// Claim for the SumReduce trace.
pub type SumReduceClaim = Claim<SumReduceColumn>;
/// Claim for the Recip trace.
pub type RecipClaim = Claim<RecipColumn>;

/// Represents columns of a trace.
pub trait TraceColumn {
    /// Returns the number of columns associated with the specific trace type.
    ///
    /// Main trace columns: first element of the tuple
    /// Interaction trace columns: second element of the tuple
    fn count() -> (usize, usize);
}

/// Represents a claim.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Claim<T: TraceColumn> {
    /// Logarithmic size (base 2) of the trace.
    pub log_size: u32,
    /// Phantom data to associate with the trace column type.
    _marker: std::marker::PhantomData<T>,
}

impl<T: TraceColumn> Claim<T> {
    /// Creates a new claim with the given log size and node information.
    pub const fn new(log_size: u32) -> Self {
        Self {
            log_size,
            _marker: std::marker::PhantomData,
        }
    }

    /// Computes log sizes for preprocessed, main, and interaction traces.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let (main_trace_cols, interaction_trace_cols) = T::count();
        let preprocessed_trace_log_sizes: Vec<u32> = vec![self.log_size];
        let trace_log_sizes = vec![self.log_size; main_trace_cols];
        let interaction_trace_log_sizes: Vec<u32> =
            vec![self.log_size; SECURE_EXTENSION_DEGREE * interaction_trace_cols];
        TreeVec::new(vec![
            preprocessed_trace_log_sizes,
            trace_log_sizes,
            interaction_trace_log_sizes,
        ])
    }

    /// Mix the log size of the table and the node structure to the Fiat-Shamir [`Channel`].
    pub fn mix_into(&self, channel: &mut impl Channel) {
        // Mix log_size
        channel.mix_u64(self.log_size.into());
    }
}

/// Enum representing different types of claims.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClaimType {
    Add(Claim<AddColumn>),
    Mul(Claim<MulColumn>),
    LessThan(Claim<LessThanColumn>),
    SumReduce(Claim<SumReduceColumn>),
    Recip(Claim<RecipColumn>),
}

/// The claim of the interaction phase 2 (with the logUp protocol).
///
/// The claimed sum is the total sum, which is the computed sum of the logUp extension column,
/// including the padding rows.
/// It allows proving that the main trace of a component is either a permutation, or a sublist of
/// another.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InteractionClaim {
    /// The computed sum of the logUp extension column, including padding rows (which are actually
    /// set to a multiplicity of 0).
    pub claimed_sum: SecureField,
}

impl InteractionClaim {
    /// Mix the sum from the logUp protocol into the Fiat-Shamir [`Channel`],
    /// to bound the proof to the trace.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.claimed_sum]);
    }
}

// Defines the relation for the node lookup elements.
// It allows to constrain relationship between nodes.
relation!(NodeElements, 2);

/// All the interaction elements required by the components during the interaction phase 2.
///
/// The elements are drawn from a Fiat-Shamir [`Channel`], currently using the BLAKE2 hash.
#[derive(Clone, Debug)]
pub struct LuminairInteractionElements {
    pub node_lookup_elements: NodeElements,
}

impl LuminairInteractionElements {
    /// Draw all the interaction elements needed for
    /// all the components of the system.
    pub fn draw(channel: &mut impl Channel) -> Self {
        let node_lookup_elements = NodeElements::draw(channel);

        Self {
            node_lookup_elements,
        }
    }
}

/// All the components that consitute LuminAIR.
///
/// Components are used by the prover as a `ComponentProver`,
/// and by the verifier as a `Component`.
pub struct LuminairComponents {
    add: Option<AddComponent>,
    mul: Option<MulComponent>,
    lessthan: Option<LessThanComponent>,
    sum_reduce: Option<SumReduceComponent>,
    recip: Option<RecipComponent>,
}

impl LuminairComponents {
    /// Initializes components from claims and interaction elements.
    pub fn new(
        claim: &LuminairClaim,
        interaction_elements: &LuminairInteractionElements,
        interaction_claim: &LuminairInteractionClaim,
        is_first_log_sizes: &[u32],
    ) -> Self {
        let tree_span_provider = &mut TraceLocationAllocator::new_with_preproccessed_columns(
            &is_first_log_sizes
                .iter()
                .copied()
                .map(|log_size| IsFirst::new(log_size).id())
                .collect::<Vec<_>>(),
        );

        let add = if let Some(ref add_claim) = claim.add {
            Some(AddComponent::new(
                tree_span_provider,
                AddEval::new(
                    &add_claim,
                    interaction_elements.node_lookup_elements.clone(),
                ),
                interaction_claim.add.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let mul = if let Some(ref mul_claim) = claim.mul {
            Some(MulComponent::new(
                tree_span_provider,
                MulEval::new(
                    &mul_claim,
                    interaction_elements.node_lookup_elements.clone(),
                ),
                interaction_claim.mul.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let lessthan = if let Some(ref lessthan_claim) = claim.lessthan {
            Some(LessThanComponent::new(
                tree_span_provider,
                LessThanEval::new(
                    &lessthan_claim,
                    interaction_elements.node_lookup_elements.clone(),
                ),
                interaction_claim.lessthan.as_ref().unwrap().claimed_sum,
        let sum_reduce = if let Some(ref sum_reduce_claim) = claim.sum_reduce {
            Some(SumReduceComponent::new(
                tree_span_provider,
                SumReduceEval::new(
                    &sum_reduce_claim,
                    interaction_elements.node_lookup_elements.clone(),
                ),
                interaction_claim.sum_reduce.as_ref().unwrap().claimed_sum,
              ))
        } else {
            None
        };
      
        let recip = if let Some(ref recip_claim) = claim.recip {
            Some(RecipComponent::new(
                tree_span_provider,
                RecipEval::new(
                    &recip_claim,
                    interaction_elements.node_lookup_elements.clone(),
                ),
                interaction_claim.recip.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        Self { add, mul, lessthan }
        Self { add, mul, sum_reduce, recip }
    }

    /// Returns the `ComponentProver` of each components, used by the prover.
    pub fn provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        let mut components: Vec<&dyn ComponentProver<SimdBackend>> = vec![];

        if let Some(ref add_component) = self.add {
            components.push(add_component);
        }
        if let Some(ref mul_component) = self.mul {
            components.push(mul_component);
        }
        if let Some(ref lessthan_component) = self.lessthan {
            components.push(lessthan_component);
        if let Some(ref sum_reduce_component) = self.sum_reduce {
            components.push(sum_reduce_component);
        }
        if let Some(ref recip_component) = self.recip {
            components.push(recip_component);
        }
        components
    }

    /// Returns the `Component` of each components used by the verifier.
    pub fn components(&self) -> Vec<&dyn Component> {
        self.provers()
            .into_iter()
            .map(|component| component as &dyn Component)
            .collect()
    }
}
