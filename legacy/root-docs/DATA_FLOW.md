# Musical Music-Text - Data Flow Architecture

This document explains the complete data flow pipeline from raw text input to rendered musical output.

## Overview Pipeline

```
Raw Text → Pest → Parse Tree → parser.rs → AST → spatial_parser.rs → Enhanced AST → FSM → Renderers → Output
```

## Stage-by-Stage Breakdown

### 1. **Raw Text Input**
```
S R G M P
  _____
```
- User types musical notation with spatial layout
- Multiple lines: content line, annotation lines, lyrics
- Original character positions preserved

### 2. **Pest Parser Generator**
**Input:** Raw text  
**Output:** Parse tree (structured tokens)  
**Responsibility:** Tokenization and grammar matching

```rust
// Pest produces structured token tree:
Rule::document [
  Rule::stave [
    Rule::content_line [
      Rule::measure [
        Rule::beat [ Rule::beat_item[Rule::pitch["S"]] ]
        Rule::beat [ Rule::beat_item[Rule::pitch["R"]] ] 
        Rule::beat [ Rule::beat_item[Rule::pitch["G"]] ]
        Rule::beat [ Rule::beat_item[Rule::pitch["M"]] ]
        Rule::beat [ Rule::beat_item[Rule::pitch["P"]] ]
      ]
    ],
    Rule::lower_line [
      Rule::beat_grouping["_____"]
    ]
  ]
]
```

**Key Features:**
- **No code generation** - uses procedural macros at compile time
- **Zero runtime cost** - compiled to optimized parsing functions
- **Position tracking** - preserves original character positions for spatial analysis
- **Grammar-driven** - defined in `grammar/notation.pest`

### 3. **Our Parser (`src/parser.rs`)**
**Input:** Pest parse tree  
**Output:** Musical AST  
**Responsibility:** Convert tokens to semantic musical structures

```rust
// Converts parse tree to musical AST:
Document {
  staves: [
    Stave {
      content_line: ContentLine {
        measures: [
          Measure {
            beats: [
              Beat { elements: [Pitch("S", pos:0)] },
              Beat { elements: [Pitch("R", pos:2)] },
              Beat { elements: [Pitch("G", pos:4)] },
              Beat { elements: [Pitch("M", pos:6)] },
              Beat { elements: [Pitch("P", pos:8)] }
            ]
          }
        ]
      },
      lower_lines: [
        AnnotationLine {
          items: [BeatGrouping("_____", pos:2-7)]
        }
      ]
    }
  ]
}
```

**Key Features:**
- **Semantic understanding** - knows what beats, pitches, measures mean
- **Position preservation** - maintains character positions from original text
- **Multi-line support** - handles content lines, annotation lines, lyrics
- **Unified beat structure** - no more delimited/undelimited distinction
- **Clean separation** - pure parsing logic only, spatial analysis handled separately

### 4. **Spatial Analyzer (`src/spatial_parser.rs`)**
**Input:** Complete Document from AST Builder  
**Output:** Enhanced AST with spatial correlations  
**Responsibility:** Correlate multi-line annotations with content line notes

**Three Main Functions:**

#### 4a. **Slur Analysis** (`spatial_parser::analyze_slurs`)
- **Input**: `upper_lines` with underscore sequences  
- **Process**: Maps slur segments to note positions using spatial correlation via `find_slur_segments()`
- **Output**: Notes marked with slur information

#### 4b. **Octave Assignment** (`spatial_parser::assign_octave_markers`)  
- **Input**: `upper_lines` + `lower_lines` with octave markers (`.`, `:`, `*`, `'`)
- **Process**: Two-phase spatial correlation:
  - Phase 1: Direct position matching (exact column alignment)
  - Phase 2: Nearest neighbor assignment for unmatched markers
- **Conversion**: `.`→±1, `:`→±2, `*`→±3, `'`→±4 octaves via `octave_marker_to_number()`
- **Output**: Notes with assigned octave values via `apply_octave_assignments()`

#### 4c. **Syllable Assignment** (`spatial_parser::assign_syllables_to_notes`)
- **Input**: `lyrics_lines` with syllable sequences
- **Process**: Assigns syllables to singable notes in order, honoring slur markings
- **Output**: Notes with attached syllable information

