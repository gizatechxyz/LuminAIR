use stwo_prover::relation;

pub mod table;
pub mod component;

// Defines the relation for the LUT elements.
// It allows to constrain LUTs.
relation!(SinLookupElements, 2);
