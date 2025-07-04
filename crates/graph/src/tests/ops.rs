use super::{assert_close, random_vec_rng};
use crate::graph::LuminairGraph;
use crate::StwoCompiler;
use crate::{binary_test, unary_test};
use luminair_prover::prover::prove;
use luminair_verifier::verifier::verify;
use luminal::prelude::*;
use luminal_cpu::CPUCompiler;
use rand::{rngs::StdRng, SeedableRng};

// The tests are inspired by Luminal's CUDA tests:
// https://github.com/raphaelDkhn/luminal/blob/main/crates/luminal_cuda/src/tests/fp32.rs

// =============== UNARY ===============
// unary_test!(|a| a.recip(), test_recip, f32, true);
unary_test!(|a| a.sin(), test_sin, f32, true);
unary_test!(|a| a.sqrt(), test_sqrt, f32, true);
unary_test!(|a| a.exp2(), test_exp2, f32, true);

// =============== BINARY ===============

binary_test!(|a, b| a + b, test_add, f32, false);
binary_test!(|a, b| a * b, test_mul, f32, false);

// =============== REDUCE ===============

#[test]
fn test_sum_reduce() {
    // Graph setup
    let mut cx = Graph::new();
    let mut rng = StdRng::seed_from_u64(1);
    let data = random_vec_rng(4 * 100, &mut rng, false);
    let a = cx.tensor((1, 4, 100));
    a.set(data.clone());
    let mut b = a.sum_reduce(1).retrieve();
    let mut c = a.sum_reduce(0).retrieve();
    let mut d = a.sum_reduce(2).retrieve();

    // Compilation and execution using StwoCompiler
    cx.compile(
        <(GenericCompiler, StwoCompiler)>::default(),
        (&mut b, &mut c, &mut d),
    );

    let mut settings = cx.gen_circuit_settings();
    b.drop();
    c.drop();
    d.drop();
    let trace = cx
        .gen_trace(&mut settings)
        .expect("Trace generation failed");
    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
    verify(proof, settings).expect("Proof verification failed");

    // CPUCompiler comparison
    let mut cx_cpu = Graph::new();
    let a_cpu = cx_cpu.tensor((1, 4, 100)).set(data.clone());
    let mut b_cpu = a_cpu.sum_reduce(1).retrieve();
    let mut c_cpu = a_cpu.sum_reduce(0).retrieve();
    let mut d_cpu = a_cpu.sum_reduce(2).retrieve();
    cx_cpu.compile(
        <(GenericCompiler, CPUCompiler)>::default(),
        (&mut b_cpu, &mut c_cpu, &mut d_cpu),
    );
    cx_cpu.execute();

    // Assert outputs are close
    assert_close(&b.data(), &b_cpu.data());
    assert_close(&c.data(), &c_cpu.data());
    assert_close(&d.data(), &d_cpu.data());
}

#[test]
fn test_max_reduce() {
    // Graph setup
    let mut cx = Graph::new();
    let mut rng = StdRng::seed_from_u64(1);
    let data = random_vec_rng(4 * 100, &mut rng, false);
    let a = cx.tensor((1, 4, 100));
    a.set(data.clone());
    let mut b = a.max_reduce(1).retrieve();
    let mut c = a.max_reduce(0).retrieve();
    let mut d = a.max_reduce(2).retrieve();

    // Compilation and execution using StwoCompiler
    cx.compile(
        <(GenericCompiler, StwoCompiler)>::default(),
        (&mut b, &mut c, &mut d),
    );
    let mut settings = cx.gen_circuit_settings();
    b.drop();
    c.drop();
    d.drop();
    let trace = cx
        .gen_trace(&mut settings)
        .expect("Trace generation failed");
    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
    verify(proof, settings).expect("Proof verification failed");

    // CPUCompiler comparison
    let mut cx_cpu = Graph::new();
    let a_cpu = cx_cpu.tensor((1, 4, 100)).set(data.clone());
    let mut b_cpu = a_cpu.max_reduce(1).retrieve();
    let mut c_cpu = a_cpu.max_reduce(0).retrieve();
    let mut d_cpu = a_cpu.max_reduce(2).retrieve();
    cx_cpu.compile(
        <(GenericCompiler, CPUCompiler)>::default(),
        (&mut b_cpu, &mut c_cpu, &mut d_cpu),
    );
    cx_cpu.execute();

    // Assert outputs are close
    assert_close(&b.data(), &b_cpu.data());
    assert_close(&c.data(), &c_cpu.data());
    assert_close(&d.data(), &d_cpu.data());
}

