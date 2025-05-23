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
  "name": "@luminair/verifier",
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
npm install @luminair/verifier
\`\`\`

## Usage

\`\`\`javascript
import init, { verify, test_wasm_module } from '@luminair/verifier';

async function verifyProof() {
    // Initialize the WASM module
    await init();
    
    // Test the module
    console.log(test_wasm_module());
    
    // Load binary files
    const proofResponse = await fetch('./proof.bin');
    const settingsResponse = await fetch('./settings.bin');
    
    const proofBytes = new Uint8Array(await proofResponse.arrayBuffer());
    const settingsBytes = new Uint8Array(await settingsResponse.arrayBuffer());
    
    // Verify proof using binary data
    const result = verify(proofBytes, settingsBytes);
    
    if (result.success) {
        console.log('Proof verification successful!');
    } else {
        console.error('Verification failed:', result.error_message);
    }
}

verifyProof();
\`\`\`

## API

### \`verify(proofBytes: Uint8Array, settingsBytes: Uint8Array): VerificationResult\`

Verifies a LuminAIR proof.

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

## Binary Format

The verifier expects binary files serialized using the \`bincode\` format:

- **Proof files**: Contains \`LuminairProof<Blake2sMerkleHasher>\` 
- **Settings files**: Contains \`CircuitSettings\`
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
        button:disabled {
            background-color: #6c757d;
            cursor: not-allowed;
        }
        .file-input {
            margin: 10px 0;
            padding: 10px;
            border: 2px dashed #ddd;
            border-radius: 5px;
            text-align: center;
        }
        .file-input.has-file {
            border-color: #28a745;
            background-color: #f8fff9;
        }
        input[type="file"] {
            margin: 5px 0;
        }
        .file-info {
            font-size: 12px;
            color: #666;
            margin-top: 5px;
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
        <p>Select binary proof and settings files below:</p>
        
        <div class="file-input" id="proofFileContainer">
            <label for="proofFile">Proof Binary File (.bin):</label><br>
            <input type="file" id="proofFile" accept=".bin" onchange="handleFileSelect('proof')">
            <div class="file-info" id="proofFileInfo">No file selected</div>
        </div>
        
        <div class="file-input" id="settingsFileContainer">
            <label for="settingsFile">Settings Binary File (.bin):</label><br>
            <input type="file" id="settingsFile" accept=".bin" onchange="handleFileSelect('settings')">
            <div class="file-info" id="settingsFileInfo">No file selected</div>
        </div>
        
        <button id="verifyButton" onclick="verifyProof()" disabled>Verify Proof</button>
        <div id="verificationResult"></div>
    </div>

    <script type="module">
        import init, { verify, test_wasm_module, get_version } from './luminair_verifier_wasm.js';
        
        let wasmInitialized = false;
        let proofFile = null;
        let settingsFile = null;
        
        async function initWasm() {
            if (!wasmInitialized) {
                await init();
                wasmInitialized = true;
                console.log('WASM module initialized');
                console.log('Version:', get_version());
            }
        }
        
        window.handleFileSelect = function(type) {
            const fileInput = document.getElementById(type + 'File');
            const fileInfo = document.getElementById(type + 'FileInfo');
            const container = document.getElementById(type + 'FileContainer');
            
            if (fileInput.files.length > 0) {
                const file = fileInput.files[0];
                if (type === 'proof') {
                    proofFile = file;
                } else {
                    settingsFile = file;
                }
                
                fileInfo.textContent = `Selected: ${file.name} (${(file.size / 1024).toFixed(1)} KB)`;
                container.classList.add('has-file');
            } else {
                if (type === 'proof') {
                    proofFile = null;
                } else {
                    settingsFile = null;
                }
                
                fileInfo.textContent = 'No file selected';
                container.classList.remove('has-file');
            }
            
            // Enable verify button only when both files are selected
            const verifyButton = document.getElementById('verifyButton');
            verifyButton.disabled = !(proofFile && settingsFile);
        };
        
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
                
                if (!proofFile || !settingsFile) {
                    throw new Error('Please select both proof and settings binary files');
                }
                
                // Read file contents as binary data
                const proofBytes = new Uint8Array(await proofFile.arrayBuffer());
                const settingsBytes = new Uint8Array(await settingsFile.arrayBuffer());
                
                console.log(`Proof file size: ${proofBytes.length} bytes`);
                console.log(`Settings file size: ${settingsBytes.length} bytes`);
                
                const result = verify(proofBytes, settingsBytes);
                
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