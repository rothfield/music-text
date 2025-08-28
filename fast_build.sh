#!/bin/bash

# Quick development build script
# Use this for day-to-day development

set -e

PROJECT_DIR="/home/john/projects/notation_parser"
cd "$PROJECT_DIR"

echo "⚡ Fast Development Build"
echo "========================"

# Check if ramdisk is set up
if [ -L "target" ]; then
    RAMDISK_TARGET=$(readlink target)
    if mountpoint -q "$(dirname "$RAMDISK_TARGET")" 2>/dev/null; then
        echo "✅ Using ramdisk at $RAMDISK_TARGET"
    else
        echo "⚠️  Ramdisk not mounted, using regular filesystem"
    fi
else
    echo "ℹ️  Using regular filesystem (no ramdisk)"
fi

echo ""

# Quick check first (fastest feedback)
echo "🔍 Quick check..."
time cargo check

echo ""

# Build if check passed
echo "🔨 Building..."
time cargo build

echo ""

# Build WASM for web testing
echo "🌐 Building WASM..."
time wasm-pack build --dev --target web --out-dir webapp/pkg

echo ""
echo "✅ Build complete!"

# Show target directory size
echo "📊 Target directory usage:"
du -sh target/ 2>/dev/null || echo "Could not check target size"