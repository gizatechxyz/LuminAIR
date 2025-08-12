use num_traits::One;
use stwo_prover::{
    constraint_framework::{EvalAtRow, FrameworkComponent, FrameworkEval, RelationEntry},
    core::fields::m31::M31,
};

use crate::{
    components::{lookups::range_check::RangeCheckLookupElements, LessThanClaim, NodeElements},
    DEFAULT_FP_SCALE_FACTOR, TWO_POW_31_MINUS_1,
};

pub type LessThanComponent = FrameworkComponent<LessThanEval>;

pub struct LessThanEval {
    log_size: u32,
    range_check_log_size: u32,
    node_elements: NodeElements,
    range_check_elements: RangeCheckLookupElements,
}

impl LessThanEval {
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

impl FrameworkEval for LessThanEval {
    fn log_size(&self) -> u32 {
        self.log_size
    }

    fn max_constraint_log_degree_bound(&self) -> u32 {
        std::cmp::max(self.log_size, self.range_check_log_size) + 1
    }

    fn evaluate<E: EvalAtRow>(&self, mut eval: E) -> E {
        // Use 31 bits for the constraint (maximum for M31 field)
        let two_pow_k = E::F::from(M31::from_u32_unchecked(TWO_POW_31_MINUS_1));
        let scale_factor = E::F::from(M31::from_u32_unchecked(DEFAULT_FP_SCALE_FACTOR));

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

        // 4-limb decomposition of diff
        let limb0 = eval.next_trace_mask();
        let limb1 = eval.next_trace_mask();
        let limb2 = eval.next_trace_mask();
        let limb3 = eval.next_trace_mask();

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
        eval.add_constraint(out_val.clone() - ((E::F::one() - borrow.clone()) * scale_factor));

        // Core arithmetic constraint: lhs + diff = rhs + borrow * 2^k
        eval.add_constraint(
            lhs_val.clone() + diff_val.clone() - rhs_val.clone() - (borrow * two_pow_k),
        );

        // Limb decomposition constraint: diff = limb3*2^24 + limb2*2^16 + limb1*2^8 + limb0
        let two_pow_8 = E::F::from(M31::from_u32_unchecked(1u32 << 8));
        let two_pow_16 = E::F::from(M31::from_u32_unchecked(1u32 << 16));
        let two_pow_24 = E::F::from(M31::from_u32_unchecked(1u32 << 24));

        let recomposed_diff = limb3.clone() * two_pow_24
            + limb2.clone() * two_pow_16
            + limb1.clone() * two_pow_8
            + limb0.clone();
        eval.add_constraint(diff_val - recomposed_diff);

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

        // 2. Range check on each limb (four separate 8-bit range checks)

        eval.add_to_relation(RelationEntry::new(
            &self.range_check_elements,
            diff_mult.clone().into(),
            &[limb0],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.range_check_elements,
            diff_mult.clone().into(),
            &[limb1],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.range_check_elements,
            diff_mult.clone().into(),
            &[limb2],
        ));

        eval.add_to_relation(RelationEntry::new(
            &self.range_check_elements,
            diff_mult.into(),
            &[limb3],
        ));

        eval.finalize_logup();
        eval
    }
}
