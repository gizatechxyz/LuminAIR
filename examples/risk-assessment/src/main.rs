use luminair::prelude::*;
use std::time::Instant;

use crate::scenario::{scenarios, Scenario};

mod scenario;

/// DeFi Protocol Risk Assessment with ZK Proofs
///
/// This example demonstrates how to use LuminAIR to create verifiable risk calculations
/// for DeFi protocols. It computes:
/// - Value at Risk (VaR): The maximum expected loss at a given confidence level
/// - Conditional Value at Risk (CVaR): The expected loss in tail scenarios
/// - Maximum Loss: The worst-case scenario loss
///
/// All calculations are proven using STARK proofs.
fn main() {
    println!("=== Verfifiable CVaR Risk Assessment ===\n");
    println!("Scenario: Analyzing potential losses from liquidation events");
    println!("during a market downturn and diverse DeFi-specific stressors\n");

    // ======= DeFi Context: Scenario Set =======
    // Positive = loss,
    // Negative = profit;
    // we will sort worst→best before ZK.
    let scenarios: Vec<Scenario> = scenarios();

    // Pull just the numeric losses for the ZK circuit
    let protocol_losses: Vec<f32> = scenarios.iter().map(|s| s.loss_pct).collect();

    let confidence_level: f32 = 0.95; // 95% confidence
    let n_scenarios = protocol_losses.len();

    println!("Protocol analyzed {} market scenarios", n_scenarios);
    println!("Confidence level: {:.0}%\n", confidence_level * 100.0);

    // ======= Risk Calculation Setup =======
    assert!(n_scenarios > 0);
    assert!((0.0..1.0).contains(&confidence_level));

    // Calculate tail size: worst (1 - alpha) of scenarios (at least 1)
    let tail_scenarios = ((1.0 - confidence_level) * n_scenarios as f32).ceil() as usize;
    let tail_scenarios = tail_scenarios.max(1).min(n_scenarios);
    let var_index = tail_scenarios - 1; // VaR is boundary of the tail

    // ======= Build ZK Computation Graph (numeric only) =======
    let mut cx = Graph::new();

    // Define tensors for protocol loss scenarios
    let losses = cx.tensor((n_scenarios,)).set(protocol_losses.clone());
    let indices: Vec<f32> = (0..n_scenarios).map(|i| i as f32).collect();
    let idx = cx.tensor((n_scenarios,)).set(indices);
    let tail_t = cx
        .tensor((n_scenarios,))
        .set(vec![tail_scenarios as f32; n_scenarios]);
    let var_t = cx
        .tensor((n_scenarios,))
        .set(vec![var_index as f32; n_scenarios]);

    // ======= CVaR: Expected Loss in Tail Scenarios =======
    let tail_mask = idx.less_than(tail_t); // 1 for worst scenarios, 0 otherwise
    let tail_losses_sum = (losses * tail_mask).sum_reduce(0);
    let tail_count = tail_mask.sum_reduce(0);
    let cvar = tail_losses_sum * tail_count.recip(); // mean of tail

    // ======= VaR: Threshold Loss Value =======
    // One-hot at var_index using comparison trick
    let var_onehot = idx.less_than(tail_t) - idx.less_than(var_t);
    let var_value = (losses * var_onehot).sum_reduce(0);

    // ======= Additional Metric: Max Loss (index 0 after sorting) =======
    let zero_t = cx.tensor((n_scenarios,)).set(vec![0.0; n_scenarios]);
    let one_t = cx.tensor((n_scenarios,)).set(vec![1.0; n_scenarios]);
    let max_loss_onehot = idx.less_than(one_t) - idx.less_than(zero_t);
    let max_loss = (losses * max_loss_onehot).sum_reduce(0);

    // Prepare output retrievals
    let mut cvar_out = cvar.retrieve();
    let mut var_out = var_value.retrieve();
    let mut max_loss_out = max_loss.retrieve();

    // ======= Generate ZK Proof of Risk Calculations =======
    cx.compile(
        <(GenericCompiler, StwoCompiler)>::default(),
        &mut (&mut cvar_out, &mut var_out, &mut max_loss_out),
    );

    let mut settings = cx.gen_circuit_settings(16);
    let trace = cx.gen_trace(&mut settings).unwrap();
    let t_prove = Instant::now();
    let zk_proof = prove(trace, settings.clone()).unwrap();
    let dt_prove = t_prove.elapsed();

    // Verify the proof
    let t_verify = Instant::now();
    let verify_result = verify(zk_proof, settings);
    let is_verified = verify_result.is_ok();
    if let Err(e) = verify_result {
        println!("Verification error: {:?}", e);
    }
    let dt_verify = t_verify.elapsed();

    // ======= Display Risk Metrics for Protocol Governance =======
    let cvar_result = cvar_out.data()[0];
    let var_result = var_out.data()[0];
    let max_loss_result = max_loss_out.data()[0];

    println!("\n╔════════════════════════════════════════════╗");
    println!("║     DeFi Protocol Risk Assessment Report   ║");
    println!("╚════════════════════════════════════════════╝\n");

    println!(
        "📊 Risk Metrics ({}% Confidence Level):",
        (confidence_level * 100.0) as u32
    );
    println!("────────────────────────────────────────");
    println!("• VaR_{:.2}:  {:.2}%", confidence_level, var_result);
    println!(
        "  → Protocol won't lose more than {:.2}% in {:.0}% of scenarios",
        var_result,
        confidence_level * 100.0
    );

    println!("\n• CVaR_{:.2}: {:.2}%", confidence_level, cvar_result);
    println!(
        "  → Average loss is {:.2}% conditional on tail events",
        cvar_result
    );

    println!("\n• Max Loss:  {:.2}%", max_loss_result);
    println!("  → Worst-case scenario loss (black swan event)");

    // Risk-based recommendations
    println!("\n💡 Protocol Recommendations:");
    println!("────────────────────────────────────────");
    if cvar_result > 15.0 {
        println!("⚠️  CRITICAL: Tail risk too high!");
        println!("   • Increase minimum collateral ratios");
        println!(
            "   • Add insurance fund of at least {:.1}% of TVL",
            cvar_result
        );
    } else if cvar_result > 10.0 {
        println!("⚠️  WARNING: Elevated tail risk");
        println!("   • Consider raising liquidation thresholds");
        println!(
            "   • Increase protocol reserves to {:.1}% of TVL",
            cvar_result * 0.8
        );
        println!("   • Monitor large positions more frequently");
    } else if cvar_result > 5.0 {
        println!("✓  MODERATE: Acceptable risk level");
        println!("   • Maintain reserves at {:.1}% of TVL", cvar_result * 0.6);
        println!("   • Current liquidation parameters adequate");
    } else {
        println!("✅  HEALTHY: Low risk profile");
        println!("   • Protocol well-capitalized");
        println!("   • Consider optimizing capital efficiency");
    }

    // ZK Proof verification status
    println!("\n🔐 ZK Proof Status:");
    println!("────────────────────────────────────────");
    if is_verified {
        println!("✅ Proof verified successfully!");
        println!("• Proving time:         {:.5}s", dt_prove.as_secs_f64());
        println!("• Verification time:    {:.5}s", dt_verify.as_secs_f64());
        println!("   → Risk calculations are mathematically sound");
        println!("   → Can be submitted and verified on-chain for transparency");
    } else {
        println!("❌ Proof verification failed!");
        println!("   → Do not use these metrics for governance decisions");
    }
}
