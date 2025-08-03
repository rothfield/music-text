#!/bin/bash

# This script runs the Rust program 'notation_parser' and then
# converts the generated .clr files to .svg files.
# It passes a filename as an argument if one is provided.
# Otherwise, it runs the program to read from stdin.

set -e

# --- Run the Rust Program ---
if [ -n "$1" ]; then
    # A filename is provided, pass it to the program
    cargo run --bin cli -- "$1"
    base_filename=$(basename -- "$1")
    base_name="${base_filename%.*}" # a.123 -> a
else
    # No filename, the program will read from stdin
    cargo run --bin cli
    base_name="stdin"
fi

# --- Convert CLR to SVG ---
tokenizer_clr="${base_name}.tokenizer.clr"
flattener_clr="${base_name}.flattener.clr"
tokenizer_svg="${base_name}.tokenizer.svg"
flattener_svg="${base_name}.flattener.svg"

if [ -f "$tokenizer_clr" ]; then
    echo "Converting $tokenizer_clr to $tokenizer_svg..."
    ~/go/bin/ansisvg < "$tokenizer_clr" > "$tokenizer_svg"
fi

if [ -f "$flattener_clr" ]; then
    echo "Converting $flattener_clr to $flattener_svg..."
    ~/go/bin/ansisvg < "$flattener_clr" > "$flattener_svg"
fi

# --- Generate LilyPond PNG ---
lily_file="${base_name}.ly"
png_file="${base_name}.png"

if [ -f "$lily_file" ]; then
    echo "Converting $lily_file to $png_file..."
    lilypond --png "$lily_file"
    
    # Strip metadata and crop watermark from PNG
    if command -v convert >/dev/null 2>&1; then
        echo "Processing $png_file..."
        # Strip metadata and autocrop to remove watermark margins
        convert "$png_file" -strip -trim +repage "$png_file" 2>/dev/null || true
    elif command -v magick >/dev/null 2>&1; then
        echo "Processing $png_file with magick..."
        # Strip metadata and autocrop to remove watermark margins
        magick "$png_file" -strip -trim +repage "$png_file" 2>/dev/null || true
    fi
fi

echo "SVG and PNG generation complete."