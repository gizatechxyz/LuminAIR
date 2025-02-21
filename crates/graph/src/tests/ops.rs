use super::{assert_close, random_vec_rng};
use crate::binary_test;
use crate::graph::LuminairGraph;
use crate::StwoCompiler;
use luminal::prelude::*;
use luminal_cpu::CPUCompiler;
use rand::{rngs::StdRng, SeedableRng};

// =============== BINARY ===============
binary_test!(|a, b| a + b, test_add, f32);
binary_test!(|a, b| a * b, test_mul, f32);
