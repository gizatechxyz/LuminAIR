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
use sqrt::{
    component::{SqrtComponent, SqrtEval},
    table::SqrtColumn,
};
use stwo::{
    core::{
        air::Component,
        channel::Channel,
        fields::{
            m31::BaseField,
            qm31::{SecureField, SECURE_EXTENSION_DEGREE},
        },
        pcs::TreeVec,
        ColumnVec,
    },
    prover::{
        backend::simd::SimdBackend,
        poly::{circle::CircleEvaluation, BitReversedOrder},
        ComponentProver,
    },
};
use stwo_constraint_framework::relation;

use stwo_constraint_framework::TraceLocationAllocator;

use sum_reduce::{
    component::{SumReduceComponent, SumReduceEval},
    table::SumReduceColumn,
};

use rem::{
    component::{RemComponent, RemEval},
    table::RemColumn,
};

use crate::{
    components::{
        contiguous::{
            component::{ContiguousComponent, ContiguousEval},
            table::ContiguousColumn,
        },
        exp2::{
            component::{Exp2Component, Exp2Eval},
            table::Exp2Column,
        },
        inputs::{
            components::{InputsComponent, InputsEval},
            table::InputsColumn,
        },
        less_than::{
            component::{LessThanComponent, LessThanEval},
            table::LessThanColumn,
        },
        log2::{
            component::{Log2Component, Log2Eval},
            table::Log2Column,
        },
        lookups::{
            exp2::{
                component::{Exp2LookupComponent, Exp2LookupEval},
                table::Exp2LookupColumn,
            },
            log2::{
                component::{Log2LookupComponent, Log2LookupEval},
                table::Log2LookupColumn,
            },
            range_check::{
                component::{RangeCheckLookupComponent, RangeCheckLookupEval},
                table::RangeCheckLookupColumn,
            },
        },
    },
    preprocessed::PreProcessedTrace,
    LuminairClaim, LuminairInteractionClaim,
};

pub mod add;
pub mod contiguous;
pub mod exp2;
pub mod inputs;
pub mod less_than;
pub mod log2;
pub mod lookups;
pub mod max_reduce;
pub mod mul;
pub mod recip;
pub mod rem;
pub mod sin;
pub mod sqrt;
pub mod sum_reduce;

pub type TraceEval = ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>;

pub type AddClaim = Claim<AddColumn>;
pub type MulClaim = Claim<MulColumn>;
pub type RecipClaim = Claim<RecipColumn>;
pub type SinClaim = Claim<SinColumn>;
pub type SinLookupClaim = Claim<SinLookupColumn>;
pub type SumReduceClaim = Claim<SumReduceColumn>;
pub type MaxReduceClaim = Claim<MaxReduceColumn>;
pub type SqrtClaim = Claim<SqrtColumn>;
pub type RemClaim = Claim<RemColumn>;
pub type Exp2Claim = Claim<Exp2Column>;
pub type Exp2LookupClaim = Claim<Exp2LookupColumn>;
pub type Log2Claim = Claim<Log2Column>;
pub type Log2LookupClaim = Claim<Log2LookupColumn>;
pub type LessThanClaim = Claim<LessThanColumn>;
pub type RangeCheckLookupClaim = Claim<RangeCheckLookupColumn>;
pub type InputsClaim = Claim<InputsColumn>;
pub type ContiguousClaim = Claim<ContiguousColumn>;

/// Trait for trace columns to specify their count
pub trait TraceColumn {
    fn count() -> (usize, usize);
}

/// Generic claim structure for any trace column type
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Claim<T: TraceColumn> {
    pub log_size: u32,
    _marker: std::marker::PhantomData<T>,
}

impl<T: TraceColumn> Claim<T> {
    /// Creates a new claim with the specified log size
    pub const fn new(log_size: u32) -> Self {
        Self {
            log_size,
            _marker: std::marker::PhantomData,
        }
    }

    /// Returns the log sizes for main and interaction trace columns
    pub fn log_sizes(&self) -> TreeVec<Vec<u32>> {
        let (main_trace_cols, interaction_trace_cols) = T::count();
        let trace_log_sizes = vec![self.log_size; main_trace_cols];
        let interaction_trace_log_sizes: Vec<u32> =
            vec![self.log_size; SECURE_EXTENSION_DEGREE * interaction_trace_cols];
        TreeVec::new(vec![vec![], trace_log_sizes, interaction_trace_log_sizes])
    }

