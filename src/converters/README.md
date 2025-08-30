# Converters Module

This module contains all notation format converters that transform the FSM output into various musical notation formats. The converters share common musical logic while handling format-specific rendering.

## Architecture

```
src/converters/
├── README.md           # This file
├── mod.rs              # Module exports and re-exports
├── transposition.rs    # Shared musical transposition logic
├── lilypond.rs         # LilyPond text format converter  
└── vexflow.rs          # VexFlow JSON format converter
```

## Shared Components

### Transposition (`transposition.rs`)

All converters use the same mathematical transposition logic to implement the **movable-do tonic system**:

```rust
pub fn transpose_degree_with_octave(
    degree: Degree, 
    octave: i8, 
    tonic: Degree
) -> (Degree, i8)
```

**Key features:**
- **Movable-do system**: Scale degrees are relative to the tonic
- **Octave wrapping**: Handles cases where transposed notes cross octave boundaries  
- **Mathematical precision**: Uses scale positions (0-6) + accidental offsets
- **Comprehensive**: Supports all degree variants (N1, N1s, N1b, N1ss, N1bb, etc.)

**Example:**
```rust
// Scale degree 7 in D major
let (transposed, adjusted_octave) = transpose_degree_with_octave(
    Degree::N7,    // Input: scale degree 7
    0,             // Input: middle octave
    Degree::N2     // Tonic: D major
);
// Result: (Degree::N1s, 1) = C# in octave above
```

## Format Converters

### LilyPond Converter (`lilypond.rs`)

Generates LilyPond source code (`.ly` files) for professional music engraving.

**Output format:** Text-based LilyPond notation
```lilypond
\relative c' {
  \key c \major
  \time 4/4
  d4 e4 fs4 g4 a4 b4 cs'4 d4
}
```

**Key features:**
- Uses shared transposition logic
- Handles tuplets with proper `\tuplet` notation
- Supports ties, slurs, and barlines
- Template-based output generation

### VexFlow Converter (`vexflow.rs`)

Generates VexFlow-compatible JSON for web-based music rendering.

**Output format:** JSON structures
```json
{
  "notes": [
    {"Note": {"keys": ["d/4"], "duration": "q"}},
    {"Note": {"keys": ["e/4"], "duration": "q"}},
    {"Note": {"keys": ["cs/5"], "duration": "q"}}
  ],
  "key_signature": "D"
}
```

**Key features:**
- Uses shared transposition logic  
- VexFlow-specific octave numbering (4 = middle C)
- Handles tuplets, ties, and beaming
- Direct JSON output for web rendering

## Common Patterns

### 1. Transposition Flow
```rust
// Both converters follow this pattern:
let (transposed_degree, adjusted_octave) = transpose_degree_with_octave(
    beat_element.degree.unwrap(),
    beat_element.octave.unwrap(), 
    current_tonic
);
```

### 2. Format-Specific Rendering
```rust
// LilyPond: degree + octave → text
let lily_note = format!("{}{}", degree_to_lily_name(degree), octave_to_marks(octave));

// VexFlow: degree + octave → JSON
let vexflow_key = format!("{}/{}", degree_to_note_name(degree), octave + 4);
```

### 3. Rhythm Handling
Both converters use FSM-calculated `tuplet_duration` values and convert them to format-specific duration representations.

## Usage Examples

### Converting FSM Output

```rust
use notation_parser::converters::{lilypond, vexflow};

// LilyPond conversion
let lily_output = lilypond::convert_elements_to_lilypond_src(
    &fsm_elements, 
    &metadata, 
    Some(&source_text)
)?;

// VexFlow conversion  
let vexflow_staves = vexflow::convert_elements_to_staff_notation(
    &fsm_elements, 
    &metadata
)?;
```

### Direct Transposition

```rust
use notation_parser::converters::transposition::transpose_degree_with_octave;

// Transpose scale degree 7 in D major
let (result_degree, result_octave) = transpose_degree_with_octave(
    Degree::N7,    // Scale degree 7
    0,             // Middle octave
    Degree::N2     // D major tonic
);
// Returns: (Degree::N1s, 1) = C# one octave up
```

## Testing

The shared transposition logic includes comprehensive unit tests:

```bash
cargo test transposition
```

Test cases verify:
- Correct transposition for all major/minor keys
- Proper octave wrapping behavior
- Edge cases with extreme accidentals
- Consistency between format converters

## Design Principles

1. **DRY (Don't Repeat Yourself)**: Musical logic shared between formats
2. **Single Responsibility**: Each converter handles one output format
3. **Consistency**: Same musical results across all formats
4. **Extensibility**: Easy to add new notation formats
5. **Testability**: Transposition logic can be tested independently

## Adding New Converters

To add support for a new notation format:

1. Create a new converter file (e.g., `musicxml.rs`)
2. Use shared transposition utilities from `transposition.rs`  
3. Implement format-specific rendering logic
4. Add exports to `mod.rs`
5. Follow existing patterns for rhythm and metadata handling

The shared transposition logic ensures consistency across all formats while keeping format-specific concerns separate.