use super::{assert_close, random_vec_rng};
use crate::graph::LuminairGraph;
use crate::StwoCompiler;
use crate::{binary_test, unary_test};
use luminal::prelude::*;
use luminal_cpu::CPUCompiler;
use rand::{rngs::StdRng, SeedableRng};

// The tests are inspired by Luminal's CUDA tests:
// https://github.com/raphaelDkhn/luminal/blob/main/crates/luminal_cuda/src/tests/fp32.rs

// =============== BINARY ===============

binary_test!(|a, b| a + b, test_add, f32, false);
