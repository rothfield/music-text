#!/bin/bash
# Rebuild the music-text project

echo "🔨 Rebuilding music-text..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
else
    echo "❌ Build failed!"
    exit 1
fi