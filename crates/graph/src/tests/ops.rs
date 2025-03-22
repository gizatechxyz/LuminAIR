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

// =============== REDUCTION ===============
#[test]
fn test_max_reduce() {
    // Create a simple tensor with known values for easy testing
    let data = vec![1.0, 2.0, 3.0, 0.5];
    
    // Graph setup with Stwo compiler
    let mut cx = Graph::new();
    let a = cx.tensor(4).set(data.clone());
    
    // Apply max_reduce on the tensor - should give a single value (the max)
    let mut b = a.max_reduce(0).retrieve();
    
    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut b);
    let trace = cx.gen_trace().expect("Trace generation failed");
    let proof = cx.prove(trace).expect("Proof generation failed");
    cx.verify(proof).expect("Proof verification failed");
    
    // Get the output
    let stwo_output = b.data();
    
    // Now do the same with CPU compiler for comparison
    let mut cx_cpu = Graph::new();
    let a_cpu = cx_cpu.tensor(4).set(data);
    let mut b_cpu = a_cpu.max_reduce(0).retrieve();
    
    cx_cpu.compile(<(GenericCompiler, CPUCompiler)>::default(), &mut b_cpu);
    cx_cpu.execute();
    
    // Get CPU output
    let cpu_output = b_cpu.data();
    
    // Assert outputs are close
    assert_close(&stwo_output, &cpu_output);
    
    // Verify the result is actually 3.0 (the max value)
    assert_eq!(stwo_output[0], 3.0);
}
