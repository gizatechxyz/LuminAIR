use num_traits::One;
use stwo_prover::{
    constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry},
    core::fields::m31::M31,
};

use crate::components::{lookups::range_check::RangeCheckLookupElements, LessThanClaim, NodeElements};

/// The STWO AIR component for element-wise LessThan operations.
pub type LessThanComponent = FrameworkComponent<LessThanEval>;

/// Defines the AIR constraints evaluation logic for the LessThan component.
/// Implements `FrameworkEval` to define trace layout, degrees, and constraints.
/// Relies heavily on LogUp arguments for consistency.
pub struct LessThanEval {
    /// Log2 size of the component's main trace segment.
    log_size: u32,
    /// Log2 size of the preprocessed RangeCheck Lookup Table.
    range_check_log_size: u32,
    /// Interaction elements for node relations (used in input/output LogUp).
    node_elements: NodeElements,
    /// Specific interaction elements for the RangeCheck LUT LogUp.
    range_check_elements: RangeCheckLookupElements,
}

impl LessThanEval {
    /// Creates a new ` LessThan2Eval` instance.
    /// Takes the component's claim, interaction elements for nodes and lookups,
    /// and the log_size of the RangeCheck LUT.
    pub fn new(
        claim: &LessThanClaim,
        node_elements: NodeElements,
        range_check_elements: RangeCheckLookupElements,
        range_check_log_size: u32,
    ) -> Self {
        Self {
            log_size: claim.log_size,
            range_check_log_size,
            node_elements,
            range_check_elements,
        }
    }
}

/// Implements the core constraint evaluation logic for the LessThan component.
impl FrameworkEval for LessThanEval {
    /// Returns the log2 size of this component's main trace segment.
    fn log_size(&self) -> u32 {
        self.log_size
    }

    /// Returns the max log2 degree bound, considering both main trace and LUT sizes.
    fn max_constraint_log_degree_bound(&self) -> u32 {
        std::cmp::max(self.log_size, self.range_check_log_size) + 1
    }

    /// Evaluates the LessThan AIR constraints on a given evaluation point (`eval`).
    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        const BIT_LENGTH: u32 = 16; // TODO: make it dynamic.
        let two_pow_k = E::F::from(M31::from_u32_unchecked(1 << BIT_LENGTH));

        // IDs
        let node_id = eval.next_trace_mask();
        let lhs_id = eval.next_trace_mask();
        let rhs_id = eval.next_trace_mask();
        let idx = eval.next_trace_mask();
        let is_last_idx = eval.next_trace_mask();

        // Next IDs for transition constraints
        let next_node_id = eval.next_trace_mask();
        let next_lhs_id = eval.next_trace_mask();
        let next_rhs_id = eval.next_trace_mask();
        let next_idx = eval.next_trace_mask();

        // Values for consistency constraints
        let lhs_val = eval.next_trace_mask();
        let rhs_val = eval.next_trace_mask();
        let out_val = eval.next_trace_mask();
        let diff_val = eval.next_trace_mask();
        let borrow = eval.next_trace_mask();

        // Multiplicities for interaction constraints
        let lhs_mult = eval.next_trace_mask();
        let rhs_mult = eval.next_trace_mask();
        let out_mult = eval.next_trace_mask();
        let diff_mult = eval.next_trace_mask();

        // ┌─────────────────────────────┐
        // │   Consistency Constraints   │
        // └─────────────────────────────┘

        // The is_last_idx flag is either 0 or 1.
        eval.add_constraint(is_last_idx.clone() * (is_last_idx.clone() - E::F::one()));

        // `borrow` and `out_val` must be boolean and opposite
        eval.add_constraint(borrow.clone() * (borrow.clone() - E::F::one()));
        eval.add_constraint(out_val.clone() + borrow.clone() - E::F::one());

        // Core arithmetic constraint: lhs + diff = rhs + borrow * 2^k
        eval.add_constraint(
            lhs_val.clone() + diff_val.clone() - rhs_val.clone() - (borrow * two_pow_k),
        );

        // ┌────────────────────────────┐
        // │   Transition Constraints   │
        // └────────────────────────────┘

        // If this is not the last index for this operation, then:
        // 1. The next row should be for the same operation on the same tensors.
        // 2. The index should increment by 1.
        let not_last = E::F::one() - is_last_idx;

        // Same node ID
        eval.add_constraint(not_last.clone() * (next_node_id - node_id.clone()));

        // Same tensor IDs
        eval.add_constraint(not_last.clone() * (next_lhs_id - lhs_id.clone()));
        eval.add_constraint(not_last.clone() * (next_rhs_id - rhs_id.clone()));

        // Index increment by 1
        eval.add_constraint(not_last * (next_idx - idx - E::F::one()));

        // ┌─────────────────────────────┐
        // │   Interaction Constraints   │
        // └─────────────────────────────┘

        // 1. Connect inputs and output to the computational graph

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            lhs_mult.into(),
            &[lhs_val, lhs_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            rhs_mult.into(),
            &[rhs_val, rhs_id],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.node_elements,
            out_mult.into(),
            &[out_val, node_id],
        ));

        // 2. Range check on the difference

        eval.add_to_relation(RelationEntry::new(
            &self.range_check_elements,
            diff_mult.into(),
            &[diff_val],
        ));

        eval.finalize_logup();
        eval
    }
}
