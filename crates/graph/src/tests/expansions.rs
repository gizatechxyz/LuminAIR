use crate::{graph::LuminairGraph, StwoCompiler};
use luminair_prover::prover::prove;
use luminair_verifier::verifier::verify;
use luminal_cpu::CPUCompiler;
use luminal::prelude::*;
use rand::{rngs::StdRng, SeedableRng};

use super::random_vec_rng;

fn assert_close(a_vec: &[f32], b_vec: &[f32]) {
    assert_eq!(a_vec.len(), b_vec.len(), "Number of elements doesn't match");
    for (i, (a, b)) in a_vec.iter().zip(b_vec.iter()).enumerate() {
        if (a - b).abs() > 1e-3 {
            panic!(
                "{a} is not close to {b}, index {i}, avg distance: {}",
                a_vec
                    .iter()
                    .zip(b_vec.iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum::<f32>()
                    / a_vec.len() as f32
            );
        }
    }
}

/// Test proving and verification with various expansion scenarios
/// to ensure it's proving and verifying e2e without LogUp errors.
fn test_expansion_scenario<F>(
    name: &str,
    graph_builder: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(&mut Graph) -> GraphTensor + Clone,
{
    println!("Testing expansion scenario: {}", name);

    let mut cx = Graph::new();
    let mut result = graph_builder(&mut cx).retrieve();

    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut result);
    let mut settings = cx.gen_circuit_settings();

    let trace = cx
        .gen_trace(&mut settings)
        .map_err(|e| format!("Trace generation failed for {}: {:?}", name, e))?;
     let (proof, _) = prove(trace, settings.clone())
        .map_err(|e| format!("Proof generation failed for {}: {:?}", name, e))?;
    verify(proof, settings)
        .map_err(|e| format!("Proof verification failed for {}: {:?}", name, e))?;

    let stwo_result = result.data();

    // Compare with CPU execution for correctness
    let mut cx_cpu = Graph::new();
    let mut result_cpu = graph_builder(&mut cx_cpu).retrieve();
    cx_cpu.compile(<(GenericCompiler, CPUCompiler)>::default(), &mut result_cpu);
    cx_cpu.execute();
    let cpu_result = result_cpu.data();

    assert_close(&stwo_result, &cpu_result);
    println!("✅ {} passed", name);
    Ok(())
}

#[test]
fn test_single_dimension_expansion() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("single_dimension_expansion", |cx| {
        let mut rng = StdRng::seed_from_u64(42);
        let a = cx.tensor((2, 3)).set(random_vec_rng(6, &mut rng, false));
        let b = cx.tensor((2, 1)).set(random_vec_rng(2, &mut rng, false));

        // Expand b along dimension 1 to match a
        let b_expanded = b.expand(1, 3);
        a * b_expanded
    })
}

#[test]
fn test_multiple_dimension_expansion() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("multiple_dimension_expansion", |cx| {
        let mut rng = StdRng::seed_from_u64(43);
        let a = cx.tensor((3, 4, 2)).set(random_vec_rng(24, &mut rng, false));
        let b = cx.tensor((1, 1, 2)).set(random_vec_rng(2, &mut rng, false));

        // Expand b along multiple dimensions
        let b_expanded = b.expand(0, 3).expand(1, 4);
        a + b_expanded
    })
}

#[test]
fn test_scalar_broadcasting() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("scalar_broadcasting", |cx| {
        let mut rng = StdRng::seed_from_u64(44);
        let a = cx.tensor((3, 4)).set(random_vec_rng(12, &mut rng, false));
        let scalar = cx.tensor((1, 1)).set(vec![2.5]);

        // Broadcast scalar to match a's shape
        let scalar_expanded = scalar.expand_to((3, 4));
        a * scalar_expanded
    })
}

#[test]
fn test_chained_expansions() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("chained_expansions", |cx| {
        let mut rng = StdRng::seed_from_u64(45);
        let a = cx.tensor((2, 3)).set(random_vec_rng(6, &mut rng, false));
        let b = cx.tensor((1, 3)).set(random_vec_rng(3, &mut rng, false));

        // First operation with expansion
        let b_expanded = b.expand(0, 2);
        let intermediate = a + b_expanded;

        // Second operation with further expansion
        let c = cx.tensor((2, 1)).set(random_vec_rng(2, &mut rng, false));
        let c_expanded = c.expand(1, 3);
        intermediate * c_expanded
    })
}

