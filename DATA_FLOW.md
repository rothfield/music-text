# Notation Parser Data Flow Documentation

This document traces the complete data flow through the notation parser, from raw text input to final output formats.

## Overview

The parser transforms textual music notation through multiple phases, each adding structure and semantic meaning:

```
Raw Text → Tokens → Spatial Analysis → Musical Structure → Output Formats
```

## Detailed Data Flow

### Phase 1: Lexical Analysis

**Input**: Raw text notation
**Module**: `handwritten_lexer.rs`
**Function**: `tokenize_with_handwritten_lexer()`

```rust
// Example input:
"G -P | S\n_________"

// Becomes tokens:
[
    Token { type: "PITCH", value: "G", line: 1, col: 0 },
    Token { type: "PITCH", value: "-P", line: 1, col: 2 },
    Token { type: "BARLINE", value: "|", line: 1, col: 5 },
    Token { type: "PITCH", value: "S", line: 1, col: 7 },
    Token { type: "SYMBOLS", value: "_________", line: 2, col: 0 },
]
```

**Key Operations**:
- Character-by-character tokenization
- Position tracking (line/column)
- Token type classification (PITCH, BARLINE, SYMBOLS, etc.)

**Output**: `Vec<Token>` with position information

### Phase 2: Metadata Extraction

**Module**: `lexer.rs`
**Function**: `parse_metadata()`

```rust
// Input: All tokens
// Output: (Metadata, Vec<Token>) - metadata extracted, remaining tokens
```

**Key Operations**:
- Extract title, directives, key signatures
- Detect notation system (Sargam, Western, Number)
- Return remaining non-metadata tokens

### Phase 3: Spatial Analysis

**Module**: `spatial_analysis.rs`
**Function**: `attach_floating_elements()`

**Purpose**: Process spatial/visual elements like slur overlines that span multiple tokens.

```rust
// Before spatial analysis:
Tokens: [G, -P, |, S] + [_________]

// After spatial analysis:
Hierarchical Nodes: [
    Node { type: "note", value: "G", ... },
    Node { type: "note", value: "-P", ... },
    Node { type: "barline", value: "|", ... },
    Node { type: "note", value: "S", ... },
]
```

**Key Operations**:
- Convert tokens to preliminary Node structures
- Create spatial mappings for overlines/underlines
- Build hierarchical relationships

**Output**: `Vec<Node>` with spatial structure

### Phase 4: Slur Attribution

**Module**: `spatial_analysis.rs`
**Function**: `apply_slurs_and_regions_to_nodes()`

**Purpose**: Analyze spatial overlines and mark affected nodes with slur attributes.

```rust
// Input: Hierarchical nodes + overline "_________"
// Processing: Maps overline position to underlying notes

// Output: Nodes with boolean slur attributes
[
    Node { type: "note", value: "G", slur_start: Some(true), ... },
    Node { type: "note", value: "-P", slur_start: None, slur_end: None, ... },
    Node { type: "barline", value: "|", ... },
    Node { type: "note", value: "S", slur_end: Some(true), ... },
]
```

**Key Operations**:
- Map overline spans to underlying notes
- Set `slur_start: true` on first affected note
- Set `slur_end: true` on last affected note
- Handle beat brackets similarly

**Critical Insight**: At this stage, slurs exist as **boolean attributes**, not as discrete tokens.

### Phase 5: Musical Structure (FSM)

**Module**: `rhythm_fsm.rs`  
**Function**: `group_nodes_with_fsm()`

**Purpose**: Group individual notes into musical beats and measures.

```rust
// Input: Flat list of nodes
[Note(G), Note(-P), Barline(|), Note(S)]

// Output: Structured hierarchy with beats
[
    Beat {
        nodes: [Note(G), Note(-P)],
        divisions: 2,
        ...
    },
    Barline(|),
    Beat {
        nodes: [Note(S)],
        divisions: 1,
        ...
    }
]
```

**Key Operations**:
- Identify beat boundaries
- Calculate rhythmic divisions
- Group notes into beat structures
- Preserve non-beat elements (barlines, etc.) at top level

### Phase 6: Slur Token Conversion

