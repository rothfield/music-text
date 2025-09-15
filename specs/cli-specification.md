# Music Text CLI Specification

## Implementation Status

**Current Implementation:** Enhanced CLI with subcommand support and TUI REPL
- ✅ `--input` string parsing
- ✅ `--file` file parsing
- ✅ `--output json|debug`
- ✅ `--web` server mode
- ✅ `--help` documentation
- ✅ Stdin input support
- ✅ **TUI REPL**: `music-text repl` (dual-pane terminal UI with live updates)
- ✅ **Document command**: `music-text document` (JSON output)
- ✅ **LilyPond command**: `music-text full-lily` (LilyPond output)
- ✅ **Performance stub**: `music-text perf` (placeholder)

**Not Yet Implemented:** Advanced CLI features marked with `[NOT YET IMPLEMENTED]` throughout this spec
- ❌ Subcommands (parse, render, debug, batch, completions)
- ❌ Additional output formats (svg - tokens available in TUI)
- ❌ Error handling and exit codes
- ❌ Configuration files

**Note**: The Makefile now works correctly with restored subcommands:
- ✅ `make repl` → `music-text repl` (TUI REPL with dual-pane interface) - **ENHANCED**
- ✅ `make perf` → `music-text perf` (Performance benchmarks stub)
- ✅ `make test-cli` → `music-text document` and `music-text full-lily` (Document/LilyPond commands) - **RESTORED**
- ❌ Shell completions support expected

## TUI REPL Features

The `music-text repl` command launches a modern terminal user interface with:

### **Interface Layout**
```
┌─ Input (ESC to quit, Tab to switch format) ──┬─ Output - LilyPond ─────────┐
│ S R G M                                      │ \version "2.24.0"           │
│ P D N S'                                     │ \score {                    │
│                                              │   \new Staff {              │
│                                              │     c'4 d'4 e'4 f'4         │
│                                              │   }                         │
│                                              │ }                           │
├──────────────────────────────────────────────┼─────────────────────────────┤
│ [LilyPond] [JSON] [Debug] [Tree] [Tokens]                                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### **Controls**
- **Type**: Enter musical notation (updates live on every keystroke)
- **Tab**: Switch output formats (LilyPond, JSON, Debug, Tree, Tokens)
- **Shift+Tab**: Previous format
- **Enter**: New line
- **Backspace**: Delete character
- **Arrow keys**: Move cursor left/right
- **ESC**: Exit TUI

### **Features**
- **Live updates**: Output refreshes automatically as you type
- **Multiple formats**: Switch between LilyPond, JSON, Debug, and syntax tokens
- **Error display**: Red error messages with clear formatting
- **Web API integration**: Uses same parsing pipeline as web interface
- **Consistent behavior**: Results match web UI exactly

### **Architecture**
The TUI REPL connects to the web server API for parsing, providing:
- ✅ Consistent results between web UI and TUI
- ✅ Hot reload capability (restart web server, not TUI)
- ✅ All output formats available
- ✅ ~7ms response time (imperceptible latency)

## Overview

This specification defines the command-line interface (CLI) for the Music Text notation parser, providing a comprehensive tool for developers, musicians, and automated systems to parse, transform, and generate music notation from text input.

### Design Philosophy

The Music Text CLI follows modern CLI design principles:
- **Intuitive**: Uses familiar command patterns and standard flag conventions
- **Consistent**: Uniform behavior across all operations and output formats
- **Composable**: Designed for pipeline integration and automation
- **Accessible**: Clear help documentation and error messages
- **Backward Compatible**: Stable interface for long-term tool integration

### Primary Use Cases

1. **Development Integration**: Parse notation in build systems and CI/CD pipelines
2. **Batch Processing**: Convert multiple music text files to various output formats
3. **Debugging**: Inspect parsing results, syntax trees, and intermediate representations
4. **Format Conversion**: Transform between notation formats (text → LilyPond, SVG, JSON)
5. **Validation**: Verify notation syntax and structure

## Command Structure

### Basic Syntax
```bash
music-text [GLOBAL_OPTIONS] [COMMAND] [COMMAND_OPTIONS] [INPUT]
```

### Command Categories
The CLI uses a **hybrid approach** combining simple operations with subcommands for complex workflows:

```bash
# Simple operations (direct flags)
music-text --input "S R G M" --output json
music-text --file notation.txt --format lilypond

