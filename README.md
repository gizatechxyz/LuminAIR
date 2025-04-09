# LuminAIR

<div align="center">
  <img src="docs/images/gh-banner.png" style="width: 70%; height: auto;">
  </br>
</div>

<div align="center">
  <h3>
    <a href="https://luminair.gizatech.xyz/welcome">
      Documentation
    </a>
    <span> | </span>
    <a href="https://luminair.gizatech.xyz/more/roadmap">
      Roadmap
    </a>
    <span> | </span>
    <a href="https://luminair.gizatech.xyz/more/benchmarks">
      Benchmarks
    </a>
  </h3>
  </br>
</div>

LuminAIR is a **Machine Learning** framework that leverages [Circle STARK Proofs](https://eprint.iacr.org/2024/278) to ensure the integrity of computational graphs.

It allows provers to cryptographically demonstrate that a computational graph has been executed correctly,
while verifiers can validate these proofs with significantly fewer resources than re-executing the graph.

This makes it ideal for applications where trustlessness and integrity are paramount, such as healthcare, finance, decentralized protocols and verifiable agents.

> **⚠️ Disclaimer:** LuminAIR is currently under active development 🏗️.

## 🚀 Quick Start

To see LuminAIR in action, run the provided example:

```bash
$ cd examples/simple
$ cargo run
```

```rust
use luminair_graph::{graph::LuminairGraph, StwoCompiler};
use luminal::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cx = Graph::new();

    // Define tensors
    let a = cx.tensor((2, 2)).set(vec![1.0, 2.0, 3.0, 4.0]);
    let b = cx.tensor((2, 2)).set(vec![10.0, 20.0, 30.0, 40.0]);
    let w = cx.tensor((2, 2)).set(vec![-1.0, -1.0, -1.0, -1.0]);

    // Build computation graph
    let c = a * b;
    let mut d = (c + w).retrieve();

    // Compile the computation graph
    cx.compile(<(GenericCompiler, StwoCompiler)>::default(), &mut d);

    // Execute and generate a trace of the computation graph
    let trace = cx.gen_trace()?;

    // Generate proof and verify
    let proof = cx.prove(trace)?;
    cx.verify(proof)?;

    Ok(())
}
```

## 📖 Documentation

You can check our official documentation [here](https://luminair.gizatech.xyz/).

## 🔮 Roadmap

You can check our roadmap to unlock ML integrity [here](https://luminair.gizatech.xyz/more/roadmap).

## 🫶 Contribute

Contribute to LuminAIR and be rewarded via [OnlyDust](https://app.onlydust.com/projects/giza/overview).

Check the contribution guideline [here](https://luminair.gizatech.xyz/more/contribute)

## 📊 Benchmarks

Check performance benchmarks for LuminAIR operators [here](https://luminair.gizatech.xyz/more/benchmarks).

## 💖 Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/raphaelDkhn"><img src="https://avatars.githubusercontent.com/u/113879115?v=4?s=100" width="100px;" alt="raphaelDkhn"/><br /><sub><b>raphaelDkhn</b></sub></a><br /><a href="https://github.com/gizatechxyz/LuminAIR/commits?author=raphaelDkhn" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://audithub.app"><img src="https://avatars.githubusercontent.com/u/71888134?v=4?s=100" width="100px;" alt="malatrax"/><br /><sub><b>malatrax</b></sub></a><br /><a href="https://github.com/gizatechxyz/LuminAIR/commits?author=zmalatrax" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/blewater"><img src="https://avatars.githubusercontent.com/u/2580304?v=4?s=100" width="100px;" alt="Mario Karagiorgas"/><br /><sub><b>Mario Karagiorgas</b></sub></a><br /><a href="https://github.com/gizatechxyz/LuminAIR/commits?author=blewater" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/Tbelleng"><img src="https://avatars.githubusercontent.com/u/117627242?v=4?s=100" width="100px;" alt="Tbelleng"/><br /><sub><b>Tbelleng</b></sub></a><br /><a href="https://github.com/gizatechxyz/LuminAIR/commits?author=Tbelleng" title="Code">💻</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/sukrucildirr"><img src="https://avatars.githubusercontent.com/u/32969880?v=4?s=100" width="100px;" alt="sukrucildirr"/><br /><sub><b>sukrucildirr</b></sub></a><br /><a href="https://github.com/gizatechxyz/LuminAIR/commits?author=sukrucildirr" title="Documentation">📖</a></td>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/hakymulla"><img src="https://avatars.githubusercontent.com/u/25408889?v=4?s=100" width="100px;" alt="Kazeem Hakeem"/><br /><sub><b>Kazeem Hakeem</b></sub></a><br /><a href="https://github.com/gizatechxyz/LuminAIR/commits?author=hakymulla" title="Code">💻</a></td>
    </tr>
  </tbody>
</table>

<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->

<!-- ALL-CONTRIBUTORS-LIST:END -->

## Acknowledgements

A special thanks to the developers and maintainers of the foundational projects that make LuminAIR possible:

- [Luminal](https://github.com/jafioti/luminal): For providing a robust and flexible deep-learning library that serves as the backbone of LuminAIR.
- [Stwo](https://github.com/starkware-libs/stwo): For offering a powerful prover and constraint library.
- [Brainfuck-Stwo](https://github.com/kkrt-labs/stwo-brainfuck): Inspiration for creating AIR with the Stwo library.

## License

LuminAIR is open-source software released under the [MIT](https://opensource.org/license/mit) License.