#[test]
fn test_multiple_consumers_different_expansions() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("multiple_consumers_different_expansions", |cx| {
        let mut rng = StdRng::seed_from_u64(46);
        let base = cx.tensor((2, 2)).set(random_vec_rng(4, &mut rng, false));

        // Consumer 1: expand to (2, 2, 3)
        let consumer1 = base.expand(2, 3);
        let a = cx.tensor((2, 2, 3)).set(random_vec_rng(12, &mut rng, false));
        let result1 = consumer1 * a;

        // Consumer 2: expand to (2, 2, 4)
        let consumer2 = base.expand(2, 4);
        let b = cx.tensor((2, 2, 4)).set(random_vec_rng(16, &mut rng, false));
        let result2 = consumer2 + b;

        // Combine results (sum reduce to make compatible)
        let result1_reduced = result1.sum_reduce(2);
        let result2_reduced = result2.sum_reduce(2);
        result1_reduced + result2_reduced
    })
}

#[test]
fn test_mixed_real_fake_dimensions() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("mixed_real_fake_dimensions", |cx| {
        let mut rng = StdRng::seed_from_u64(47);
        let a = cx.tensor((3, 2, 4)).set(random_vec_rng(24, &mut rng, false));
        let b = cx.tensor((3, 1, 4)).set(random_vec_rng(12, &mut rng, false));

        // Expand only middle dimension (fake), keeping others real
        let b_expanded = b.expand(1, 2);
        a * b_expanded
    })
}

#[test]
fn test_row_vector_broadcasting() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("row_vector_broadcasting", |cx| {
        let mut rng = StdRng::seed_from_u64(48);
        let matrix = cx.tensor((4, 5)).set(random_vec_rng(20, &mut rng, false));
        let row_vec = cx.tensor((1, 5)).set(random_vec_rng(5, &mut rng, false));

        // Broadcast row vector across rows
        let row_expanded = row_vec.expand(0, 4);
        matrix + row_expanded
    })
}

#[test]
fn test_column_vector_broadcasting() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("column_vector_broadcasting", |cx| {
        let mut rng = StdRng::seed_from_u64(49);
        let matrix = cx.tensor((4, 5)).set(random_vec_rng(20, &mut rng, false));
        let col_vec = cx.tensor((4, 1)).set(random_vec_rng(4, &mut rng, false));

        // Broadcast column vector across columns
        let col_expanded = col_vec.expand(1, 5);
        matrix * col_expanded
    })
}

#[test]
fn test_complex_expansion_chain() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("complex_expansion_chain", |cx| {
        let mut rng = StdRng::seed_from_u64(50);

        // Start with different shaped tensors
        let a = cx.tensor((2, 3)).set(random_vec_rng(6, &mut rng, false));
        let b = cx.tensor((1, 3)).set(random_vec_rng(3, &mut rng, false));
        let c = cx.tensor((2, 1)).set(random_vec_rng(2, &mut rng, false));
        let d = cx.tensor((1, 1)).set(vec![1.5]);

        // Create complex expansion patterns
        let b_exp = b.expand(0, 2); // (1,3) -> (2,3)
        let c_exp = c.expand(1, 3); // (2,1) -> (2,3)
        let d_exp = d.expand_to((2, 3)); // (1,1) -> (2,3)

        // Chain operations with expansions
        let step1 = a + b_exp;
        let step2 = step1 * c_exp;
        let step3 = step2 + d_exp;

        // Further expand result for more operations
        let step3_exp = step3.expand(2, 4); // (2,3) -> (2,3,4)
        let e = cx.tensor((2, 3, 4)).set(random_vec_rng(24, &mut rng, false));

        step3_exp * e
    })
}

#[test]
fn test_nested_operations_with_expansions() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("nested_operations_with_expansions", |cx| {
        let mut rng = StdRng::seed_from_u64(51);

        // Base tensors with different shapes
        let x = cx.tensor((3, 2)).set(random_vec_rng(6, &mut rng, false));
        let y = cx.tensor((1, 2)).set(random_vec_rng(2, &mut rng, false));
        let z = cx.tensor((3, 1)).set(random_vec_rng(3, &mut rng, false));

        // Nested operations: (x + y_expanded) * (x + z_expanded)
        let y_exp = y.expand(0, 3);
        let z_exp = z.expand(1, 2);

        let left_side = x + y_exp;
        let right_side = x + z_exp;

        left_side * right_side
    })
}

