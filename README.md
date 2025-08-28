# Notation Parser - V2 Architecture

A **revolutionary multi-notation music parser** with V2 clean-slate architecture, supporting Western, Sargam (Indian), and Numeric notation systems. The parser features **mathematical tuplet processing**, **AI-first documentation**, and **dual converter rewrites** for LilyPond and VexFlow output.

## V2 Major Features & Innovations

### **🚀 V2 Architecture Innovations**
- **Clean-Slate FSM-Centric Design**: Mathematical rhythm processing as architectural core
- **Type-Safe Data Structures**: `ParsedElement` enums eliminate impossible states
- **Mathematical Precision**: Fractional arithmetic throughout, zero floating point
- **AI-First Documentation**: 886+ lines of LLM-focused domain knowledge encoding
- **Dual Converter Rewrites**: Complete VexFlow and LilyPond V2 implementations

### **🎵 Musical Capabilities**
- **Advanced Tuplet Processing**: Power-of-2 detection with CRITICAL tuplet duration rule
- **Multi-Notation Support**: Western (A-G), Sargam (S,r,R,g,G,m,M,P,d,D,n,N), Numeric (1-7)
- **Rhythm Detection**: Clean FSM-based parsing with mathematical subdivisions
- **Extended Notes**: Dash notation for note extension (e.g., `1-2-3` → 5/4 tuplet)
- **Enhanced Lyrics**: Structured `ParsedChild::Syllable` with spatial positioning

### **🔧 Output Systems**
- **V2 LilyPond**: Mustache template system with compact web-optimized SVG
- **V2 VexFlow**: Direct FSM processing, no hierarchical dependencies  
- **WASM Integration**: Successfully built webapp/pkg with V2 system
- **Web UI**: Dual output display (VexFlow + LilyPond) with working WASM
- **CLI Enhancement**: `--to-lilypond` flag with unified V2 parser

## Notation Syntax

## V2 System Examples

### **Mathematical Tuplet Processing**

V2 uses the **CRITICAL tuplet duration rule** for precise rhythm calculation:

```rust
// Input: "1-2-3 -4#" (Complex rhythm with tuplet)
// V2 Processing:

// Beat 1: "1-2-3" → divisions=5 (NOT power of 2 = 5/4 tuplet)
// Rule: Find next lower power of 2: 5 → 4, calculate as divisions=4
// Each unit = 1/4 ÷ 4 = 1/16
// Note 1: 2×(1/16) = 1/8 → eighth note
// Note 2: 2×(1/16) = 1/8 → eighth note  
// Note 3: 1×(1/16) = 1/16 → sixteenth note

// Beat 2: "-4#" → divisions=2 (IS power of 2 = regular beat)
// Rest: quarter, Note 4#: quarter

// V2 Outputs:
LilyPond: "\tuplet 5/4 { c8 d8 e16 } r4 fs4"
VexFlow:  [{"notes": [...], "tuplet": {"ratio": [5,4], "notes": [0,1,2]}}]
```

### **Type-Safe Data Structures**

```rust
// V1 OLD: Monolithic Node with Optional fields
pub struct Node {
    pub node_type: String,           // "PITCH", "REST"  
    pub pitch_code: Option<PitchCode>, // Only sometimes present
    pub syl: Option<String>,         // Only for lyrics
}

// V2 NEW: Type-safe ParsedElement enums
pub enum ParsedElement {
    Note { 
        pitch_code: PitchCode,           // Always present
        octave: i8,                      // Always present
        children: Vec<ParsedChild>,      // Structured syllables/ornaments
        duration: Option<(usize, usize)>, // Mathematical fractions
    },
    Rest { /* structured fields */ },
}
```

### **Enhanced Lyrics System** 

```rust
// V1: Flat string field
pub syl: Option<String>

// V2: Structured with positioning  
ParsedChild::Syllable { 
    text: String,
    distance: i8,  // Vertical positioning
}
```

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