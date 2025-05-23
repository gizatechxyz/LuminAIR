# LuminAIR WASM Verifier

A WebAssembly-based verifier for LuminAIR STARK proofs, enabling proof verification directly in the browser.

## Overview

This WASM verifier provides the same verification capabilities as the Rust verifier but compiled to WebAssembly for browser environments. It allows users to verify LuminAIR proofs client-side without needing to trust a remote verification service.

## Building

### Prerequisites

1. Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/):

   ```bash
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

2. Make sure you have the Rust toolchain installed with the `wasm32-unknown-unknown` target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Build the WASM package

```bash
# Make the build script executable
chmod +x build.sh

# Run the build
./build.sh
```

This will create a `pkg/` directory with the compiled WASM module and JavaScript bindings.

## Usage

### Installation

```bash
npm install @luminair/verifier-wasm
```

Or use the locally built package:

```bash
cd pkg
npm pack
# Then install the generated .tgz file in your project
```

### Basic Usage

```javascript
import init, {
  verify_proof_wasm,
  test_wasm_module,
} from "@luminair/verifier-wasm";

async function main() {
  // Initialize the WASM module
  await init();

  // Test that the module is working
  console.log(test_wasm_module());

  // Verify a proof
  const proofArray = new Uint8Array(proofBinaryData);
  const settingsArray = new Uint8Array(settingsBinaryData);
  
  const result = verify_proof_wasm(proofArray, settingsArray);

  if (result.success) {
    console.log("✅ Proof verification successful!");
  } else {
    console.error("❌ Verification failed:", result.error_message);
  }
}

main();
```

### Loading Binary Data from Files

```javascript
import init, { verify_proof_wasm } from "@luminair/verifier-wasm";

async function verifyFromFiles(proofFile, settingsFile) {
  await init();

  // Read binary files
  const proofBytes = new Uint8Array(await proofFile.arrayBuffer());
  const settingsBytes = new Uint8Array(await settingsFile.arrayBuffer());

  const result = verify_proof_wasm(proofBytes, settingsBytes);
  return result;
}

// Usage with file input
document.getElementById('fileInput').addEventListener('change', async (event) => {
  const files = event.target.files;
  if (files.length >= 2) {
    const result = await verifyFromFiles(files[0], files[1]);
    console.log('Verification result:', result);
  }
});
```

### HTML Example

```html
<!DOCTYPE html>
<html>
  <head>
    <title>LuminAIR Verifier</title>
  </head>
  <body>
    <script type="module">
      import init, { verify_proof_wasm } from "./luminair_verifier_wasm.js";

      async function verifyProof() {
        await init();

        // Load your binary proof and settings data
        const proofResponse = await fetch('./proof.bin');
        const settingsResponse = await fetch('./settings.bin');
        
        const proofBytes = new Uint8Array(await proofResponse.arrayBuffer());
        const settingsBytes = new Uint8Array(await settingsResponse.arrayBuffer());

        const result = verify_proof_wasm(proofBytes, settingsBytes);

        if (result.success) {
          alert("Proof verified successfully!");
        } else {
          alert("Verification failed: " + result.error_message);
        }
      }

      // Add to window for demo purposes
      window.verifyProof = verifyProof;
    </script>

    <input type="file" id="proofFile" accept=".bin" />
    <input type="file" id="settingsFile" accept=".bin" />
    <button onclick="verifyProof()">Verify Proof</button>
  </body>
</html>
```

## API Reference

### Functions

#### `verify_proof_wasm(proofBytes: Uint8Array, settingsBytes: Uint8Array): VerificationResult`

Verifies a LuminAIR proof from binary data.

**Parameters:**

- `proofBytes`: Binary proof data as Uint8Array
- `settingsBytes`: Binary settings data as Uint8Array

**Returns:** `VerificationResult` object

#### `test_wasm_module(): string`

Tests if the WASM module is working correctly.

**Returns:** Success message string

#### `get_version(): string`

Returns the version of the verifier.

**Returns:** Version string

### Types

#### `VerificationResult`

```typescript
interface VerificationResult {
  readonly success: boolean;
  readonly error_message?: string;
}
```

## Testing

After building, you can test the WASM module:

1. Start a local server:

   ```bash
   cd pkg
   python -m http.server 8000
   ```

2. Open `http://localhost:8000/example.html` in your browser

