#!/bin/bash
# This script rebuilds the WebAssembly package.

echo "Rebuilding Wasm package..."
wasm-pack build --target web
echo "Wasm package rebuilt successfully."
