use luminair::prelude::*;
use ndarray::*;
use std::time::Instant;

/// Physics-Informed Neural Network for Black-Scholes Option Pricing
/// Demonstrates neural network inference with zero-knowledge proof generation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize computational graph
    let mut graph = Graph::new();

    // Load pre-trained PINN weights (3-layer network: 2 -> 64 -> 64 -> 1)
    let weights = load_pinn_weights()?;

    // Build neural network layers
    let network = build_network(&mut graph, weights);

    // Define forward pass computation
    let input = graph.tensor((1, 2));
    let mut output = forward_pass(&network, input);

    // Compile the computational graph
    compile_graph(&mut graph, &mut output);

    // Set Black-Scholes parameters: [Stock price=15.0, Time to expiration=0.5 years]
    input.set(vec![15.0, 0.5]);

    // Generate and verify ZK proof
    let result = generate_and_verify_proof(&mut graph, &mut output)?;

    print_results(&result);

    Ok(())
}

struct PinnWeights {
    layer1_w: Array2<f32>,
    layer1_b: Array1<f32>,
    layer2_w: Array2<f32>,
    layer2_b: Array1<f32>,
    layer3_w: Array2<f32>,
    layer3_b: Array1<f32>,
}

struct Network {
    layer1: Linear,
    layer2: Linear,
    layer3: Linear,
}

fn load_pinn_weights() -> Result<PinnWeights, Box<dyn std::error::Error>> {
    Ok(PinnWeights {
        layer1_w: ndarray_npy::read_npy("model/weights/layer1_weight.npy")?,
        layer1_b: ndarray_npy::read_npy("model/weights/layer1_bias.npy")?,
        layer2_w: ndarray_npy::read_npy("model/weights/layer2_weight.npy")?,
        layer2_b: ndarray_npy::read_npy("model/weights/layer2_bias.npy")?,
        layer3_w: ndarray_npy::read_npy("model/weights/layer3_weight.npy")?,
        layer3_b: ndarray_npy::read_npy("model/weights/layer3_bias.npy")?,
    })
}

fn build_network(graph: &mut Graph, weights: PinnWeights) -> Network {
    // Layer 1: Input (2 features: S, t) -> Hidden (64 neurons)
    let layer1 = Linear::new(2, 64, true, graph);
    layer1
        .weight
        .set(weights.layer1_w.as_slice().unwrap().to_vec());
    if let Some(bias) = &layer1.bias {
        bias.set(weights.layer1_b.as_slice().unwrap().to_vec());
    }

    // Layer 2: Hidden (64 neurons) -> Hidden (64 neurons)
    let layer2 = Linear::new(64, 64, true, graph);
    layer2
        .weight
        .set(weights.layer2_w.as_slice().unwrap().to_vec());
    if let Some(bias) = &layer2.bias {
        bias.set(weights.layer2_b.as_slice().unwrap().to_vec());
    }

    // Layer 3: Hidden (64 neurons) -> Output (1 neuron: option price)
    let layer3 = Linear::new(64, 1, true, graph);
    layer3
        .weight
        .set(weights.layer3_w.as_slice().unwrap().to_vec());
    if let Some(bias) = &layer3.bias {
        bias.set(weights.layer3_b.as_slice().unwrap().to_vec());
    }

    Network {
        layer1,
        layer2,
        layer3,
    }
}

fn forward_pass(network: &Network, input: GraphTensor) -> GraphTensor {
    // Forward pass: input -> layer1 -> tanh -> layer2 -> tanh -> layer3
    let x = network.layer1.forward(input);
    let x = x.tanh();
    let x = network.layer2.forward(x);
    let x = x.tanh();
    network.layer3.forward(x).retrieve()
}

fn compile_graph(graph: &mut Graph, output: &mut GraphTensor) {
    graph.compile(
        (GenericCompiler::default(), StwoCompiler::default()),
        output,
    );
}

fn generate_and_verify_proof(
    graph: &mut Graph,
    output: &mut GraphTensor,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    // Generate circuit settings for ZK proof
    // Note: In real-world applications, circuit settings should be generated once and reused for multiple inferences.
    println!("Generating Circuit Settings...");
    let mut settings = graph.gen_circuit_settings();
    println!("✅ Circuit Settings generated");

    // Generate execution trace
    let timing_start = Instant::now();
    println!("Generating Trace...");
    let trace = graph.gen_trace(&mut settings)?;
    println!("✅ Trace generated in {:?}", timing_start.elapsed());

    // Generate ZK proof
    let timing_start = Instant::now();
    println!("Generating Proof...");
    let proof = prove(trace, settings.clone())?;
    println!("✅ Proof generated in {:?}", timing_start.elapsed());

    // Verify ZK proof
    // Note: In real-world applications, proof verification should be performed by another party.
    let timing_start = Instant::now();
    println!("Verifying Proof...");
    verify(proof, settings)?;
    println!("✅ Proof verified in {:?}", timing_start.elapsed());

    Ok(output.data())
}

fn print_results(result: &[f32]) {
    println!("Black-Scholes PINN Results:");
    println!("Input: Stock Price = $15.00, Time = 0.5 years");
    println!("Predicted Option Price: ${:.6}", result[0]);
}
