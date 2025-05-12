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

/// Errors that can occur during AIR trace generation or processing.
#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum TraceError {
    /// Indicates that a component trace was unexpectedly empty.
    #[error("The trace is empty.")]
    EmptyTrace,
}

/// Type alias for a vector of circle evaluations representing trace columns.
/// Used commonly as the format for trace data passed to the STWO prover/verifier.
pub type TraceEval = ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>;

/// Type alias for the claim associated with the Add component's trace.
pub type AddClaim = Claim<AddColumn>;
/// Type alias for the claim associated with the Mul component's trace.
pub type MulClaim = Claim<MulColumn>;
/// Type alias for the claim associated with the Recip component's trace.
pub type RecipClaim = Claim<RecipColumn>;
/// Type alias for the claim associated with the Sin component's trace.
pub type SinClaim = Claim<SinColumn>;
/// Type alias for the claim associated with the SinLookup component's trace.
pub type SinLookupClaim = Claim<SinLookupColumn>;
/// Type alias for the claim associated with the SumReduce component's trace.
pub type SumReduceClaim = Claim<SumReduceColumn>;
/// Type alias for the claim associated with the MaxReduce component's trace.
pub type MaxReduceClaim = Claim<MaxReduceColumn>;

/// Trait implemented by trace column definitions (e.g., `AddColumn`).
/// Provides metadata about the number of columns used by the component.
pub trait TraceColumn {
    /// Returns the number of columns for the main trace and interaction trace, respectively.
    ///
    /// This information is used to allocate space in the overall trace commitment tree.
    fn count() -> (usize, usize);
}

/// Generic structure representing a claim associated with a specific component's trace.
///
/// Stores the log2 size of the trace segment and uses `PhantomData` to link to the
/// specific `TraceColumn` type (`T`), allowing access to column count metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Claim<T: TraceColumn> {
    /// Logarithmic size (base 2) of this component's trace segment.
    pub log_size: u32,
    /// Marker associating this claim with a specific `TraceColumn` type (e.g., `AddColumn`).
    _marker: std::marker::PhantomData<T>,
}

impl<T: TraceColumn> Claim<T> {
    /// Creates a new claim for a component trace of the given `log_size`.
    pub const fn new(log_size: u32) -> Self {
        Self {
            log_size,
            _marker: std::marker::PhantomData,
        }
    }

    /// Calculates the log sizes needed for this component in the commitment tree.
    /// Returns a `TreeVec` containing empty (preprocessed), main trace, and interaction trace log sizes.
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let (main_trace_cols, interaction_trace_cols) = T::count();
        let trace_log_sizes = vec![self.log_size; main_trace_cols];
        let interaction_trace_log_sizes: Vec<u32> =
            vec![self.log_size; SECURE_EXTENSION_DEGREE * interaction_trace_cols];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_trace_log_sizes])
    }

    /// Mixes the essential claim data (currently just `log_size`) into the Fiat-Shamir channel.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        // Mix log_size
        channel.mix_u64(self.log_size.into());
    }
}

/// Enum wrapping specific claim types for different AIR components.
/// Allows holding claims of various component types in a single structure (e.g., `LuminairClaim`).
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClaimType {
    /// Claim for an Add component trace.
    Add(Claim<AddColumn>),
    /// Claim for a Mul component trace.
    Mul(Claim<MulColumn>),
    /// Claim for a Recip component trace.
    Recip(Claim<RecipColumn>),
    /// Claim for a Sin component trace.
    Sin(Claim<SinColumn>),
    /// Claim for a SinLookup component trace.
    SinLookup(Claim<SinLookupColumn>),
    /// Claim for a SumReduce component trace.
    SumReduce(Claim<SumReduceColumn>),
    /// Claim for a MaxReduce component trace.
    MaxReduce(Claim<MaxReduceColumn>),
}

/// Represents the claim resulting from the interaction phase (e.g., LogUp protocol).
///
/// Stores the accumulated sum (`claimed_sum`) calculated from the interaction columns.
/// This sum is crucial for verifying relationships like lookups or permutations between trace segments.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InteractionClaim {
    /// The final accumulated value from the interaction protocol.
    /// Must balance out across related components for the proof to be valid.
    pub claimed_sum: SecureField,
}

impl InteractionClaim {
    /// Mixes the `claimed_sum` into the Fiat-Shamir channel.
    /// This binds the result of the interaction phase to the overall proof transcript.
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.claimed_sum]);
    }
}

// Interaction elements related to graph node structure/connections.
// Drawn from the channel and used in interaction phase constraints.
relation!(NodeElements, 2);

/// Container for all interaction elements drawn from the Fiat-Shamir channel.
///
/// These random elements are used in constructing interaction trace columns and constraints.
#[derive(Clone, Debug)]
pub struct LuminairInteractionElements {
    /// Interaction elements related to node connections/structure.
    pub node_elements: NodeElements,
    /// Interaction elements specific to lookup arguments.
    pub lookup_elements: LookupElements,
}

impl LuminairInteractionElements {
    /// Draws all necessary interaction elements (`NodeElements`, `LookupElements`) from the channel.
    pub fn draw(channel: &mut impl Channel) -> Self {
        let node_elements = NodeElements::draw(channel);
        let lookup_elements = LookupElements::draw(channel);

        Self {
            node_elements,
            lookup_elements,
        }
    }
}

/// Aggregates all active AIR components for the LuminAIR system.
///
/// This structure holds instances of the specific STWO component implementations
/// (e.g., `AddComponent`, `MulComponent`) based on the claims generated during the trace phase.
/// It provides methods to access these components as needed by the STWO prover and verifier.
pub struct LuminairComponents {
    /// Optional Add component instance.
    add: Option<AddComponent>,
    /// Optional Mul component instance.
    mul: Option<MulComponent>,
    /// Optional Recip component instance.
    recip: Option<RecipComponent>,
    /// Optional Sin component instance.
    sin: Option<SinComponent>,
    /// Optional SinLookup component instance.
    sin_lookup: Option<SinLookupComponent>,
    /// Optional SumReduce component instance.
    sum_reduce: Option<SumReduceComponent>,
    /// Optional MaxReduce component instance.
    max_reduce: Option<MaxReduceComponent>,
}

impl LuminairComponents {
    /// Creates a `LuminairComponents` instance from collected claims and interaction elements.
    ///
    /// Initializes only the components that have corresponding claims present in `claim`.
    /// Uses a `TraceLocationAllocator` to assign segments within the overall trace commitment tree.
    /// Requires preprocessed trace info and lookup configurations for component setup.
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

    /// Returns a vector of references to the active components, cast as `ComponentProver`.
    /// This is used to provide the prover with the necessary constraint logic and trace generation helpers.
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

    /// Returns a vector of references to the active components, cast as `Component`.
    /// This is used to provide the verifier with the necessary constraint logic.
    pub fn components(&self) -> Vec<&dyn Component> {
        self.provers()
            .into_iter()
            .map(|component| component as &dyn Component)
            .collect()
    }
}
