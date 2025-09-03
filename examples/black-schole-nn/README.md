# Black-Scholes zkML with LuminAIR

A zkML implementation that uses Physics-Informed Neural Networks (PINNs) to solve the Black-Scholes partial differential equation for option pricing, with cryptographic proofs generated via LuminAIR and S-two prover.

PINNs are neural networks that are trained not just on data, but also on the **underlying physical (or financial) laws** described by differential equations.

## Overview

This project demonstrates how to:

1. Train a PINN model in PyTorch to solve the Black-Scholes PDE
2. Export trained weights for use with LuminAIR's zkML framework
3. Generate cryptographic proofs that neural network inference was computed correctly
4. Verify proofs to ensure computational integrity without re-execution

## Acknowledgments

This project builds upon [BlackScholesPINN](https://github.com/PieroPaialungaAI/BlackScholesPINN) by Piero Paialunga.
The original repository provides the training code for the Physics-Informed Neural Network used to approximate the Black-Scholes PDE.

The purpose of this example is to extend that work by demonstrating how `BlackScholesPINN` can be integrated with the LuminAIR zkML framework to generate verifiable zk proofs for option pricing computations usefull for DeFi protocols. 

## Project Structure

```
.
├── Makefile              # Build automation
├── Cargo.toml           # Rust dependencies for zkML
├── pyproject.toml       # Python project configuration
├── src/
│   └── main.rs          # zkML inference with LuminAIR + Stwo proofs
└── model/
    ├── config.json      # Black-Scholes parameters (K=20, σ=0.25, r=0.05)
    ├── main.py          # PINN training pipeline
    ├── model.py         # Neural network architecture (2→64→64→1)
    ├── black_scholes.py # Physics-Informed Neural Network implementation
    ├── export_weights.py # Weight export for zkML
    └── ...             # Additional training modules
```

## Quick Start

### 1. Setup Environment

```bash
make setup
```

### 2. Train the PINN Model

```bash
make train
```

This trains the PINN to satisfy the Black-Scholes PDE:

```
∂V/∂t + (1/2)σ²S²∂²V/∂S² + rS∂V/∂S - rV = 0
```

### 3. Generate zkML Proofs

```bash
make predict
```

This will:

- Load exported neural network weights
- Compile the computational graph for zero-knowledge circuits
- Generate a cryptographic proof of correct option price calculation
- Verify the proof using Circle STARK verification

## zkML Technical Details

### Neural Network Architecture

- **Input**: Stock price (S) and time to expiration (t)
- **Architecture**: 2 → 64 → 64 → 1 (fully connected layers with tanh activation)
- **Output**: Option price V(S,t)

### Zero-Knowledge Proof Generation

1. **Circuit Compilation**: Neural network operations compiled to arithmetic circuits
2. **Trace Generation**: Execution trace of forward pass computation
3. **Proof Generation**: Circle STARK proof demonstrating correct computation
4. **Verification**: Cryptographic verification without re-executing the network

### Example Output

```
Input: Stock Price = $15.00, Time = 0.5 years
Predicted Option Price: $2.847291
✅ Proof generated and verified
```

## Black-Scholes Parameters

Default configuration in `model/config.json`:

- Strike price (K): $20
- Risk-free rate (r): 5%
- Volatility (σ): 25%
- Time to expiration (T): 1 year

## Full Workflow

```bash
make all  # setup → train → zkML proof generation
```

## Cleaning Up

```bash
make clean
```
