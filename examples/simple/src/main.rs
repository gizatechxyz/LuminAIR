use luminair::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cx = Graph::new();

    // ======= Define initializers =======
    let a = cx.tensor((2, 2)).set(vec![1.0, 2.0, 3.0, 4.0]);
    let b = cx.tensor((2, 2)).set(vec![10.0, 20.0, 30.0, 40.0]);
    let w = cx.tensor((2, 2)).set(vec![-1.0, -1.0, -1.0, -1.0]);

    // ======= Define graph =======
    let c = a * b;
    let d = c + w;
    let mut e = (c * d).retrieve();

    // ======= Compile graph =======
    println!("Compiling computation graph...");
    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut e);
    println!("Graph compiled successfully. ✅");

    // ======= Generate circuit settings =======
    println!("Generating circuits settings...");
    let mut settings = cx.gen_circuit_settings();
    println!("Settings generated successfully. ✅");

    // ======= Execute graph & generate trace =======
    println!("Executing graph and generating execution trace...");
    let trace = cx.gen_trace(&mut settings)?;
    println!("Execution trace generated successfully. ✅");
    println!("Final result: {:?}", e);

    // ======= Prove & Verify =======
    println!("Generating proof for execution trace...");
    let proof = prove(trace, settings.clone())?;
    println!("Proof generated successfully. ✅");

    settings.to_bincode_file("./settings.bin")?;
    proof.to_bincode_file("./proof.bin")?;

    println!("Verifying proof...");
    verify(proof, settings)?;
    println!("Proof verified successfully. Computation integrity ensured. 🎉");

    Ok(())
}
