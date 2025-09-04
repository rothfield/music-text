#!/bin/bash
# Run the notation parser CLI demo

echo "🎵 Notation Parser Demo (Pest Grammar)"
echo "📝 Testing basic parsing functionality..."
echo ""

echo "🚀 Building project..."
cargo build --release --bin cli

echo ""
echo "🎶 Testing Sargam notation: 'S R G M'"
cargo run --release --bin cli -- --input "S R G M" --system sargam --output debug

echo ""
echo "🔢 Testing Number notation: '1 2 3 4'"
cargo run --release --bin cli -- --input "1 2 3 4" --system number --output debug

echo ""
echo "🎼 Testing Western notation: 'C D E F'"
cargo run --release --bin cli -- --input "C D E F" --system western --output debug

echo ""
echo "✅ Demo complete!"