# Complex operations (subcommands)
music-text parse --input "S R G M" --output json --validate
music-text render --file notation.txt --format svg --theme dark
music-text debug --input "S R G" --show-tree --show-tokens
```

## Global Options

### Input Sources
```bash
-i, --input <TEXT>        Parse notation from command line string
-f, --file <PATH>         Parse notation from file path
    --stdin               Read notation from standard input (default if no input specified) [NOT YET IMPLEMENTED]
```

### Output Control
```bash
-o, --output <FORMAT>     Output format: json, debug [currently implemented]; lilypond, svg, tokens, tree [planned]
    --pretty              Pretty-print JSON output (default for interactive use) [NOT YET IMPLEMENTED]
    --compact             Compact output (default for non-interactive use) [NOT YET IMPLEMENTED]
    --quiet, -q           Suppress status messages and warnings [NOT YET IMPLEMENTED]
    --verbose, -v         Show detailed processing information [NOT YET IMPLEMENTED]
```

### Global Flags
```bash
    --web                 Start web server mode [currently implemented]
    --version, -V         Show version information [NOT YET IMPLEMENTED]
    --help, -h            Show help information [currently implemented]
```

## Core Commands

### 1. Default Command (Parse)
When no explicit command is given, performs basic parsing:

```bash
# These are equivalent:
music-text --input "S R G M" --output json
music-text parse --input "S R G M" --output json
```

**Input Options:**
- Accepts text via `--input`, `--file`, or stdin
- Supports UTF-8 encoding
- Handles multiple stave documents

**Output Formats:**
- `json`: Structured JSON representation of parsed document
- `debug`: Human-readable debug format with detailed structure
- `lilypond`: LilyPond source code for professional typesetting
- `svg`: Scalable Vector Graphics (requires LilyPond installation)
- `tokens`: Syntax tokens for editor integration
- `tree`: Abstract syntax tree visualization

### 2. Parse Command
Explicit parsing with advanced options:

```bash
music-text parse [OPTIONS] [INPUT]
```

**Parse-specific Options:**
```bash
    --validate            Perform comprehensive validation checks
    --show-warnings       Display parsing warnings and suggestions
    --strict             Treat warnings as errors (exit code 1)
    --roundtrip          Perform roundtrip validation test
```

**Examples:**
```bash
# Basic parsing with validation
music-text parse --input "S R G M" --validate

# Parse file with strict mode
music-text parse --file song.txt --strict --output json

# Roundtrip validation
music-text parse --input "S R G M" --roundtrip --verbose
```

### 3. Render Command
Generate formatted output with advanced rendering options:

```bash
music-text render [OPTIONS] [INPUT]
```

**Render-specific Options:**
```bash
    --format <FORMAT>     Output format: lilypond, svg, pdf, midi
    --theme <THEME>       Rendering theme: default, minimal, classical
    --dpi <NUMBER>        SVG/PDF resolution (default: 300)
    --paper-size <SIZE>   Paper size: a4, letter, custom
    --no-headers          Exclude title and metadata headers
```

**Examples:**
```bash
# Generate SVG with high DPI
music-text render --file song.txt --format svg --dpi 600

# Create minimal LilyPond without headers
music-text render --input "S R G M" --format lilypond --theme minimal --no-headers
```

### 4. Debug Command
Advanced debugging and introspection:

```bash
music-text debug [OPTIONS] [INPUT]
```

**Debug-specific Options:**
```bash
    --show-tree           Display abstract syntax tree
    --show-tokens         Display syntax tokens for editor integration
    --show-pipeline       Show all pipeline stages and transformations
    --show-spatial        Display spatial assignment processing
    --show-rhythm         Show rhythm analysis results
    --benchmark           Include performance timing information
```

**Examples:**
```bash
# Full debugging pipeline
music-text debug --input "S R G M" --show-pipeline --benchmark

# Token analysis for editor development
music-text debug --file test.txt --show-tokens --output json
```

### 5. Batch Command
Process multiple files:

```bash
music-text batch [OPTIONS] <INPUT_PATTERN> <OUTPUT_DIR>
```

**Batch-specific Options:**
```bash
    --pattern <GLOB>      File matching pattern (default: "*.txt")
    --parallel <N>        Number of parallel jobs (default: CPU cores)
    --continue-on-error   Don't stop batch processing on individual file errors
    --summary             Show processing summary and statistics
```

**Examples:**
```bash
# Convert all .txt files to LilyPond
music-text batch --format lilypond "songs/*.txt" output/

