use stwo_prover::relation;

pub mod component;
pub mod table;

// Defines the relation for the LUT elements.
// It allows to constrain LUTs.
relation!(SinLookupElements, 2);
