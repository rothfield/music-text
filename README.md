# Music-Text

A Rust-based musical notation parser using Pest grammar for parsing multiple notation systems and converting them to various musical formats (VexFlow, LilyPond, etc.).

## Current Architecture (As-Is)

### Architecture Pattern: AST-First Rendering

**Key Principle**: AST → Renderers (not FSM → Renderers)
- **FSM Role**: Rhythm analysis and tuplet detection only
- **Renderer Role**: Transform AST to output formats (LilyPond, VexFlow, etc.)  
- **Clean Separation**: FSM enriches/analyzes AST, renderers consume AST

### Core Components

1. **Pest Grammar Parser** (`src/document/grammar.pest`)
   - Handles multiple notation systems (sargam, number, western, abc, doremi)
   - Supports multi-line notation with annotations
   - Processes barlines, beats, segments, and musical structure

2. **Document Processing Pipeline**
   - **Document Model** (`src/document/model.rs`): Core data structures for musical notation
   - **Tree Transformer** (`src/document/tree_transformer/`): Transforms Pest parse tree to document model
   - **Pipeline** (`src/pipeline.rs`): Orchestrates the processing pipeline
   - **Stave Parser** (`src/stave_parser.rs`): Parses individual staves

3. **Output Generation**
   - **LilyPond**: Generates LilyPond notation source
   - **VexFlow**: Generates VexFlow JSON and SVG for web rendering

4. **Web Interface**
   - **Web Server** (`src/web_server.rs`): Integrated Rust web server module on port 3000
   - **Web UI** (`webapp/`): Interactive data flow pipeline visualization with VexFlow rendering

## Supported Notation Systems

The parser supports multiple musical notation input systems, all with tonic-based transposition:

1. **Sargam**: `S R G M P D N` (Indian classical)
2. **Number**: `1 2 3 4 5 6 7` (numeric system)
3. **Western**: `C D E F G A B` (western notation)
4. **ABC**: Standard ABC notation format
5. **Hindi**: Unicode Hindi notation characters
6. **Doremi**: `d r m f s l t` (doremi system)

### Key Features

- **Tonic-based system**: All notation systems work with configurable tonic (e.g., `key: D`)
- **Rhythm parsing**: Complex tuplets, ties, and duration handling via FSM
- **Multi-line notation**: Supports upper annotation lines (ornaments, chords, tala), content lines, lower annotation lines (octave markers), and lyrics
- **Barlines and structure**: Full support for single bars, double bars, repeats, and segment organization
- **Slurs and ornaments**: Parenthetical slurs and ornament notation
- **Spatial beat grouping**: Underline notation below content lines for complex rhythmic groupings

## Grammar Structure

The Pest grammar (`src/document/grammar.pest`) is organized into:

### Top-Level Structure
- `document`: Root rule handling directives and staves
- `directives_section`: Key-value pairs (e.g., `key: C`, `time: 4/4`)
- `stave`: Multi-line musical notation unit

### Line Types
- `content_line`: Main musical notes with barlines and segments
- `upper_annotation_line`: Ornaments, chords, tala markings above notes
- `lower_annotation_line`: Octave dots and kommal indicators below notes
- `lyrics_line`: Syllables aligned with musical content

### Musical Elements
- `segment`: Collections of beats separated by spaces
- `beat`: Note groupings (spatial regrouping handled by underline processor)
- `pitch`: Note names with optional accidentals
- `dash`: Note extender symbol (`-`)
- `barline`: Various types (`|`, `||`, `|:`, `:|`, `|.`)

## API Endpoints

### Web Server API (Port 3000)

#### `GET /api/parse`
Main parsing endpoint
- Query parameters:
  - `text`: The notation text to parse (URL encoded)
- Response: JSON with parsed output including VexFlow data, LilyPond, and VexFlow SVG

## Installation & Usage

### Prerequisites
- Rust 1.70+

### Build
```bash
# Clean build (removes previous artifacts)
cargo clean
cargo build --release
```

### Running the Application

#### Web Server with UI (Port 3000)
```bash
# Start the integrated web server
./target/release/music-text --web

# Then visit http://localhost:3000 for the interactive UI
```