# Parallel SVG generation
music-text batch --format svg --parallel 8 "*.txt" rendered/
```

## Input Handling

### Input Sources (Priority Order)
1. **Command Line**: `--input "notation text"`
2. **File Path**: `--file path/to/notation.txt`
3. **Standard Input**: Automatic if no other input specified

### Input Validation
- **Encoding**: UTF-8 required, with clear error messages for invalid encoding
- **Size Limits**: Reasonable limits with helpful error messages
- **Format Detection**: Automatic detection of notation system (Sargam, Number, Western)

### File Path Handling
```bash
# Absolute paths
music-text --file /home/user/song.txt

# Relative paths
music-text --file ./notation/song.txt

# Glob patterns (batch mode)
music-text batch "songs/*.txt" output/
```

## Output Formats

### JSON Format (`--output json`)
Structured representation of parsed document:

```json
{
  "success": true,
  "parsed_document": { /* Document structure */ },
  "rhythm_analyzed_document": { /* Processed document */ },
  "metadata": {
    "notation_systems": ["Sargam"],
    "stave_count": 1,
    "processing_time_ms": 15
  }
}
```

**Features:**
- Pretty-printed by default in interactive mode
- Compact format for non-interactive use
- Includes metadata and processing statistics

### Debug Format (`--output debug`)
Human-readable detailed representation:

```
Document Structure:
├── Stave #1 (Sargam notation)
│   ├── Content Line: |S R G M|
│   │   ├── Note: S (Sa, tonic)
│   │   ├── Whitespace: " "
│   │   ├── Note: R (Re, second)
│   │   └── ...
│   └── Rhythm Analysis: 4 beats, 4/4 time
└── Processing: 15ms, 0 warnings
```

### LilyPond Format (`--output lilypond`)
Professional music typesetting source:

```lilypond
\version "2.24"
\score {
  \relative c' {
    c d e f
  }
  \layout { }
}
```

### SVG Format (`--output svg`)
Scalable vector graphics (requires LilyPond):

- High-quality rendered notation
- Configurable DPI and paper size
- Embedded fonts for portability

### Syntax Tokens Format (`--output tokens`)
For editor integration and syntax highlighting:

```json
[
  {"type": "note", "start": 0, "end": 1, "content": "S"},
  {"type": "whitespace", "start": 1, "end": 2, "content": " "},
  {"type": "note", "start": 2, "end": 3, "content": "R"}
]
```

### Tree Format (`--output tree`)
Abstract syntax tree visualization:

```
Document
├── Stave
│   ├── ContentLine
│   │   ├── Note(S)
│   │   ├── Whitespace
│   │   └── Note(R)
│   └── RhythmItems
│       └── Beat(quarter)
```

## Error Handling

### Exit Codes
```bash
0    Success
1    General error (parsing, validation, file I/O)
2    Invalid command line arguments
3    File not found or permission denied
4    Invalid input format or encoding
5    Missing dependencies (e.g., LilyPond for SVG output)
64   Internal error (bug - please report)
```

### Error Message Format
```bash
music-text: error: <category>: <specific error>
  └─ suggestion: <helpful suggestion>
  └─ location: line 2, column 5
```

### Error Categories
- **Parse Error**: Invalid notation syntax
- **Validation Error**: Semantic issues (mixed notation systems, etc.)
- **I/O Error**: File access, permission issues
- **Dependency Error**: Missing external tools (LilyPond)
- **Internal Error**: Unexpected failures

### Error Examples
```bash
# Parse error with location
music-text: error: parse: unexpected character '&' in notation
  └─ suggestion: use only valid notation characters (S R G M P D N, 1-7, etc.)
  └─ location: line 1, column 8

# Missing dependency
music-text: error: dependency: LilyPond not found in PATH
  └─ suggestion: install LilyPond to enable SVG output
  └─ help: see https://lilypond.org/download.html

# File not found
music-text: error: file: 'song.txt' not found
  └─ suggestion: check file path and permissions
```

## Configuration

### Configuration File (Optional)
Location: `~/.config/music-text/config.toml`

```toml
[default]
output_format = "json"
pretty_print = true
show_warnings = true

[render]
theme = "default"
dpi = 300
paper_size = "a4"

