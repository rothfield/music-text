# Notation Parser

A multi-notation music parser supporting Western, Sargam (Indian), and Numeric notation systems. The parser converts text-based musical notation into structured data formats including LilyPond, VexFlow, YAML, and JSON.

## Features

- **Multi-Notation Support**: Automatically detects and parses Western (A-G), Sargam (S,r,R,g,G,m,M,P,d,D,n,N), and Numeric (1-7) notation systems
- **Multiple Output Formats**:
  - LilyPond notation for music engraving
  - VexFlow JSON for web-based music rendering
  - YAML/JSON for structured data processing
  - Colorized HTML output with syntax highlighting
- **Rhythm Detection**: FSM-based rhythm parsing with support for beats, measures, and time signatures
- **Metadata Extraction**: Parses titles, directives, key signatures, and other musical metadata
- **Octave Support**: Handles octave markers (`,` `.` `'` `:`)
- **WASM Support**: Can be compiled to WebAssembly for browser-based applications
- **CLI and Library**: Available as both a command-line tool and a Rust library

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Build from Source

```bash
git clone https://github.com/yourusername/notation_parser.git
cd notation_parser
cargo build --release
```

### Install CLI

```bash
cargo install --path .
```

## Usage

### Command Line Interface

```bash
# Parse a file
notation_parser input.txt

# Parse from stdin
echo "C D E F G" | notation_parser

# Parse and save output files
cargo run --bin cli mymusic.txt
# Creates: test_output/mymusic_colored.html, mymusic.ly, mymusic.yaml, mymusic.json
```

### As a Library

```rust
use notation_parser::{unified_parser, convert_to_lilypond};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = "Title: My Song\nC D E F | G A B C'";
    let document = unified_parser(input)?;
    let lilypond = convert_to_lilypond(&document)?;
    println!("{}", lilypond);
    Ok(())
}
```

### WebAssembly Usage

```javascript
import init, { parse_notation, get_lilypond_output, get_json_output } from './notation_parser.js';

await init();

if (parse_notation("C D E F G")) {
    console.log(get_lilypond_output());
    console.log(get_json_output());
}
```

## Notation Examples

### Western Notation
```
Title: Scale in C Major
C D E F G A B C'
```

### Sargam Notation
```
Title: Indian Classical Scale
S R G m P D N S'
```

### Numeric Notation
```
Title: Simple Melody
1 2 3 4 5 6 7 1'
```

### With Rhythm and Octaves
```
Title: Rhythmic Example
C D E F | G, A, B, C | D' E' F' G' |
```

## Output Formats

### LilyPond Output
Generates standard LilyPond notation files that can be compiled to PDF sheet music:
```lilypond
\version "2.24.0"
\header {
    title = "My Song"
}
\relative c' {
    c4 d e f | g a b c |
}
```

### VexFlow JSON
Produces JSON format compatible with VexFlow.js for web rendering:
```json
[
  {
    "clef": "treble",
    "keys": ["c/4"],
    "duration": "q",
    "note_type": "Note"
  }
]
```

## Architecture

This parser handles **hand-written spatial textual notation** that has never been parsed before. Unlike traditional music notation formats, this system preserves the 2D spatial relationships in the original text.

### Key Design Insight: Dual AST Levels

The parser maintains **two distinct AST representations**:

1. **Parsed AST** (Flat) - What the parser extracts from the raw text:
   - Individual notes, rests, barlines as they appear spatially
   - No artificial groupings that don't exist in the source notation
   - Faithful to what the human actually wrote down

2. **Structured AST** (Hierarchical) - What the FSM creates for musical interpretation:
   - Groups individual elements into beats, measures, tuplets
   - Adds semantic musical meaning (e.g., "these 5 notes form a quintuplet")
   - Optimized for music rendering systems (VexFlow, LilyPond)

### Processing Pipeline

1. **Lexical Analysis**: Tokenizes input text using a handwritten lexer
2. **Notation Detection**: Automatically identifies the notation system (Western/Sargam/Numeric)
3. **Spatial Analysis**: 
   - **Phase 1** (`node_builder`): Converts tokens to flat hierarchical nodes, handling vertical spatial relationships (octave markers, ornaments, lyrics)
   - **Phase 2** (`region_processor`): Processes horizontal spatial regions (slur overlines, beat brackets)
4. **Rhythm Analysis**: FSM transforms flat AST into structured AST with beats and measures
5. **Conversion**: Transforms the structured AST into various output formats

### Spatial Layout Handling

The system uniquely handles **2D spatial notation** where:
- **Slurs** are drawn as overlines: `_______` above notes like `G -P | S`
- **Octave markers** appear above/below pitches: dots, colons, apostrophes
- **Lyrics** are positioned below the musical line
- **Beat brackets** use underscores below notes to group rhythmic units

The handwritten/pencil-and-paper approach allows easy notation of spatial information (similar to Neumes in medieval notation). Our challenge here is to allow users to enter this spatial musical information as text, preserving the 2D relationships that are crucial for musical meaning in a linear text format.

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test binary
cargo run --bin simple_test
cargo run --bin test_notation_detection
```

### Building for WASM

```bash
wasm-pack build --target web
```

### Project Structure

```
notation_parser/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── models.rs           # Core data structures
│   ├── lexer.rs            # Tokenization
│   ├── parser.rs           # Parsing logic
│   ├── notation_detector.rs # Auto-detection of notation systems
│   ├── rhythm_fsm.rs       # Rhythm state machine
│   ├── lilypond_converter.rs # LilyPond output
│   ├── vexflow_converter.rs  # VexFlow JSON output
│   └── bin/
│       └── cli.rs          # Command-line interface
├── Cargo.toml
└── README.md
```

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- VexFlow converter inspired by Tarmo Johannes' vexflow-react-components
- Built with Rust and the wasm-bindgen ecosystem