#### CLI Usage
```bash
# Parse with different output stages
./target/release/music-text pest "|1 2 3"        # Show raw PEST parse tree
./target/release/music-text document "|1 2 3"    # Show parsed document structure
./target/release/music-text processed "|1 2 3"   # Show processed staves
./target/release/music-text minimal-lily "|1 2 3" # Show minimal LilyPond notation
./target/release/music-text full-lily "|1 2 3"    # Show full LilyPond score
./target/release/music-text vexflow "|1 2 3"      # Show VexFlow data structure
./target/release/music-text vexflow-svg "|1 2 3"  # Show VexFlow SVG rendering
./target/release/music-text all "|1 2 3"          # Show all stages

# Read from stdin
echo "|1 2 3" | ./target/release/music-text document
cat input.notation | ./target/release/music-text full-lily
```

### Stopping the Application
```bash
# Stop with Ctrl+C in the terminal
# Or find and kill the process:
pkill -f "music-text --web"
```

## Data Flow Pipeline

### Parsing Flow
1. **Input Text** → Pest Grammar Parser
2. **Pest Parse Tree** → AST Conversion (`src/parser.rs`)
3. **Raw AST** → Spatial Processing (`src/spatial_parser.rs`)
   - Slur analysis
   - Octave marker assignment
   - Syllable to note mapping
4. **Spatial AST** → Rhythm FSM (when implemented)
5. **Enriched AST** → Output Renderers
   - LilyPond source generation
   - VexFlow JSON generation
   - YAML representation

### Current Status
- ✅ Pest grammar parsing
- ✅ AST generation
- ✅ Spatial processing (slurs, octaves, lyrics)
- ✅ Web UI with data flow visualization
- ✅ API endpoints for parsing
- ⚠️ Rhythm FSM integration (in development)
- ⚠️ Empty segments issue (being investigated)
- ✅ Leading newline handling (fixed)

## Examples

### Simple Number Notation
```
Input: "1 2 3 4"
Output: Four quarter notes C D E F
```

### Sargam with Rhythm
```
Input: "| S-R G-M |"
Output: Half notes Sa-Re, half notes Ga-Ma in 4/4 time
```

### Complex Tuplet
```
Input: "1-2-3"
Output: 3/2 tuplet with dotted quarter C, quarter D, eighth E
```

### Multi-line with Annotations
```
Input:
```
[Am]     [Dm]
| S R G M | P D N S |
do re mi fa  sol la ti do
```
Output: Sargam notes with chord symbols and lyrics
```

## Project Structure

```
music-text/
├── src/
│   ├── main.rs                  # CLI interface and entry point
│   ├── web_server.rs            # Web server module (port 3000)
│   ├── lib.rs                   # Public API
│   ├── pipeline.rs              # Processing pipeline
│   ├── stave_parser.rs          # Stave parsing logic
│   └── document/                # Document processing
│       ├── mod.rs               # Module exports
│       ├── model.rs             # Domain models
│       ├── pest_interface.rs    # Pest integration
│       ├── grammar.pest         # Pest grammar file
│       └── tree_transformer/    # AST transformation
│           ├── mod.rs           # Transformer exports
│           ├── document.rs      # Document transformer
│           ├── stave.rs         # Stave transformer
│           ├── content_line.rs  # Content line transformer
│           ├── pitch.rs         # Pitch transformer
│           └── helpers.rs       # Helper functions
├── webapp/
│   ├── app.js                   # Web UI JavaScript
│   ├── index.html               # Web UI HTML
│   └── styles.css               # Web UI styles
├── Cargo.toml
└── README.md
```

### Testing
```bash
# Run unit tests
cargo test

# Test CLI with specific notation
./target/release/music-text pest "S R G M"
./target/release/music-text full-lily "1 2 3"

# Start web server for interactive testing
./target/release/music-text --web
# Then visit http://localhost:3000
```

## Known Issues

1. **Empty Segments**: Parser currently not populating segments in AST (under investigation)
2. **Rhythm FSM Integration**: Not yet connected to the parsing pipeline
3. **Renderer Integration**: LilyPond and VexFlow renderers not yet connected

## Future Work

- Complete rhythm FSM integration
- Fix empty segments issue in parser
- Connect output renderers to the pipeline
- Add WASM support for browser-side parsing
- Improve error reporting and recovery
- Add more comprehensive test suite

## License

MI