#[test]
fn test_reduction_after_expansion() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("reduction_after_expansion", |cx| {
        let mut rng = StdRng::seed_from_u64(52);

        let base = cx.tensor((2, 3)).set(random_vec_rng(6, &mut rng, false));
        let weights = cx.tensor((1, 3)).set(random_vec_rng(3, &mut rng, false));

        // Expand and multiply
        let weights_exp = weights.expand(0, 2);
        let weighted = base * weights_exp;

        // Then reduce - this creates interesting LogUp patterns
        weighted.sum_reduce(1)
    })
}

#[test]
fn test_large_expansion_factors() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("large_expansion_factors", |cx| {
        let mut rng = StdRng::seed_from_u64(53);

        // Small tensor expanded to large size
        let small = cx.tensor((1, 1)).set(vec![3.14]);
        let large = cx.tensor((8, 16)).set(random_vec_rng(128, &mut rng, false));

        // Large expansion factor: 1*1 -> 8*16 = 128x expansion
        let small_exp = small.expand_to((8, 16));

        large + small_exp
    })
}

#[test]
fn test_expansion_with_unary_operations() -> Result<(), Box<dyn std::error::Error>> {
    test_expansion_scenario("expansion_with_unary_operations", |cx| {
        let mut rng = StdRng::seed_from_u64(54);

        let base = cx.tensor((2, 2)).set(random_vec_rng(4, &mut rng, false));

        // Apply unary operation, then expand
        let processed = base.sin();
        let expanded = processed.expand(2, 3);

        // Use expanded result in binary operation
        let other = cx.tensor((2, 2, 3)).set(random_vec_rng(12, &mut rng, false));
        expanded * other
    })
}
#[test]
fn test_expansion_compatibility_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    // Test edge cases that might cause LogUp issues

    // Case 1: Zero-sized expansion
    test_expansion_scenario("zero_expansion_edge_case", |cx| {
        let mut rng = StdRng::seed_from_u64(56);
        let a = cx.tensor((1, 4)).set(random_vec_rng(4, &mut rng, false));
        let b = cx.tensor((3, 4)).set(random_vec_rng(12, &mut rng, false));

        // Standard expansion that should work
        let a_exp = a.expand(0, 3);
        a_exp + b
    })?;

    // Case 2: Identity expansion (expand by factor 1)
    test_expansion_scenario("identity_expansion_edge_case", |cx| {
        let mut rng = StdRng::seed_from_u64(57);
        let a = cx.tensor((3, 3)).set(random_vec_rng(9, &mut rng, false));

        // This should be equivalent to no expansion
        let a_exp = a.expand(2, 1);
        let other = cx.tensor((3, 3, 1)).set(random_vec_rng(9, &mut rng, false));
        a_exp + other
    })?;

    Ok(())
}

#[test]
fn test_expansion_integration_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running comprehensive expansion integration test...");

    // This test combines multiple expansion patterns in a single graph
    test_expansion_scenario("comprehensive_integration", |cx| {
        let mut rng = StdRng::seed_from_u64(100);

        // Layer 1: Basic tensors
        let input1 = cx.tensor((2, 3)).set(random_vec_rng(6, &mut rng, false));
        let input2 = cx.tensor((1, 3)).set(random_vec_rng(3, &mut rng, false));
        let input3 = cx.tensor((2, 1)).set(random_vec_rng(2, &mut rng, false));
        let bias = cx.tensor((1, 1)).set(vec![0.1]);

        // Layer 2: First set of expansions and operations
        let input2_exp = input2.expand(0, 2); // (1,3) -> (2,3)
        let input3_exp = input3.expand(1, 3); // (2,1) -> (2,3)
        let bias_exp = bias.expand_to((2, 3)); // (1,1) -> (2,3)

        let intermediate1 = input1 + input2_exp;
        let intermediate2 = intermediate1 * input3_exp;
        let intermediate3 = intermediate2 + bias_exp;

        // Layer 3: More complex operations with the intermediate result
        let intermediate3_sin = intermediate3.sin(); // Unary op preserves shape
        let intermediate3_exp = intermediate3_sin.expand(2, 4); // (2,3) -> (2,3,4)

        let filter = cx.tensor((2, 3, 4)).set(random_vec_rng(24, &mut rng, false));
        let filtered = intermediate3_exp * filter;

        // Layer 4: Reduction and final operations
        let reduced = filtered.sum_reduce(2); // (2,3,4) -> (2,3)
        let final_bias = cx.tensor((1, 1)).set(vec![-0.05]);
        let final_bias_exp = final_bias.expand_to((2, 3));

        reduced + final_bias_exp
    })?;

    println!("✅ All expansion integration scenarios passed!");
    Ok(())
}
