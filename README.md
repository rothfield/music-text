# Music-Text

A Rust-based musical notation parser using Pest grammar for parsing multiple notation systems and converting them to various musical formats (VexFlow, LilyPond, etc.).

## Current Architecture (As-Is)

### Architecture Pattern: AST-First Rendering

**Key Principle**: AST → Renderers (not FSM → Renderers)
- **FSM Role**: Rhythm analysis and tuplet detection only
- **Renderer Role**: Transform AST to output formats (LilyPond, VexFlow, etc.)  
- **Clean Separation**: FSM enriches/analyzes AST, renderers consume AST

### Core Components

1. **Pest Grammar Parser** (`grammar/notation.pest`)
   - Handles multiple notation systems (sargam, number, western, abc, doremi)
   - Supports multi-line notation with annotations
   - Processes barlines, beats, segments, and musical structure

2. **AST Processing Pipeline**
   - **Parser** (`src/parser.rs`): Pest grammar to AST conversion
   - **AST** (`src/ast.rs`): Core data structures for musical notation
   - **Spatial Parser** (`src/spatial_parser.rs`): Handles slurs, octave markers, syllable assignment
   - **AST to Parsed** (`src/ast_to_parsed.rs`): Converts AST to parsed elements

3. **Rhythm Processing** (In Development)
   - **Rhythm FSM** (`src/rhythm_fsm.rs`): Rhythm analysis and tuplet detection
   - **Parser V2 FSM** (`src/parser_v2_fsm.rs`): Beat grouping and subdivision logic
   - **Simple FSM** (`src/simple_fsm.rs`): Simplified rhythm processing

4. **Output Renderers** (Terminology Note: Using "renderer" not "converter")
   - **LilyPond Renderer** (`src/renderers/lilypond/`): Generate LilyPond notation source
   - **VexFlow Renderer** (`src/renderers/vexflow/`): Generate VexFlow JSON for web rendering

5. **Web Interface**
   - **Web Server** (`src/web_server.rs`): Rust web server on port 3000
   - **Express Proxy** (`webapp/server.js`): Node.js server on port 8000
   - **Web UI** (`webapp/public/index.html`): Interactive data flow pipeline visualization

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

The Pest grammar (`src/grammar.pest`) is organized into:

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

### Pest Parser Server (Port 3000)

#### `GET /health`
Health check endpoint
- Query params: `?detailed=true` for extended info
- Response: `{ status, parser, version, endpoints }`

#### `POST /api/parse`
Main parsing endpoint
- Request body:
  ```json
  {
    "notation": "1 2 3",
    "system": "number",  // optional: "auto", "sargam", "number", "western", "abc", "doremi"
    "output": ["ast", "vexflow", "lilypond", "yaml"]  // optional
  }
  ```
- Response:
  ```json
  {
    "success": true,
    "error": null,
    "ast": {...},
    "spatial": "...",
    "vexflow": {...},
    "lilypond": "...",
    "yaml": "..."
  }
  ```

#### `POST /api/parse/ast`
Returns only AST output

#### `POST /api/parse/full`
Returns all available output formats

### Web UI Server (Port 8000)

Express server serving the interactive web interface and proxying requests to the pest parser.

## Installation & Usage

### Prerequisites
- Rust 1.70+
- Node.js 16+ (for web interface)

### Build
```bash
cargo build --release
```

### Running the Servers

#### 1. Start the Pest Parser Server (Port 3000)
```bash
./target/release/cli --web
# Or
cargo run -- --web
```

#### 2. Start the Web UI Server (Port 8000)
```bash
cd webapp
node server.js
```

Then visit http://localhost:8000 for the interactive UI.

### CLI Usage
```bash
# Parse a string directly
./target/release/cli --input "| S R G M |"

# Parse a file
./target/release/cli --file input.notation

# Different output formats
./target/release/cli --input "1 2 3" --output json
./target/release/cli --input "1 2 3" --output debug
./target/release/cli --input "1 2 3" --output ast

# Specify notation system
./target/release/cli --input "S R G M" --system sargam
./target/release/cli --input "1 2 3 4" --system number
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
│   ├── main.rs                  # CLI interface
│   ├── lib.rs                   # Public API
│   ├── parser.rs                # Pest grammar integration
│   ├── ast.rs                   # AST definitions
│   ├── ast_to_parsed.rs         # AST to parsed elements
│   ├── spatial_parser.rs        # Spatial processing (slurs, octaves)
│   ├── parser_v2_fsm.rs         # V2 rhythm FSM
│   ├── rhythm_fsm.rs            # Rhythm analysis
│   ├── simple_fsm.rs            # Simple FSM implementation
│   ├── web_server.rs            # Rust web server (port 3000)
│   ├── web.rs                   # Web utilities
│   ├── models/
│   │   ├── mod.rs               # Model exports
│   │   ├── domain.rs            # Domain types
│   │   ├── parsed.rs            # Parsed element types
│   │   ├── pitch.rs             # Pitch/degree definitions
│   │   └── rhythm.rs            # Rhythm utilities
│   └── renderers/
│       ├── lilypond/            # LilyPond generation
│       └── vexflow/             # VexFlow JSON generation
├── grammar/
│   └── notation.pest            # Pest grammar file
├── webapp/
│   ├── server.js                # Express server (port 8000)
│   └── public/
│       ├── index.html           # Web UI
│       └── js/
│           ├── main.js          # UI logic
│           └── api.js           # API client
├── Cargo.toml
└── README.md
```

### Testing
```bash
# Run unit tests
cargo test

# Test specific notation system
cargo run -- --input "S R G M" --system sargam --output debug

# Start web server for interactive testing
cargo run -- --web
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
