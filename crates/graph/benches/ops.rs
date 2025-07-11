use criterion::{criterion_group, criterion_main, Criterion, PlotConfiguration};
use luminair_graph::{graph::LuminairGraph, StwoCompiler};
use luminair_prover::prover::prove;
use luminair_verifier::verifier::verify;
use luminal::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fmt;

fn random_vec_rng<R: Rng>(n: usize, rng: &mut R, nonzero: bool) -> Vec<f32> {
    (0..n)
        .map(|_| {
            let mut value = rng.gen_range(-0.5..0.5);
            if nonzero {
                while value < 0.001 {
                    value = rng.gen_range(-0.5..0.5);
                }
            }
            value
        })
        .collect()
}

macro_rules! create_binary {
    ($func:expr, ($a_rows:expr, $a_cols:expr), ($b_rows:expr, $b_cols:expr), $nonzero:expr) => {{
        let mut rng = StdRng::seed_from_u64(42);
        let a_data = random_vec_rng($a_rows * $a_cols, &mut rng, $nonzero);
        let b_data = random_vec_rng($b_rows * $b_cols, &mut rng, $nonzero);

        // Graph setup
        let mut cx = Graph::new();
        let a = cx.tensor(($a_rows, $a_cols)).set(a_data.clone());
        let b = cx.tensor(($b_rows, $b_cols)).set(b_data.clone());
        let f = $func;
        let mut c = f(a, b).retrieve();

        // Compilation and execution using StwoCompiler
        cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut c);

        cx
    }};
}

macro_rules! create_unary {
    ($func:expr, ($a_rows:expr, $a_cols:expr), $nonzero:expr) => {{
        let mut rng = StdRng::seed_from_u64(42);
        let a_data = random_vec_rng($a_rows * $a_cols, &mut rng, $nonzero);

        // Graph setup
        let mut cx = Graph::new();
        let a = cx.tensor(($a_rows, $a_cols)).set(a_data.clone());
        let f = $func;
        let mut c = f(a).retrieve();

        // Compilation and execution using StwoCompiler
        cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut c);

        cx
    }};
}

// Define a benchmark parameter that combines operation and tensor size
#[derive(Debug, Clone, Copy)]
enum Stage {
    TraceGeneration,
    Proving,
    Verification,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stage::TraceGeneration => write!(f, "Trace Generation"),
            Stage::Proving => write!(f, "Proving"),
            Stage::Verification => write!(f, "Verification"),
        }
    }
}

struct BenchParams {
    stage: Stage,
    size: (usize, usize),
}

impl fmt::Display for BenchParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({}x{})", self.stage, self.size.0, self.size.1)
    }
}

// Benchmark for Add operator
fn benchmark_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("Add Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(|a, b| a + b, (rows, cols), (rows, cols), false);
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(|a, b| a + b, (rows, cols), (rows, cols), false);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(|a, b| a + b, (rows, cols), (rows, cols), false);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

// Benchmark for Mul operator
fn benchmark_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mul Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(|a, b| a * b, (rows, cols), (rows, cols), false);
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(|a, b| a * b, (rows, cols), (rows, cols), false);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(|a, b| a * b, (rows, cols), (rows, cols), false);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

// Benchmark for Recip operator
fn benchmark_recip(c: &mut Criterion) {
    let mut group = c.benchmark_group("Recip Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.recip(), (rows, cols), true);
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.recip(), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.recip(), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

// Benchmark for Sum Reduce operator
fn benchmark_sum_reduce(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sum Reduce Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph =
                        create_unary!(|a: GraphTensor| a.sum_reduce(0), (rows, cols), true);
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph =
                        create_unary!(|a: GraphTensor| a.sum_reduce(0), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph =
                        create_unary!(|a: GraphTensor| a.sum_reduce(0), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

// Benchmark for Max Reduce operator
fn benchmark_max_reduce(c: &mut Criterion) {
    let mut group = c.benchmark_group("Max Reduce Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph =
                        create_unary!(|a: GraphTensor| a.max_reduce(0), (rows, cols), true);
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph =
                        create_unary!(|a: GraphTensor| a.max_reduce(0), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph =
                        create_unary!(|a: GraphTensor| a.max_reduce(0), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

// Benchmark for Sin operator
fn benchmark_sin(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sin Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.sin(), (rows, cols), true);
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.sin(), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.sin(), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

// Benchmark for Sqrt operator
fn benchmark_sqrt(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sqrt Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.sqrt(), (rows, cols), true);
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.sqrt(), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.sqrt(), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

// Benchmark for Exp2 operator
fn benchmark_exp2(c: &mut Criterion) {
    let mut group = c.benchmark_group("Exp2 Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.exp2(), (rows, cols), true);
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.exp2(), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_unary!(|a: GraphTensor| a.exp2(), (rows, cols), true);
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

// Benchmark for LessThan operator
fn benchmark_less_than(c: &mut Criterion) {
    let mut group = c.benchmark_group("LessThan Operator");
    group
        .plot_config(PlotConfiguration::default().summary_scale(criterion::AxisScale::Logarithmic));

    let sizes = [(32, 32)];

    for &size in &sizes {
        let (rows, cols) = size;

        // Trace generation
        let params = BenchParams {
            stage: Stage::TraceGeneration,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(
                        |a: GraphTensor, b: GraphTensor| a.less_than(b),
                        (rows, cols),
                        (rows, cols),
                        false
                    );
                    let settings = graph.gen_circuit_settings();
                    (graph, settings)
                },
                |(mut graph, mut settings)| {
                    let _trace = graph.gen_trace(&mut settings);
                },
            )
        });

        // Proof generation
        let params = BenchParams {
            stage: Stage::Proving,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(
                        |a: GraphTensor, b: GraphTensor| a.less_than(b),
                        (rows, cols),
                        (rows, cols),
                        false
                    );
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    (settings, trace)
                },
                |(settings, trace)| {
                    let _proof = prove(trace, settings).expect("Proof generation failed");
                },
            )
        });

        // Verification
        let params = BenchParams {
            stage: Stage::Verification,
            size,
        };
        group.bench_function(params.to_string(), |b| {
            b.iter_with_setup(
                || {
                    let mut graph = create_binary!(
                        |a: GraphTensor, b: GraphTensor| a.less_than(b),
                        (rows, cols),
                        (rows, cols),
                        false
                    );
                    let mut settings = graph.gen_circuit_settings();
                    let trace = graph
                        .gen_trace(&mut settings)
                        .expect("Trace generation failed");
                    let proof = prove(trace, settings.clone()).expect("Proof generation failed");
                    (settings, proof)
                },
                |(settings, proof)| {
                    verify(proof, settings).expect("Proof verification failed");
                },
            )
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_add,
    benchmark_mul,
    benchmark_recip,
    benchmark_sum_reduce,
    benchmark_max_reduce,
    benchmark_sin,
    benchmark_sqrt,
    benchmark_exp2,
    benchmark_less_than
);
criterion_main!(benches);