[debug]
show_timing = false
show_pipeline = false
```

### Environment Variables
```bash
MUSIC_TEXT_CONFIG      # Override config file location
MUSIC_TEXT_OUTPUT      # Default output format
MUSIC_TEXT_LILYPOND    # LilyPond binary path
MUSIC_TEXT_TEMP_DIR    # Temporary file directory
```

## Integration Patterns

### Pipeline Integration
```bash
# Shell pipelines
echo "S R G M" | music-text --output lilypond > song.ly
cat songs.txt | music-text parse --validate --quiet

# Build systems (Make, Cargo, npm)
%.ly: %.txt
	music-text --file $< --output lilypond > $@

# CI/CD validation
music-text batch --validate --strict "src/**/*.txt" /dev/null
```

### Exit Code Usage
```bash
# Validation in scripts
if music-text parse --file song.txt --validate --quiet; then
    echo "Valid notation"
else
    echo "Invalid notation (exit code: $?)"
fi
```

### JSON Output Processing
```bash
# Extract specific data with jq
music-text --input "S R G M" --output json | jq '.metadata.stave_count'

# Error handling in scripts
result=$(music-text parse --file song.txt --output json 2>&1)
if [[ $? -eq 0 ]]; then
    echo "$result" | jq '.parsed_document'
else
    echo "Parse failed: $result" >&2
fi
```

## Performance Considerations

### Optimization Flags
```bash
    --no-validate         Skip validation for faster processing
    --no-rhythm           Skip rhythm analysis
    --no-spatial          Skip spatial processing
    --cache-dir <DIR>     Cache directory for expensive operations
```

### Memory Management
- **Streaming**: Large files processed in chunks when possible
- **Parallel Processing**: Batch operations use available CPU cores
- **Caching**: Intermediate results cached for repeated operations

### Benchmarking
```bash
# Performance measurement
music-text debug --input "complex notation" --benchmark

# Output includes:
# - Parse time: 15ms
# - Spatial processing: 5ms
# - Rhythm analysis: 8ms
# - Total: 28ms
```

## Accessibility and Usability

### Help System
```bash
# Comprehensive help
music-text --help

# Command-specific help
music-text parse --help
music-text render --help

# Quick reference
music-text --help-formats    # List all output formats
music-text --help-examples   # Show common usage examples
```

### Discoverability
- **Tab Completion**: Bash/Zsh completion scripts
- **Progressive Disclosure**: Basic → intermediate → advanced options
- **Examples**: Contextual examples in help text

### Internationalization
- **Error Messages**: Clear English with technical precision
- **Unicode Support**: Full UTF-8 input/output support
- **Localization**: Framework for future language support

## Testing and Quality Assurance

### Test Coverage
- **Unit Tests**: All CLI argument parsing and validation
- **Integration Tests**: End-to-end workflows and error scenarios
- **Performance Tests**: Timing and memory usage benchmarks
- **Compatibility Tests**: Cross-platform behavior verification

### Automated Testing
```bash
# Test suite execution
cargo test cli_tests

# Specific test categories
cargo test cli_args          # Argument parsing
cargo test cli_integration   # End-to-end workflows
cargo test cli_errors        # Error handling
```

## Implementation Notes

### Dependencies
- **clap**: Robust argument parsing with derive macros
- **serde_json**: JSON serialization with pretty-printing
- **tokio**: Async runtime for web server mode
- **anyhow**: Error handling and context

### Code Structure
```
src/
├── main.rs              # CLI entry point and argument parsing
├── cli/
│   ├── mod.rs           # CLI module organization
│   ├── commands.rs      # Command implementations
│   ├── args.rs          # Argument parsing and validation
│   ├── output.rs        # Output format handling
│   ├── errors.rs        # Error formatting and codes
│   └── batch.rs         # Batch processing implementation
└── ...
```

### Backward Compatibility
- **Semantic Versioning**: Major.Minor.Patch version scheme
- **Stable Interface**: CLI interface stability guarantees
- **Migration Path**: Clear upgrade guides for breaking changes

## Future Enhancements

### Planned Features
- **Interactive Mode**: REPL-style interface for experimentation
- **Plugin System**: Custom output format extensions
- **Language Server**: LSP integration for editor support
- **Watch Mode**: Auto-regenerate on file changes

### Advanced Workflows
```bash
# Interactive exploration
music-text interactive

# Custom format plugins
music-text --format custom:my-plugin --input "S R G M"

# File watching
music-text watch --format svg "src/**/*.txt" output/
```

---

*This specification defines a comprehensive, user-friendly CLI that serves both casual users and professional developers while maintaining consistency with modern CLI design principles.*