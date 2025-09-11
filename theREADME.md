# Music-Text

A Rust-based musical notation parser using hand-written recursive descent parsing for multiple notation systems and converting them to various musical formats (VexFlow, LilyPond, etc.).

## 🚀 PERMANENT DEVELOPMENT MODE

**This project is permanently in development mode for fastest iteration.**

- **ONE BINARY**: Always build and use the debug binary with warnings suppressed
- **ALL FEATURES INCLUDED**: GUI, web server, CLI - all in one binary
- **FASTEST COMPILATION**: `RUSTFLAGS="-A warnings" cargo build --features gui`
- **NO RELEASE BUILDS**: We prioritize iteration speed over optimization
- **INSTANT FEEDBACK**: Focus on rapid development cycles
- **⚠️ IMPORTANT**: Always use `make build` instead of direct cargo commands to ensure proper flags and configuration

**📋 Note**: Use `MUSIC_TEXT_SPECIFICATION.md` as the source of truth for terminology and naming conventions.

## 🚨 CRITICAL: Monospaced Font Required

**IMPORTANT**: Music-Text notation is **column-based** and requires a **monospaced font** for proper alignment. When using Music-Text:
- **Terminal/CLI**: Use a monospaced font (most terminals already do)
- **Text Editors**: Use a monospaced font like Courier New, Consolas, or Monaco
- **Web Interface**: The textarea is configured to use monospace fonts automatically

Column alignment is essential for features like slurs, octave markers, and multi-line annotations to work correctly.

## Current Architecture (Post-Refactoring)

### Architecture Pattern: Incremental Pipeline (Parse → Analyze → Render)

**Key Principle**: Clear separation of parsing, analysis, and rendering concerns
- **Parse Stage**: Text input to structured representation  
- **Analysis Stage**: Rhythm FSM, temporal analysis, and semantic processing
- **Render Stage**: Multi-format output generation

### Core Components

1. **Hand-Written Recursive Descent Parser** (`src/parse/`)
   - **RENAMED** from `src/document/` during incremental refactoring
   - Handles multiple notation systems (sargam, number, western, abc, doremi)
   - Supports multi-line notation with annotations and multi-stave structures
   - Processes barlines, beats, segments, and musical structure
   - Clean paragraph-based document structure parsing

2. **Processing Pipeline**
   - **Parse Model** (`src/parse/model.rs`): Core data structures for musical notation
   - **Document Parser** (`src/parse/document_parser/`): Direct document structure parsing
   - **Pipeline** (`src/pipeline.rs`): Orchestrates the processing pipeline
   - **Stave Parser** (`src/stave/`): Processes individual staves

3. **Output Generation**

   **⚠️ IMPORTANT: LilyPond is the Primary Renderer**
   - **LilyPond First**: When implementing new features, ALWAYS implement LilyPond rendering first
   - **Professional Quality**: LilyPond produces publication-quality musical scores
   - **SVG Output**: `lilypond --format=svg` generates high-quality vector graphics
   - **CLI Commands**: 
     - `music-text full-lily` - Complete LilyPond score with headers
     - `music-text minimal-lily` - Minimal LilyPond notation
   
   **Secondary Renderers:**
   - **VexFlow**: Generates VexFlow JSON and SVG for web rendering (secondary priority)

4. **Web Interface**
   - **Web Server** (`src/web_server.rs`): Integrated Rust web server module on port 3000
   - **Web UI** (`webapp/`): Interactive data flow pipeline visualization with VexFlow rendering
   - **⚠️ NOTE**: No separate Node.js server for production - the Rust binary serves static files and API endpoints directly
   - **⚠️ NOTE**: Node.js is only used for Playwright browser testing (`npx playwright test`)

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
- **Spatial slurs**: Underscore notation above content lines for phrase markings (`_____`)
- **Spatial beat grouping**: Underscore notation below content lines for rhythmic groupings (`_____`)
- **Hybrid annotation model**: Both boundary information (Start/Middle/End roles) and convenience boolean flags

## Grammar Structure

The Pest grammar (`src/document/grammar.pest`) is organized into:

## Notation Rules

A line of music is identified as a "content line".

- **With Barline:** If a line contains a barline (`|`), it is always treated as musical content.
- **Without Barline:** If a line does not contain a barline, it must have at least three consecutive musical notes (e.g., `S R G`, `1 2 3`, `C D E`) to be considered a content line. Otherwise, it is treated as a `text_line`.

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
  - `input`: The notation text to parse (URL encoded)
- Response: JSON with parsed output including VexFlow data, LilyPond, and VexFlow SVG

#### `POST /api/lilypond-svg`
Fast LilyPond SVG generation endpoint
- Request body (JSON):
  - `notation`: Music notation string (e.g., `"|SRG"`, `"|1 2 3"`)
- Response: JSON with SVG content
  - `success`: Boolean indicating if generation succeeded
  - `svg_content`: Generated SVG string (if successful)
  - `error`: Error message (if failed)

**Note**: This endpoint directly processes music notation and generates optimized LilyPond source for fastest SVG generation.

## Installation & Usage

### Prerequisites
- Rust 1.70+
- System GUI libraries (for egui/eframe support)

### Build
```bash
# Fast debug build with all features (permanent dev mode)
RUSTFLAGS="-A warnings" cargo build --features gui

# Or use make for convenience (automatically includes GUI)
make build

# Clean and rebuild
make fresh
```