    /// Mixes the claim's log size into the given channel
    pub fn mix_into(&self, channel: &mut impl Channel) {
        // Mix log_size
        channel.mix_u64(self.log_size.into());
    }
}

/// Enumeration of all possible claim types
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ClaimType {
    Add(Claim<AddColumn>),
    Mul(Claim<MulColumn>),
    Recip(Claim<RecipColumn>),
    Sin(Claim<SinColumn>),
    SinLookup(Claim<SinLookupColumn>),
    SumReduce(Claim<SumReduceColumn>),
    MaxReduce(Claim<MaxReduceColumn>),
    Sqrt(Claim<SqrtColumn>),
    Rem(Claim<RemColumn>),
    Exp2(Claim<Exp2Column>),
    Exp2Lookup(Claim<Exp2LookupColumn>),
    Log2(Claim<Log2Column>),
    Log2Lookup(Claim<Log2LookupColumn>),
    LessThan(Claim<LessThanColumn>),
    RangeCheckLookup(Claim<RangeCheckLookupColumn>),
    Inputs(Claim<InputsColumn>),
    Contiguous(Claim<ContiguousColumn>),
}

/// Interaction claim containing a claimed sum
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InteractionClaim {
    pub claimed_sum: SecureField,
}

impl InteractionClaim {
    /// Mixes the claimed sum into the given channel
    pub fn mix_into(&self, channel: &mut impl Channel) {
        channel.mix_felts(&[self.claimed_sum]);
    }
}

// Interaction elements related to graph node structure/connections.
// Drawn from the channel and used in interaction phase constraints.
relation!(NodeElements, 2);

/// Collection of interaction elements for LuminAIR
#[derive(Clone, Debug)]
pub struct LuminairInteractionElements {
    pub node_elements: NodeElements,
    pub lookup_elements: LookupElements,
}

impl LuminairInteractionElements {
    /// Draws interaction elements from the given channel
    pub fn draw(channel: &mut impl Channel) -> Self {
        let node_elements = NodeElements::draw(channel);
        let lookup_elements = LookupElements::draw(channel);

        Self {
            node_elements,
            lookup_elements,
        }
    }
}

/// Collection of all LuminAIR components
pub struct LuminairComponents {
    add: Option<AddComponent>,
    mul: Option<MulComponent>,
    recip: Option<RecipComponent>,
    sin: Option<SinComponent>,
    sin_lookup: Option<SinLookupComponent>,
    sum_reduce: Option<SumReduceComponent>,
    max_reduce: Option<MaxReduceComponent>,
    sqrt: Option<SqrtComponent>,
    rem: Option<RemComponent>,
    exp2: Option<Exp2Component>,
    exp2_lookup: Option<Exp2LookupComponent>,
    log2: Option<Log2Component>,
    log2_lookup: Option<Log2LookupComponent>,
    less_than: Option<LessThanComponent>,
    range_check_lookup: Option<RangeCheckLookupComponent>,
    inputs: Option<InputsComponent>,
    contiguous: Option<ContiguousComponent>,
}

impl LuminairComponents {
    /// Creates new LuminAIR components from claims and configuration
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