**Module**: `lib.rs`
**Function**: `convert_slur_attributes_to_tokens()`

**Purpose**: Convert boolean slur attributes into explicit SLUR_START/SLUR_END nodes.

```rust
// Before conversion:
Beat {
    nodes: [
        Note { value: "G", slur_start: true, ... },
        Note { value: "S", slur_end: true, ... }
    ]
}

// After conversion:
Beat {
    nodes: [
        Node { type: "SLUR_START", value: "(", ... },
        Note { value: "G", slur_start: None, ... },
        Note { value: "S", slur_end: None, ... },
        Node { type: "SLUR_END", value: ")", ... }
    ]
}
```

**Key Operations**:
- Scan for nodes with `slur_start: true`
- Insert `SLUR_START` node before the marked note
- Scan for nodes with `slur_end: true` 
- Insert `SLUR_END` node after the marked note
- Clear the boolean attributes

**Critical Insight**: This is where slur **attributes become tokens**. The final AST contains explicit slur nodes that converters can process.

### Phase 7: Lyrics Distribution

**Module**: `lyrics.rs`
**Function**: `distribute_syllables_to_notes()`

**Purpose**: Attach lyric syllables to appropriate notes.

```rust
// Input: Structured nodes + lyrics lines
// Output: Notes with syl: Option<String> populated
```

**Key Operations**:
- Parse lyrics from separate text lines
- Distribute syllables to notes
- Handle melisma (multiple notes per syllable)
- Respect slur boundaries

### Phase 8: Document Creation

**Output**: Final `Document` structure ready for converters:

```rust
Document {
    metadata: Metadata { /* title, key, etc. */ },
    nodes: Vec<Node>,  // Fully processed hierarchical structure
    notation_system: Some("Sargam"),
}
```

## Converter Pathways

The final `Document` feeds into multiple output converters:

### VexFlow Conversion
**Module**: `vexflow_fsm_converter.rs`
**Function**: `convert_fsm_to_vexflow()`

- Processes explicit `SLUR_START`/`SLUR_END` nodes
- Creates VexFlow JSON with slur rendering instructions
- Handles cross-barline slurs

### LilyPond Conversion  
**Module**: `lilypond_converter.rs`
**Function**: `convert_to_lilypond()`

- Converts nodes to LilyPond syntax
- Processes slur tokens into `( )` syntax
- Handles pitch conversion between notation systems

## Key Design Insights

### 1. Two-Phase Slur Processing

Slurs undergo a **two-stage transformation**:
1. **Spatial → Attributes**: Overlines become boolean flags
2. **Attributes → Tokens**: Boolean flags become explicit nodes

This design allows:
- Spatial analysis to work with visual layout
- Converters to work with discrete tokens
- Clean separation between parsing phases

### 2. Hierarchical Structure Evolution

The data structure evolves from flat to hierarchical:
- **Tokens**: Flat list with position info
- **Spatial Nodes**: Flat list with spatial relationships
- **Structured Nodes**: Hierarchical with beats and musical groupings

### 3. Position Information Preservation

Position data (`row`, `col`) is preserved throughout all phases, enabling:
- Error reporting with exact locations
- Spatial analysis of overlays
- Debugging and visualization

## AST Refactoring Implications

For the planned enum-based AST refactoring:

1. **SlurStart/SlurEnd variants are required** - they exist as real nodes in the final tree
2. **Beat structure must be preserved** - the FSM creates meaningful beat groupings
3. **Position information is critical** - every enum variant needs location data
4. **Two-phase slur processing should be maintained** - it cleanly separates concerns

## Testing Strategy

Each phase can be tested independently:
- **Lexer**: Input text → Expected tokens
- **Spatial**: Tokens + overlines → Nodes with slur attributes  
- **FSM**: Flat nodes → Structured beats
- **Token conversion**: Slur attributes → Explicit slur nodes
- **End-to-end**: Full notation → Multiple output formats

## Performance Considerations

- **Memory usage**: Multiple intermediate representations
- **Processing time**: Each phase scans the entire structure
- **Cache locality**: Frequent node transformations

The enum-based refactoring should address memory efficiency while preserving the logical phase separation.