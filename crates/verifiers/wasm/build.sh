#!/bin/bash

# Build script for the LuminAIR WASM verifier
# This script builds the WASM package and generates TypeScript bindings

set -e

echo "🚀 Building LuminAIR WASM Verifier..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack is not installed. Please install it with:"
    echo "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Navigate to the WASM verifier directory
cd "$(dirname "$0")"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf pkg/ || true

# Build the WASM package
echo "🔨 Building WASM package..."
wasm-pack build --target web --out-dir pkg --scope luminair

# Copy additional files
echo "📁 Copying additional files..."

# Copy custom TypeScript definitions
if [ -f "luminair_verifier_wasm.d.ts" ]; then
    echo "📝 Copying custom TypeScript definitions..."
    cp luminair_verifier_wasm.d.ts pkg/luminair_verifier_wasm.d.ts
fi

# Create a simple package.json if it doesn't exist
if [ ! -f pkg/package.json ]; then
    echo "📦 Creating package.json..."
    cat > pkg/package.json << EOF
{
  "name": "@luminair/verifier-wasm",
  "version": "0.0.1",
  "description": "LuminAIR WASM Verifier for browser-based proof verification",
  "main": "luminair_verifier_wasm.js",
  "module": "luminair_verifier_wasm.js",
  "types": "luminair_verifier_wasm.d.ts",
  "files": [
    "luminair_verifier_wasm_bg.wasm",
    "luminair_verifier_wasm.js",
    "luminair_verifier_wasm.d.ts"
  ],
  "keywords": [
    "wasm",
    "proof",
    "verification",
    "luminair",
    "stark"
  ],
  "author": "LuminAIR Team",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/raphaelDkhn/Luminair"
  }
}
EOF
fi

# Create a README for the package
echo "📄 Creating README..."
cat > pkg/README.md << EOF
# LuminAIR WASM Verifier

Browser-based proof verifier for LuminAIR STARK proofs.

## Installation

\`\`\`bash
npm install @luminair/verifier-wasm
\`\`\`

## Usage

\`\`\`javascript
import init, { verify_proof_wasm, test_wasm_module } from '@luminair/verifier-wasm';

async function verifyProof() {
    // Initialize the WASM module
    await init();
    
    // Test the module
    console.log(test_wasm_module());
    
    // Verify a proof (proof and settings should be JSON strings)
    const result = verify_proof_wasm(proofJson, settingsJson);
    
    if (result.success) {
        console.log('Proof verification successful!');
    } else {
        console.error('Verification failed:', result.error_message);
    }
}

verifyProof();
\`\`\`

## API

### \`verify_proof_wasm(proofJson: string, settingsJson: string): VerificationResult\`

Verifies a LuminAIR proof from JSON strings.

### \`verify_proof_binary(proofBytes: Uint8Array, settingsBytes: Uint8Array): VerificationResult\`

Verifies a LuminAIR proof from binary data (more efficient).

### \`test_wasm_module(): string\`

Tests if the WASM module is working correctly.

### \`get_version(): string\`

Returns the version of the verifier.

## VerificationResult

\`\`\`typescript
interface VerificationResult {
    readonly success: boolean;
    readonly error_message?: string;
}
\`\`\`
EOF

# Create example HTML file
echo "🌐 Creating example HTML..."
cat > pkg/example.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>LuminAIR WASM Verifier Example</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }
        .container {
            border: 1px solid #ddd;
            padding: 20px;
            border-radius: 5px;
            margin: 10px 0;
        }
        .success {
            background-color: #d4edda;
            border-color: #c3e6cb;
        }
        .error {
            background-color: #f8d7da;
            border-color: #f5c6cb;
        }
        button {
            background-color: #007bff;
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 5px;
            cursor: pointer;
        }
        button:hover {
            background-color: #0056b3;
        }
        textarea {
            width: 100%;
            height: 100px;
            margin: 10px 0;
        }
    </style>
</head>
<body>
    <h1>LuminAIR WASM Verifier Example</h1>
    
    <div class="container">
        <h2>Module Test</h2>
        <button onclick="testModule()">Test WASM Module</button>
        <div id="moduleResult"></div>
    </div>
    
    <div class="container">
        <h2>Proof Verification</h2>
        <p>Enter your proof JSON and settings JSON below:</p>
        
        <label for="proofInput">Proof JSON:</label>
        <textarea id="proofInput" placeholder="Paste your proof JSON here..."></textarea>
        
        <label for="settingsInput">Settings JSON:</label>
        <textarea id="settingsInput" placeholder="Paste your settings JSON here..."></textarea>
        
        <button onclick="verifyProof()">Verify Proof</button>
        <div id="verificationResult"></div>
    </div>

    <script type="module">
        import init, { verify_proof_wasm, test_wasm_module, get_version } from './luminair_verifier_wasm.js';
        
        let wasmInitialized = false;
        
        async function initWasm() {
            if (!wasmInitialized) {
                await init();
                wasmInitialized = true;
                console.log('WASM module initialized');
                console.log('Version:', get_version());
            }
        }
        
        window.testModule = async function() {
            try {
                await initWasm();
                const result = test_wasm_module();
                document.getElementById('moduleResult').innerHTML = 
                    `<div class="container success">✅ ${result}</div>`;
            } catch (error) {
                document.getElementById('moduleResult').innerHTML = 
                    `<div class="container error">❌ Error: ${error.message}</div>`;
            }
        };
        
        window.verifyProof = async function() {
            try {
                await initWasm();
                
                const proofJson = document.getElementById('proofInput').value;
                const settingsJson = document.getElementById('settingsInput').value;
                
                if (!proofJson || !settingsJson) {
                    throw new Error('Please provide both proof and settings JSON');
                }
                
                const result = verify_proof_wasm(proofJson, settingsJson);
                
                if (result.success) {
                    document.getElementById('verificationResult').innerHTML = 
                        `<div class="container success">✅ Proof verification successful!</div>`;
                } else {
                    document.getElementById('verificationResult').innerHTML = 
                        `<div class="container error">❌ Verification failed: ${result.error_message}</div>`;
                }
            } catch (error) {
                document.getElementById('verificationResult').innerHTML = 
                    `<div class="container error">❌ Error: ${error.message}</div>`;
            }
        };
        
        // Initialize on page load
        initWasm();
    </script>
</body>
</html>
EOF

echo "✅ WASM build complete!"
echo "📦 Package available in: pkg/"
echo "🌐 Example HTML available in: pkg/example.html"
echo ""
echo "To test the package:"
echo "1. cd pkg"
echo "2. python -m http.server 8000"
echo "3. Open http://localhost:8000/example.html"
echo ""
echo "To publish to npm:"
echo "1. cd pkg"
echo "2. npm publish --access public"