#[test]
fn test_less_than_32x32_32x32() {
    // Graph setup
    let mut cx = Graph::new();
    let mut rng = StdRng::seed_from_u64(1);
    let a_data = random_vec_rng(4 * 4, &mut rng, false);
    let b_data = random_vec_rng(4 * 4, &mut rng, false);
    let a = cx.tensor((4, 4));
    let b = cx.tensor((4, 4));
    a.set(a_data.clone());
    b.set(b_data.clone());
    let mut c = a.less_than(b).retrieve();

    // Compilation and execution using StwoCompiler
    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut c);
    let mut settings = cx.gen_circuit_settings();
    c.drop();
    let trace = cx
        .gen_trace(&mut settings)
        .expect("Trace generation failed");
    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
    verify(proof, settings).expect("Proof verification failed");

    // CPUCompiler comparison
    let mut cx_cpu = Graph::new();
    let a_cpu = cx_cpu.tensor((4, 4)).set(a_data.clone());
    let b_cpu = cx_cpu.tensor((4, 4)).set(b_data.clone());
    let mut c_cpu = a_cpu.less_than(b_cpu).retrieve();
    cx_cpu.compile(<(GenericCompiler, CPUCompiler)>::default(), &mut c_cpu);
    cx_cpu.execute();

    // Assert outputs are close
    assert_close(&c.data(), &c_cpu.data());
}

#[test]
fn test_less_than_17x3_17x3() {
    // Graph setup
    let mut cx = Graph::new();
    let mut rng = StdRng::seed_from_u64(1);
    let a_data = random_vec_rng(17 * 3, &mut rng, false);
    let b_data = random_vec_rng(17 * 3, &mut rng, false);
    let a = cx.tensor((17, 3));
    let b = cx.tensor((17, 3));
    a.set(a_data.clone());
    b.set(b_data.clone());
    let mut c = a.less_than(b).retrieve();

    // Compilation and execution using StwoCompiler
    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut c);
    let mut settings = cx.gen_circuit_settings();
    c.drop();
    let trace = cx
        .gen_trace(&mut settings)
        .expect("Trace generation failed");
    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
    verify(proof, settings).expect("Proof verification failed");

    // CPUCompiler comparison
    let mut cx_cpu = Graph::new();
    let a_cpu = cx_cpu.tensor((17, 3)).set(a_data.clone());
    let b_cpu = cx_cpu.tensor((17, 3)).set(b_data.clone());
    let mut c_cpu = a_cpu.less_than(b_cpu).retrieve();
    cx_cpu.compile(<(GenericCompiler, CPUCompiler)>::default(), &mut c_cpu);
    cx_cpu.execute();

    // Assert outputs are close
    assert_close(&c.data(), &c_cpu.data());
}

#[test]
fn test_less_than_3x4_3x4() {
    // Graph setup
    let mut cx = Graph::new();
    let mut rng = StdRng::seed_from_u64(1);
    let a_data = random_vec_rng(3 * 4, &mut rng, false);
    let b_data = random_vec_rng(3 * 4, &mut rng, false);
    let a = cx.tensor((3, 4));
    let b = cx.tensor((3, 4));
    a.set(a_data.clone());
    b.set(b_data.clone());
    let mut c = a.less_than(b).retrieve();

    // Compilation and execution using StwoCompiler
    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut c);
    let mut settings = cx.gen_circuit_settings();
    c.drop();
    let trace = cx
        .gen_trace(&mut settings)
        .expect("Trace generation failed");
    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
    verify(proof, settings).expect("Proof verification failed");

    // CPUCompiler comparison
    let mut cx_cpu = Graph::new();
    let a_cpu = cx_cpu.tensor((3, 4)).set(a_data.clone());
    let b_cpu = cx_cpu.tensor((3, 4)).set(b_data.clone());
    let mut c_cpu = a_cpu.less_than(b_cpu).retrieve();
    cx_cpu.compile(<(GenericCompiler, CPUCompiler)>::default(), &mut c_cpu);
    cx_cpu.execute();

    // Assert outputs are close
    assert_close(&c.data(), &c_cpu.data());
}