        let sqrt = if let Some(ref sqrt_claim) = claim.sqrt {
            Some(SqrtComponent::new(
                tree_span_provider,
                SqrtEval::new(&sqrt_claim, interaction_elements.node_elements.clone()),
                interaction_claim.sqrt.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let rem = if let Some(ref rem_claim) = claim.rem {
            Some(RemComponent::new(
                tree_span_provider,
                RemEval::new(&rem_claim, interaction_elements.node_elements.clone()),
                interaction_claim.rem.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let exp2 = if let Some(ref exp2_claim) = claim.exp2 {
            let lut_log_size = lookups.exp2.as_ref().map(|s| s.layout.log_size).unwrap();
            Some(Exp2Component::new(
                tree_span_provider,
                Exp2Eval::new(
                    &exp2_claim,
                    interaction_elements.node_elements.clone(),
                    interaction_elements.lookup_elements.exp2.clone(),
                    lut_log_size,
                ),
                interaction_claim.exp2.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let exp2_lookup = if let Some(ref exp2_lookup_claim) = claim.exp2_lookup {
            Some(Exp2LookupComponent::new(
                tree_span_provider,
                Exp2LookupEval::new(
                    &exp2_lookup_claim,
                    interaction_elements.lookup_elements.exp2.clone(),
                ),
                interaction_claim.exp2_lookup.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let log2 = if let Some(ref log2_claim) = claim.log2 {
            let lut_log_size = lookups.log2.as_ref().map(|s| s.layout.log_size).unwrap();
            Some(Log2Component::new(
                tree_span_provider,
                Log2Eval::new(
                    &log2_claim,
                    interaction_elements.node_elements.clone(),
                    interaction_elements.lookup_elements.log2.clone(),
                    lut_log_size,
                ),
                interaction_claim.log2.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let log2_lookup = if let Some(ref log2_lookup_claim) = claim.log2_lookup {
            Some(Log2LookupComponent::new(
                tree_span_provider,
                Log2LookupEval::new(
                    &log2_lookup_claim,
                    interaction_elements.lookup_elements.log2.clone(),
                ),
                interaction_claim.log2_lookup.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let less_than = if let Some(ref less_than_claim) = claim.less_than {
            let lut_log_size = lookups
                .range_check
                .as_ref()
                .map(|s| s.layout.log_size)
                .unwrap();
            Some(LessThanComponent::new(
                tree_span_provider,
                LessThanEval::new(
                    &less_than_claim,
                    interaction_elements.node_elements.clone(),
                    interaction_elements.lookup_elements.range_check.clone(),
                    lut_log_size,
                ),
                interaction_claim.less_than.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let range_check_lookup =
            if let Some(ref range_check_lookup_claim) = claim.range_check_lookup {
                let bit_length = lookups
                    .range_check
                    .as_ref()
                    .map(|s| s.layout.ranges[0])
                    .unwrap();
                Some(RangeCheckLookupComponent::new(
                    tree_span_provider,
                    RangeCheckLookupEval::new(
                        bit_length,
                        &range_check_lookup_claim,
                        interaction_elements.lookup_elements.range_check.clone(),
                    ),
                    interaction_claim
                        .range_check_lookup
                        .as_ref()
                        .unwrap()
                        .claimed_sum,
                ))
            } else {
                None
            };

        let inputs = if let Some(ref inputs_claim) = claim.inputs {
            Some(InputsComponent::new(
                tree_span_provider,
                InputsEval::new(&inputs_claim, interaction_elements.node_elements.clone()),
                interaction_claim.inputs.as_ref().unwrap().claimed_sum,
            ))
        } else {
            None
        };

        let contiguous = if let Some(ref contiguous_claim) = claim.contiguous {
            Some(ContiguousComponent::new(
                tree_span_provider,
                ContiguousEval::new(
                    &contiguous_claim,
                    interaction_elements.node_elements.clone(),
                ),
                interaction_claim.contiguous.as_ref().unwrap().claimed_sum,
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
            sqrt,
            rem,
            exp2,
            exp2_lookup,
            log2,
            log2_lookup,
            less_than,
            range_check_lookup,
            inputs,
            contiguous,
        }
    }

    /// Returns all component provers as a vector
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

        if let Some(ref component) = self.sqrt {
            components.push(component);
        }

        if let Some(ref component) = self.rem {
            components.push(component);
        }
        if let Some(ref component) = self.exp2 {
            components.push(component);
        }

        if let Some(ref component) = self.exp2_lookup {
            components.push(component);
        }

        if let Some(ref component) = self.log2 {
            components.push(component);
        }

        if let Some(ref component) = self.log2_lookup {
            components.push(component);
        }

        if let Some(ref component) = self.less_than {
            components.push(component);
        }

        if let Some(ref component) = self.range_check_lookup {
            components.push(component);
        }

        if let Some(ref component) = self.inputs {
            components.push(component);
        }

        if let Some(ref component) = self.contiguous {
            components.push(component);
        }

        components
    }

    /// Returns all components as a vector
    pub fn components(&self) -> Vec<&dyn Component> {
        self.provers()
            .into_iter()
            .map(|component| component as &dyn Component)
            .collect()
    }
}