### Running the Application

#### Web Server with UI (Port 3000)
```bash
# Start the integrated web server (recommended)
make web

# Or manually (if needed)
RUSTFLAGS="-A warnings" cargo run --features gui -- --web

# ALWAYS restart server after code changes
make kill  # Kill old servers
make web   # Start fresh

# Then visit http://localhost:3000 for the interactive UI
```

#### CLI Usage
```bash
# Parse with different output stages (recommended: build first with make)
make build  # Build the binary first

# Then use the binary directly (includes all features)
./target/debug/music-text document "|1 2 3"    # Show parsed document structure
./target/debug/music-text processed "|1 2 3"   # Show processed staves
./target/debug/music-text minimal-lily "|1 2 3" # Show minimal LilyPond notation
./target/debug/music-text full-lily "|1 2 3"    # Show full LilyPond score
./target/debug/music-text vexflow "|1 2 3"      # Show VexFlow data structure
./target/debug/music-text vexflow-svg "|1 2 3"  # Show VexFlow SVG rendering
./target/debug/music-text all "|1 2 3"          # Show all stages

# Or use cargo run (slower compilation)
RUSTFLAGS="-A warnings" cargo run --features gui -- document "|1 2 3"

# Other shortcuts
make gui    # Launch native GUI editor  
make repl   # Start interactive REPL

# Generate LilyPond SVG files directly (RECOMMENDED WORKFLOW)
cat row.txt | ./target/debug/music-text lilypond-svg -o row    # Creates row.ly and row.svg
echo "|1 2 3" | ./target/debug/music-text lilypond-svg        # Creates output.ly and output.svg

# Read from stdin
echo "|1 2 3" | ./target/debug/music-text document
cat input.notation | ./target/debug/music-text full-lily

# Manual SVG generation using LilyPond compiler
cat row.txt | ./target/debug/music-text full-lily > row.ly
lilypond --format=svg row.ly    # Creates row.svg
lilypond --format=png row.ly    # Creates row.png
```

### Stopping the Application
```bash
# Stop with Ctrl+C in the terminal
# Or find and kill the process:
pkill -f "music-text --web"
```

## Data Flow Pipeline

### Parsing Flow
1. **Input Text** → Document Parser (`src/document/document_parser/`)
2. **Raw Document** → Spatial Processing
   - Octave marker assignment
   - Slur analysis and assignment
   - Beat group assignment
   - Syllable to note mapping
3. **Processed Document** → Stave Processing (`src/stave/`)
4. **Spatial AST** → Rhythm FSM (when implemented)
5. **Enriched AST** → Output Renderers
   - LilyPond source generation
   - VexFlow JSON generation
   - YAML representation

### Current Status
- ✅ Hand-written recursive descent parser (replaced Pest)
- ✅ AST generation with proper document structure
- ✅ Multi-stave parsing and professional score generation
- ✅ Spatial processing (octaves, slurs, beat groups, lyrics)
- ✅ Hybrid annotation model with boundary information and convenience flags
- ✅ Web UI with data flow visualization
- ✅ API endpoints for parsing
- ✅ Complete LilyPond score rendering with simultaneous music
- ✅ Professional SVG generation via web server
- ✅ Interactive REPL with terse interface (Ctrl+D to submit, Ctrl+C to exit)
- ⚠️ Rhythm FSM integration (in development)
- ✅ Multi-stave marker detection fixed

## Examples

### Simple Number Notation
```
Input: "1 2 3 4"
Output: Four quarter notes C D E F
```

### Sargam Notation (Valid as lone line)
```
Input: "SRG"
Output: Three quarter notes Sa-Re-Ga (C-D-E)
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

### Multi-Stave Grouping
```
Input:
____
|123

|345
_____

|333
Output: 3-stave staff system with proper grouping and all 9 notes correctly transcribed
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
│       └── manual_parser/       # Hand-written recursive descent parser
│           ├── mod.rs           # Parser exports
│           ├── document.rs      # Document structure parser
│           ├── stave.rs         # Stave parser with multi-stave detection
│           ├── error.rs         # Error handling
│           └── element.rs       # Musical element parsing
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

# Test CLI with specific notation (fast debug build)
./target/debug/music-text document "S R G M"
./target/debug/music-text full-lily "1 2 3"

# Or use make shortcuts
make test-cli

# Multi-stave testing
echo -e "____\n|123\n\n|345\n_____\n\n|333" | ./target/debug/music-text full-lily

# Start web server for interactive testing
make web
# Then visit http://localhost:3000

# Run browser tests
make test-web
```

## Known Issues

1. **Rhythm FSM Integration**: Not yet fully connected to the parsing pipeline
2. **VexFlow Renderer**: May need updates for complex multi-stave scenarios

## Future Work

- Complete rhythm FSM integration for complex tuplets
- Add WASM support for browser-side parsing
- Improve error reporting and recovery
- Add more comprehensive test suite
- Optimize LilyPond template generation
- Expand notation system support

## Coding Guidelines

### File Organization Rules
- **NO mod.rs files**: Always use direct module imports. Each module should be a single `.rs` file or directory with named module files.
- **Example**: Use `src/document/document_parser.rs` instead of `src/document/document_parser/mod.rs`
- **Benefit**: Clearer module structure and easier navigation

## License

MIT