```rust
// Example: Input with octave markers
// . :           <- upper_lines (octave markers)  
// S R G M       <- content_line (notes)
// .   :         <- lower_lines (octave markers)
// la ti do re   <- lyrics_lines (syllables)

// After spatial analysis:
Beat { elements: [Pitch{value:"S", octave:1, syllable:"la"}] },   // Upper dot + first syllable
Beat { elements: [Pitch{value:"R", octave:2, syllable:"ti"}] },   // Upper colon + second syllable  
Beat { elements: [Pitch{value:"G", octave:-1, syllable:"do"}] },  // Lower dot + third syllable
Beat { elements: [Pitch{value:"M", octave:0, syllable:"re"}] }    // No marker + fourth syllable
```

**Algorithm:**
1. **Calculate note positions** in content line (column-based)
2. **Extract annotation positions** from upper/lower/lyrics lines
3. **Phase 1**: Match annotations to notes at exact same positions
4. **Phase 2**: Assign remaining annotations to nearest unassigned notes
5. **Apply assignments** to enhance the Document in-place

**Key Features:**
- **Column-based alignment** - uses original character positions
- **Multiple underlines** - can handle complex spatial layouts
- **Beat regrouping override** - overrides grammar's natural beat boundaries
- **Modular separation** - separated from parser.rs for cleaner architecture
- **Dedicated spatial logic** - focused solely on annotation line processing

**Module Architecture:**
- **`parser.rs`**: Pure pest-to-AST conversion, calls spatial_parser functions
- **`spatial_parser.rs`**: All spatial analysis functions (slur, octave, syllable analysis)
- **Clean interface**: Parser calls `spatial_parser::analyze_slurs()`, `assign_octave_markers()` etc.

### 5. **Rhythm FSM (`src/rhythm_fsm.rs`)**
**Input:** Spatially-grouped AST  
**Output:** FSM Items with rhythm analysis  
**Responsibility:** Add temporal/rhythmic intelligence

```rust
// Before FSM:
Beat { elements: [Pitch("R"), Pitch("G"), Pitch("M")] }

// After FSM:
Item::Beat(Beat {
  divisions: 3,                    // Total subdivisions
  elements: [
    BeatElement { 
      event: Note { degree: R, octave: 0 },
      subdivisions: 1,             // This note's duration
      duration: Fraction(1, 3),    // 1/3 of beat
      tuplet_duration: Fraction(1, 4),  // Display as quarter note
      tuplet_display_duration: Some(Fraction(1, 4))
    },
    // ... similar for G and M
  ],
  is_tuplet: false,               // 3 is power of 2? No, wait...
  tuplet_ratio: None              // Regular beat, no tuplet
})
```

**Core Responsibilities:**
- **Subdivision counting** - process dash extensions (`1-2` → Note1(2 subdivisions), Note2(1))
- **Tuplet detection** - power-of-2 check (`5 & (5-1) != 0` → tuplet)  
- **Duration calculation** - fractional math for precise timing
- **Breath mark handling** - breaks extension chains (`1- '-2` → extended note, break, rest + note)
- **Tie detection** - cross-beat note connections
- **Pass-through items** - barlines, breath marks as separate items

**Key Features:**
- **Fractional arithmetic** - never floating point, always precise fractions
- **Tuplet ratio calculation** - (divisions, next_lower_power_of_2)
- **Cross-beat tie tracking** - remembers pitches that need tying
- **Stateless per-beat** - processes each beat independently for rhythm

### 6. **Renderers (`src/renderers/`)**
**Input:** FSM Items with rhythm analysis  
**Output:** Format-specific musical notation  
**Responsibility:** Generate final output formats

#### VexFlow Renderer
```rust
// FSM Item → VexFlow JSON
Item::Beat(beat) → {
  "notes": [
    {"keys": ["d/4"], "duration": "q"},  // Quarter note D
    {"keys": ["e/4"], "duration": "q"},  // Quarter note E  
    {"keys": ["f/4"], "duration": "q"}   // Quarter note F
  ],
  "beam": true  // Beam these notes together
}
```