3. Use the interface to test the module and verify proofs using binary files

## Development

### Project Structure

```
crates/verifiers/wasm/
├── src/
│   ├── lib.rs          # Main WASM entry point
│   ├── verifier.rs     # Verification logic
│   └── utils.rs        # Utility functions
├── Cargo.toml          # Rust dependencies
├── build.sh            # Build script
└── README.md           # This file
```

### Building for Different Targets

The build script uses `--target web` by default. For other targets:

```bash
# For Node.js
wasm-pack build --target nodejs

# For bundlers (webpack, etc.)
wasm-pack build --target bundler

# For no modules (script tag)
wasm-pack build --target no-modules
```

### Debugging

The WASM module includes console logging for debugging. Check your browser's developer console for verification progress and error messages.

## Performance Considerations

- **Module initialization**: The WASM module only needs to be initialized once per page load
- **Memory usage**: Large proofs may require significant memory; monitor browser memory usage
- **File loading**: Use `fetch()` or `FileReader` API to load binary files efficiently

## Browser Compatibility

The WASM verifier works in all modern browsers that support:

- WebAssembly (97%+ browser support)
- ES6 modules (if using module imports)
- Binary data handling (Uint8Array, ArrayBuffer)

For older browsers, consider using a polyfill or the `no-modules` build target.

## Binary File Format

The verifier expects binary files serialized using the `bincode` format:

- **Proof files**: Contains `LuminairProof<Blake2sMerkleHasher>` serialized with bincode
- **Settings files**: Contains `CircuitSettings` serialized with bincode

To generate compatible binary files from Rust:

```rust
use bincode;
use std::fs::File;
use std::io::Write;

// Serialize proof to binary
let proof_bytes = bincode::serialize(&proof)?;
let mut proof_file = File::create("proof.bin")?;
proof_file.write_all(&proof_bytes)?;

// Serialize settings to binary
let settings_bytes = bincode::serialize(&settings)?;
let mut settings_file = File::create("settings.bin")?;
settings_file.write_all(&settings_bytes)?;
```

## Security Notes

- Verification happens entirely client-side
- No network requests are made during verification
- The same cryptographic verification as the Rust implementation
- Proofs and settings never leave the browser

## Contributing

When making changes:

1. Update the Rust code in `src/`
2. Run `./build.sh` to rebuild the WASM package
3. Test with the example HTML file
4. Update documentation as needed

## Troubleshooting

### Common Issues

**"Module not found" errors:**

- Ensure you've run `./build.sh` successfully
- Check that the `pkg/` directory contains the generated files

**"WASM module failed to initialize":**

- Check browser console for specific error messages
- Ensure your browser supports WebAssembly
- Try serving files over HTTP (not file://) due to CORS restrictions

**Out of memory errors:**

- Large proofs may exceed browser memory limits
- Consider splitting verification into smaller chunks
- Monitor memory usage in browser dev tools

**Verification takes too long:**

- WASM verification is computationally intensive
- Consider showing a progress indicator to users
- Binary format provides optimal performance

**Invalid binary data errors:**

- Ensure binary files are generated using the correct `bincode` serialization
- Verify file integrity and format
- Check that proof and settings are compatible versions

### Getting Help

- Check the browser console for detailed error messages
- Ensure your binary files are properly formatted
- Verify that the proof was generated with a compatible version

## License

LuminAIR is open-source software released under the [MIT](https://opensource.org/license/mit) License.
