# Risk Assessment Example

A verifiable DeFi protocol risk assessment using Zero-Knowledge proofs to calculate Value at Risk (VaR) and Conditional Value at Risk (CVaR) metrics.

## Overview

This example demonstrates how DeFi lending protocols can use LuminAIR to:
- Generate ZK proofs of risk calculations  
- Enable transparent and verifiable risk management

The implementation analyzes potential losses from liquidation events during market downturns and DeFi-specific stress scenarios.

## Risk Metrics Calculated

- **VaR (Value at Risk)**: Maximum expected loss at a given confidence level (95%)
- **CVaR (Conditional Value at Risk)**: Average loss in tail scenarios (worst-case outcomes)  
- **Max Loss**: Worst-case scenario loss (black swan events)

## Scenario Analysis

The example includes 40+ realistic DeFi risk scenarios categorized by:

- **Ultra-Crisis/Systemic**: Bridge exploits, stablecoin depegs, oracle failures
- **Market Events**: Liquidation cascades, volatility spikes, whale positions
- **Infrastructure**: Sequencer outages, gas spikes, keeper failures
- **Operational**: MEV impacts, auction mechanics, cross-chain delays

Loss percentages range from catastrophic events (24% TVL loss) to profitable scenarios (-5.2% gain).

## Zero-Knowledge Proof

The risk calculations are performed within a ZK circuit using LuminAIR, providing:
- Mathematical soundness verification
- Transparent risk assessment

## Usage

Run the risk assessment:

```bash
cargo run
```

## Output

The program provides:
- Detailed risk metrics at 95% confidence level
- Risk-based protocol recommendations
- ZK proof generation and verification status
- Performance timing for proving and verification

## Files

- `src/main.rs`: Main risk assessment logic and ZK circuit implementation
- `src/scenario.rs`: DeFi risk scenario definitions and data

## Dependencies

- `luminair`: LuminAIR framework for ZK computations
- `stwo-prover`: STARK proof system