use super::{assert_close, random_vec_rng};
use crate::graph::LuminairGraph;
use crate::StwoCompiler;
use crate::{binary_test, unary_test};
use luminal::prelude::*;
use luminal_cpu::CPUCompiler;
use rand::{rngs::StdRng, SeedableRng};

// =============== UNARY ===============
unary_test!(|a| a.recip(), test_recip, f32, true);

// =============== BINARY ===============

fn sum_reduce_wrapper_axes_0(a: GraphTensor, _b: GraphTensor) -> GraphTensor {
    a.sum_reduce(0) 
}

fn sum_reduce_wrapper_axes_1(a: GraphTensor, _b: GraphTensor) -> GraphTensor {
    a.sum_reduce(0) 
}

// Use binary_test with the wrapper function
binary_test!(sum_reduce_wrapper_axes_0, test_sum_reduce_0, f32, false);
binary_test!(sum_reduce_wrapper_axes_1, test_sum_reduce_1, f32, false);


#[test]
fn test_sum_reduce() {
    let mut cx = Graph::new();
    let a = cx.tensor((3, 2)).set([[1., 2.], [3., 2.], [3., 1.]]);
    let mut c = a.sum_reduce(0).retrieve();

    println!("C ID: {:?}\n", c.id);
    println!("C graph_ref: {:?\n}", c.graph_ref);
    println!("C shape: {:?}\n", c.shape);
    // cx.execute();

    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut c);
    let trace = cx.gen_trace().expect("Trace generation failed");
    // println!("trace: {:?}\n", trace);

    let proof = cx.prove(trace).expect("Proof generation failed");
    // println!("proof: {:?}\n", proof);

    cx.verify(proof).expect("Proof verification failed");
    // // Retrieve output data
    let stwo_output = c.data();
    println!("stwo_output: {:?}", stwo_output);


    let mut cx_cpu = Graph::new();
    // let d_a = cx_cpu.tensor(3).set([1., 2., 3.]);
    let d_a = cx_cpu.tensor((3, 2)).set([[1., 2.], [3., 2.], [3., 1.]]);
    let mut c_cpu = d_a.sum_reduce(1).retrieve();

    cx_cpu.compile(<(GenericCompiler, CPUCompiler)>::default(), &mut c_cpu);
    cx_cpu.execute();
    // Retrieve CPU output
    let cpu_output = c_cpu.data();
    println!("cpu_output: {:?}", cpu_output);


    // assert_close(&stwo_output, &cpu_output);
}

binary_test!(|a, b| a + b, test_add, f32, false);
binary_test!(|a, b| a * b, test_mul, f32, false);
