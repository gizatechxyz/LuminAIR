use luminair_graph::{graph::LuminairGraph, StwoCompiler};
use luminal::prelude::*;

// =============== Example Overview ===============
// This example demonstrates how to:
// 1. Build a computation graph with tensor operations.
// 2. Compile and execute the graph, generating execution traces.
// 3. Generate a CStark proof of the computational graph.
// 4. Verify the proof to ensure computation integrity.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // =============== Step 1: Building the Computation Graph ===============
    // Create a new computation graph to hold tensor operations.
    let mut cx = Graph::new();

    // Define three 2x2 tensors with sample data.
    // In a real-world scenario, these could be input features, weights, etc.
    let a = cx.tensor((2, 2)).set(vec![1.0, 2.0, 3.0, 4.0]);
    let b = cx.tensor((2, 2)).set(vec![10.0, 20.0, 30.0, 40.0]);
    let w = cx.tensor((2, 2)).set(vec![-1.0, -1.0, -1.0, -1.0]);

    // Define computation operations on tensors:
    let c = a * b;
    let d = c + w;
    let mut e = (c + d).retrieve();

    // At this point, no computation has occurred. We have just defined the computation graph.
    // The operations will be executed only when the trace is generated.

    // =============== Step 2: Compilation & Execution ===============
    // Compile the computation graph to transform the operations and prepare for execution.
    println!("Compiling computation graph...");
    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut e);
    println!("Graph compiled successfully. ✅");

    // Optionally, visualize the computation graph (useful for debugging or optimization).
    // cx.display();

    // Execute and generate a trace of the computation graph.
    // This is when the actual computation happens.
    println!("Executing graph and generating execution trace...");
    let trace = cx.gen_trace();
    println!("Execution trace generated successfully. ✅");

    // Retrieve and display the final result.
    let result = cx.get_output(e.id);
    println!("Final result: {:?}", result);

    // =============== Step 3: Generating Proof & Verification ===============
    // Generate a CStark proof for the trace.
    println!("Generating proof for execution trace...");
    let proof = cx.prove(trace)?;
    println!("Proof generated successfully. ✅");

    // Verify the generated proof to ensure the integrity of the computation.
    println!("Verifying proof...");
    cx.verify(proof)?;
    println!("Proof verified successfully. Computation integrity ensured. 🎉");

    Ok(())
}
