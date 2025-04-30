use luminal::{graph::Graph, op::Operator};
use rand::Rng;

mod ops;

#[macro_export]
macro_rules! single_unary_test {
    ($func:expr, $name:ident, $type:ty, ($rows:expr, $cols:expr), $nonzero:expr) => {
        paste::paste! {
            #[test]
            fn [<$name _ $rows x $cols>]() {
                let mut rng = StdRng::seed_from_u64(42);
                let a_data = random_vec_rng($rows * $cols, &mut rng, $nonzero);

                // Graph setup for Stwo compiler
                let mut cx = Graph::new();
                let a = cx.tensor(($rows, $cols)).set(a_data.clone());

                let f: fn(GraphTensor) -> GraphTensor = $func;
                let mut c = f(a).retrieve();

                // Compilation and execution using StwoCompiler
                cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut c);

                let settings = cx.gen_circuit_settings();
                c.drop();
                let trace = cx.gen_trace(settings.clone()).expect("Trace generation failed");
                let proof = cx.prove(trace, settings.clone()).expect("Proof generation failed");
                cx.verify(proof, settings.clone()).expect("Proof verification failed");
                // Retrieve output data
                let stwo_output = c.data();

                // CPUCompiler comparison
                let mut cx_cpu = Graph::new();
                let a_cpu = cx_cpu.tensor(($rows, $cols)).set(a_data);
                let mut c_cpu = f(a_cpu).retrieve();
                cx_cpu.compile(<(GenericCompiler, CPUCompiler)>::default(), &mut c_cpu);
                cx_cpu.execute();
                // Retrieve CPU output
                let cpu_output = c_cpu.data();

                // Assert outputs are close
                assert_close(&stwo_output, &cpu_output);
            }
        }
    };
}

#[macro_export]
macro_rules! unary_test {
    ($func: expr, $name: ident, $type: ty,  $nonzero: expr) => {
        // Test with 2D tensor
        $crate::single_unary_test!($func, $name, $type, (3, 4), $nonzero);
        // Test with a scalar
        $crate::single_unary_test!($func, $name, $type, (1, 1), $nonzero);
        // Test with a row vector
        $crate::single_unary_test!($func, $name, $type, (1, 8), $nonzero);
        // Test with a column vector
        $crate::single_unary_test!($func, $name, $type, (8, 1), $nonzero);
    };
}