#### LilyPond Renderer  
```rust
// FSM Item → LilyPond source
Item::Beat(beat) → "d4 e4 f4"  // Three quarter notes
```

**Key Features:**
- **Beat-based beaming** - notes within beats get beamed together
- **Tuplet notation** - `\tuplet 5/4 { c8 d8 e16 }` for complex rhythms
- **Duration mapping** - subdivisions → standard note durations
- **Tie generation** - cross-beat ties for extended notes

## Data Structure Evolution

### AST Beat (After Spatial Analysis)
```rust
Beat {
  elements: Vec<BeatElement>,      // Notes, dashes, slurs
  divisions: None,                 // Not yet calculated
  is_tuplet: None,                // Not yet determined
  tuplet_ratio: None              // Not yet calculated
}
```

### FSM Beat (After Rhythm Analysis)
```rust
Beat {
  divisions: 5,                    // Total subdivisions calculated
  elements: Vec<BeatElement>,      // With subdivision counts
  is_tuplet: true,                // Tuplet detected
  tuplet_ratio: Some((5, 4)),     // 5-tuplet ratio
}

BeatElement {
  event: Note/Rest,               // Musical event
  subdivisions: 2,                // How many subdivisions this occupies
  duration: Fraction(2, 5),       // Exact beat fraction
  tuplet_duration: Fraction(1, 8), // Display duration
}
```

## Error Handling

Each stage can produce specific error types:

- **Pest**: Grammar syntax errors, unexpected tokens
- **Parser**: Semantic conversion errors, invalid musical structures  
- **Spatial Analyzer**: Misaligned underlines, position conflicts
- **FSM**: Invalid rhythm patterns, impossible tuplets
- **Renderers**: Unsupported notation, rendering failures

## Performance Characteristics

- **Pest**: Zero runtime overhead (compile-time generation)
- **Parser**: Single-pass AST construction  
- **Spatial Analyzer**: O(n×m) where n=notes, m=underlines
- **FSM**: O(beats) - stateless per-beat processing
- **Renderers**: O(elements) - linear conversion

## Testing Strategy

Each stage can be tested independently:

```rust
// Test Pest grammar
assert_eq!(parse("S R G"), Ok(expected_parse_tree));

// Test parser conversion (parser.rs)
assert_eq!(convert_ast(parse_tree), Ok(expected_ast));

// Test spatial analysis (spatial_parser.rs)
assert_eq!(spatial_parser::analyze_slurs(ast), expected_slurred_ast);
assert_eq!(spatial_parser::assign_octave_markers(ast), expected_enhanced_ast);

// Test rhythm FSM
assert_eq!(rhythm_fsm(beats), expected_fsm_items);

// Test renderers
assert_eq!(render_vexflow(fsm_items), expected_json);
```

This modular architecture allows for targeted testing and debugging at each pipeline stage.

## Complete Pipeline Verification

**Test Function**: `src/parser.rs` - `test_lower_octave_integration()`  
**Command**: `cargo run -- --test-lower-octave`

This test demonstrates the complete data flow pipeline:

1. **Creates Manual AST**: Simulates Document with lower octave markers
2. **Applies Spatial Analysis**: Calls `spatial_parser::assign_octave_markers()` 
3. **Converts through FSM**: Uses `rhythm_fsm::convert_ast_to_fsm_output()`
4. **Generates Output**: Converts FSM to LilyPond notation
5. **Verifies Flow**: Confirms octave values flow correctly through entire pipeline

**Expected Output**:
```
BEFORE octave assignment:
Beat 0: S has octave: 0
Beat 1: R has octave: 0

FSM output:
Item 0: Beat(Beat { divisions: 1, elements: [BeatElement { event: Note { degree: N1, octave: -1 }, ... }] })
Item 1: Beat(Beat { divisions: 1, elements: [BeatElement { event: Note { degree: N2, octave: -2 }, ... }] })

LilyPond generation from FSM:
N1 (octave -1) -> LilyPond: c,
N2 (octave -2) -> LilyPond: d,,

AFTER octave assignment:
Beat 0: S has octave: -1
Beat 1: R has octave: -2
```

This verifies the complete data flow: **Raw AST → Spatial Analysis → FSM → LilyPond Output** with octave values correctly preserved throughout the entire pipeline.