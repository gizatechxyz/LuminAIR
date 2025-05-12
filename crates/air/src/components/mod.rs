use std::collections::HashMap;

use add::{
    component::{AddComponent, AddEval},
    table::AddColumn,
};
use lookups::{
    sin::{
        component::{SinLookupComponent, SinLookupEval},
        table::SinLookupColumn,
    },
    LookupElements, Lookups,
};
use max_reduce::{
    component::{MaxReduceComponent, MaxReduceEval},
    table::MaxReduceColumn,
};
use mul::{
    component::{MulComponent, MulEval},
    table::MulColumn,
};
use recip::{
    component::{RecipComponent, RecipEval},
    table::RecipColumn,
};
use serde::{Deserialize, Serialize};
use sin::{
    component::{SinComponent, SinEval},
    table::SinColumn,
};
use stwo_prover::{
    constraint_framework::TraceLocationAllocator,
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

use sum_reduce::{
    component::{SumReduceComponent, SumReduceEval},
    table::SumReduceColumn,
};
use thiserror::Error;

use crate::{preprocessed::PreProcessedTrace, LuminairClaim, LuminairInteractionClaim};

pub mod add;
pub mod lookups;
pub mod max_reduce;
pub mod mul;
pub mod recip;
pub mod sin;
pub mod sum_reduce;

/// Errors related to trace operations.
#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum TraceError {
    /// The component trace is empty.
    #[error("The trace is empty.")]
    EmptyTrace,
}

/// Alias for trace evaluation columns used in Stwo.
pub type TraceEval = ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>;

pub type AddClaim = Claim<AddColumn>;
pub type MulClaim = Claim<MulColumn>;
pub type RecipClaim = Claim<RecipColumn>;
pub type SinClaim = Claim<SinColumn>;
pub type SinLookupClaim = Claim<SinLookupColumn>;
pub type SumReduceClaim = Claim<SumReduceColumn>;
pub type MaxReduceClaim = Claim<MaxReduceColumn>;

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

    /// Computes log sizes for main, and interaction traces.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let (main_trace_cols, interaction_trace_cols) = T::count();
        let trace_log_sizes = vec![self.log_size; main_trace_cols];
        let interaction_trace_log_sizes: Vec<u32> =
            vec![self.log_size; SECURE_EXTENSION_DEGREE * interaction_trace_cols];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_trace_log_sizes])
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
    Recip(Claim<RecipColumn>),
    Sin(Claim<SinColumn>),
    SinLookup(Claim<SinLookupColumn>),
    SumReduce(Claim<SumReduceColumn>),
    MaxReduce(Claim<MaxReduceColumn>),
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

// Defines the relation for the node elements.
// It allows to constrain relationship between nodes.
relation!(NodeElements, 2);

/// All the interaction elements required by the components during the interaction phase 2.
///
/// The elements are drawn from a Fiat-Shamir [`Channel`], currently using the BLAKE2 hash.
#[derive(Clone, Debug)]
pub struct LuminairInteractionElements {
    pub node_elements: NodeElements,
    pub lookup_elements: LookupElements,
}

impl LuminairInteractionElements {
    /// Draw all the interaction elements needed for
    /// all the components of the system.
    pub fn draw(channel: &mut impl Channel) -> Self {
        let node_elements = NodeElements::draw(channel);
        let lookup_elements = LookupElements::draw(channel);

        Self {
            node_elements,
            lookup_elements,
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
    recip: Option<RecipComponent>,
    sin: Option<SinComponent>,
    sin_lookup: Option<SinLookupComponent>,
    sum_reduce: Option<SumReduceComponent>,
    max_reduce: Option<MaxReduceComponent>,
}

impl LuminairComponents {
    /// Initializes components from claims and interaction elements.
    pub fn new(
        claim: &LuminairClaim,
        interaction_elements: &LuminairInteractionElements,
        interaction_claim: &LuminairInteractionClaim,
        preprocessed_trace: &PreProcessedTrace,
        lookups: &Lookups,
    ) -> Self {
        let preprocessed_column_ids = &preprocessed_trace.ids();
        // Create a mapping from preprocessed column ID to log size
        let mut preprocessed_column_log_sizes = HashMap::new();
        for column in preprocessed_trace.columns.iter() {
            preprocessed_column_log_sizes.insert(column.id().id.clone(), column.log_size());
        }

        let tree_span_provider =
            &mut TraceLocationAllocator::new_with_preproccessed_columns(preprocessed_column_ids);

        let add = if let Some(ref add_claim) = claim.add {
            Some(AddComponent::new(
                tree_span_provider,
                AddEval::new(&add_claim, interaction_elements.node_elements.clone()),
                interaction_claim.add.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let mul = if let Some(ref mul_claim) = claim.mul {
            Some(MulComponent::new(
                tree_span_provider,
                MulEval::new(&mul_claim, interaction_elements.node_elements.clone()),
                interaction_claim.mul.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let recip = if let Some(ref recip_claim) = claim.recip {
            Some(RecipComponent::new(
                tree_span_provider,
                RecipEval::new(&recip_claim, interaction_elements.node_elements.clone()),
                interaction_claim.recip.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let sin = if let Some(ref sin_claim) = claim.sin {
            let lut_log_size = lookups.sin.as_ref().map(|s| s.layout.log_size).unwrap();
            Some(SinComponent::new(
                tree_span_provider,
                SinEval::new(
                    &sin_claim,
                    interaction_elements.node_elements.clone(),
                    interaction_elements.lookup_elements.sin.clone(),
                    lut_log_size,
                ),
                interaction_claim.sin.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let sin_lookup = if let Some(ref sin_lookup_claim) = claim.sin_lookup {
            Some(SinLookupComponent::new(
                tree_span_provider,
                SinLookupEval::new(
                    &sin_lookup_claim,
                    interaction_elements.lookup_elements.sin.clone(),
                ),
                interaction_claim.sin_lookup.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let sum_reduce = if let Some(ref sum_reduce_claim) = claim.sum_reduce {
            Some(SumReduceComponent::new(
                tree_span_provider,
                SumReduceEval::new(
                    &sum_reduce_claim,
                    interaction_elements.node_elements.clone(),
                ),
                interaction_claim.sum_reduce.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let max_reduce = if let Some(ref max_reduce_claim) = claim.max_reduce {
            Some(MaxReduceComponent::new(
                tree_span_provider,
                MaxReduceEval::new(
                    &max_reduce_claim,
                    interaction_elements.node_elements.clone(),
                ),
                interaction_claim.max_reduce.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        Self {
            add,
            mul,
            recip,
            sin,
            sin_lookup,
            sum_reduce,
            max_reduce,
        }
    }

    /// Returns the `ComponentProver` of each components, used by the prover.
    pub fn provers(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        let mut components: Vec<&dyn ComponentProver<SimdBackend>> = vec![];

        if let Some(ref component) = self.add {
            components.push(component);
        }

        if let Some(ref component) = self.mul {
            components.push(component);
        }

        if let Some(ref component) = self.recip {
            components.push(component);
        }

        if let Some(ref component) = self.sin {
            components.push(component);
        }

        if let Some(ref component) = self.sin_lookup {
            components.push(component);
        }

        if let Some(ref component) = self.sum_reduce {
            components.push(component);
        }

        if let Some(ref component) = self.max_reduce {
            components.push(component);
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