#[macro_export]
macro_rules! single_binary_test {
    ($func:expr, $name:ident, $type:ty, ($a_rows:expr, $a_cols:expr), ($b_rows:expr, $b_cols:expr), $nonzero:expr) => {
        paste::paste! {
            #[test]
            fn [<$name _ $a_rows x $a_cols _ $b_rows x $b_cols>]() {
                let mut rng = StdRng::seed_from_u64(42);
                let a_data = random_vec_rng($a_rows * $a_cols, &mut rng, $nonzero);
                let b_data = random_vec_rng($b_rows * $b_cols, &mut rng, $nonzero);

                // Graph setup
                let mut cx = Graph::new();
                let a = cx.tensor(($a_rows, $a_cols)).set(a_data.clone());
                let b = cx.tensor(($b_rows, $b_cols)).set(b_data.clone());

                // Use expand when dimensions don't match
                let a_expanded = if $a_rows != $b_rows || $a_cols != $b_cols {
                    if $a_rows == 1 && $a_cols == 1 {
                        // Scalar broadcasting to match b's shape
                        a.expand_to(($b_rows, $b_cols))
                    } else if $a_rows == 1 {
                        // Row vector broadcasting
                        a.expand(0, $b_rows)
                    } else if $a_cols == 1 {
                        // Column vector broadcasting
                        a.expand(1, $b_cols)
                    } else {
                        a
                    }
                } else {
                    a
                };

                let b_expanded = if $a_rows != $b_rows || $a_cols != $b_cols {
                    if $b_rows == 1 && $b_cols == 1 {
                        // Scalar broadcasting to match a's shape
                        b.expand_to(($a_rows, $a_cols))
                    } else if $b_rows == 1 {
                        // Row vector broadcasting
                        b.expand(0, $a_rows)
                    } else if $b_cols == 1 {
                        // Column vector broadcasting
                        b.expand(1, $a_cols)
                    } else {
                        b
                    }
                } else {
                    b
                };

                let f: fn(GraphTensor, GraphTensor) -> GraphTensor = $func;
                let mut c = f(a_expanded, b_expanded).retrieve();

                // Compilation and execution using StwoCompiler
                cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut c);
                let settings = cx.gen_circuit_settings();
                c.drop();
                let trace = cx.gen_trace(settings.clone()).expect("Trace generation failed");
                let proof = cx.prove(trace, settings.clone()).expect("Proof generation failed");
                cx.verify(proof, settings).expect("Proof verification failed");
                // Retrieve output data
                let stwo_output = c.data();

                // CPUCompiler comparison
                let mut cx_cpu = Graph::new();
                let a_cpu = cx_cpu.tensor(($a_rows, $a_cols)).set(a_data);
                let b_cpu = cx_cpu.tensor(($b_rows, $b_cols)).set(b_data);

                // Apply the same broadcasting logic to the CPU test
                let a_cpu_expanded = if $a_rows != $b_rows || $a_cols != $b_cols {
                    if $a_rows == 1 && $a_cols == 1 {
                        a_cpu.expand_to(($b_rows, $b_cols))
                    } else if $a_rows == 1 {
                        a_cpu.expand(0, $b_rows)
                    } else if $a_cols == 1 {
                        a_cpu.expand(1, $b_cols)
                    } else {
                        a_cpu
                    }
                } else {
                    a_cpu
                };

                let b_cpu_expanded = if $a_rows != $b_rows || $a_cols != $b_cols {
                    if $b_rows == 1 && $b_cols == 1 {
                        b_cpu.expand_to(($a_rows, $a_cols))
                    } else if $b_rows == 1 {
                        b_cpu.expand(0, $a_rows)
                    } else if $b_cols == 1 {
                        b_cpu.expand(1, $a_cols)
                    } else {
                        b_cpu
                    }
                } else {
                    b_cpu
                };

                let mut c_cpu = f(a_cpu_expanded, b_cpu_expanded).retrieve();
                cx_cpu.compile(<(GenericCompiler, CPUCompiler)>::default(), &mut c_cpu);
                cx_cpu.execute();
                // Retrieve CPU output
                let cpu_output = c_cpu.data();

                // Assert outputs are close
                assert_close(&stwo_output, &cpu_output);
            }
        }
    };
}

#[macro_export]
macro_rules! binary_test {
    ($func: expr, $name: ident, $type: ty, $nonzero: expr) => {
        // Test operation with same-sized tensors
        $crate::single_binary_test!($func, $name, $type, (3, 4), (3, 4), $nonzero);
        // Test with large tensors to ensure scalability
        $crate::single_binary_test!($func, $name, $type, (32, 32), (32, 32), $nonzero);
        // Test with tensors that have uneven dimensions
        $crate::single_binary_test!($func, $name, $type, (17, 13), (17, 13), $nonzero);
        // Test broadcasting a scalar (1,1) to a larger tensor
        $crate::single_binary_test!($func, $name, $type, (1, 1), (5, 5), $nonzero);
        // Test broadcasting a row vector to a matrix
        $crate::single_binary_test!($func, $name, $type, (1, 4), (3, 4), $nonzero);
        // Test broadcasting a column vector to a matrix
        $crate::single_binary_test!($func, $name, $type, (3, 1), (3, 4), $nonzero);
    };
}

#[allow(dead_code)]
pub fn assert_op_in_graph<T: Operator + 'static>(graph: &Graph) {
    assert!(
        graph.node_indices().any(|i| graph.check_node_type::<T>(i)),
        "Node not found in the graph!"
    );
}

pub fn random_vec_rng<R: Rng>(n: usize, rng: &mut R, nonzero: bool) -> Vec<f32> {
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

/// Ensure two arrays are nearly equal
pub fn assert_close(a_vec: &[f32], b_vec: &[f32]) {
    assert_close_precision(a_vec, b_vec, 1e-3);
}

/// Ensure two arrays are nearly equal to a decimal place
pub fn assert_close_precision(a_vec: &[f32], b_vec: &[f32], threshold: f32) {
    assert_eq!(a_vec.len(), b_vec.len(), "Number of elements doesn't match");
    for (i, (a, b)) in a_vec.iter().zip(b_vec.iter()).enumerate() {
        if (a - b).abs() > threshold {
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
