# Notation Parser Data Flow Documentation - V2 Architecture

This document traces the complete data flow through the V2 notation parser, from raw text input to final output formats.

## Overview - V2 Complete Architecture

The V2 parser transforms textual music notation through a **clean-slate FSM-centric pipeline**, with mathematical precision and type safety:

```
Raw Text → V2 Parser → ParsedElements → FSM V2 → FSM Output → Dual Converters → Output Formats
                  ↓           ↓              ↓             ↓
            Type-Safe    Mathematical    Clean FSM    Template-Based
             Enums      Fractions (2,3)   Tuplets     Generation
```

## V2 Architecture - Major Innovations

### **1. AI-First Documentation System**
- **886+ lines** of LLM-focused documentation
- Domain knowledge encoding specifically for AI consumption
- Critical tuplet rules and rhythm system explanations for LLMs

### **2. Clean-Slate FSM-Centric Design**
- **FSM as architectural core** - rhythm processing drives everything
- **Mathematical precision** - fractional arithmetic throughout, zero floating point
- **Type-safe data structures** - eliminated impossible states with compiler enforcement

### **3. Dual Converter Rewrites**
- **VexFlow V2**: Direct FSM → VexFlow, no hierarchical dependencies
- **LilyPond V2**: Mustache template system replacing string concatenation

## V2 Detailed Data Flow - Complete Examples

### Phase 1: V2 Parser (Type-Safe ParsedElements)

**Input**: Raw text notation
**Module**: `models_v2.rs` + `unified_parser_v2()`
**Example**: `"1-2-3 -4#"` (Number notation with rhythm and sharp)

```rust
// V1 OLD: Monolithic Node with many Optional fields
pub struct Node {
    pub node_type: String,           // "PITCH", "REST", "DASH"
    pub pitch_code: Option<PitchCode>, // Only for notes
    pub octave: Option<i8>,          // Only for notes  
    pub syl: Option<String>,         // Only for lyrics
    pub duration_fraction: Option<String>, // String: "1/4"
}

// V2 NEW: Type-Safe ParsedElement Enums
pub enum ParsedElement {
    Note { 
        pitch_code: PitchCode,           // Always present
        octave: i8,                      // Always present
        value: String,                   // Original text
        position: Position,              // Structured position
        children: Vec<ParsedChild>,      // Syllables, ornaments
        duration: Option<(usize, usize)>, // Mathematical fractions
    },
    Dash { 
        pitch_code: Option<PitchCode>, // Inherited from preceding note
        octave: Option<i8>,
        position: Position,
        duration: Option<(usize, usize)>,
    },
    Rest { /* structured fields */ },
    Barline { /* structured fields */ },
}

// Input: "1-2-3 -4#" 
// V2 ParsedElements Output:
[
    Note { pitch_code: N1, octave: 0, value: "1", position: Position(0,0), children: [] },
    Dash { pitch_code: Some(N1), octave: Some(0), position: Position(0,1) },
    Note { pitch_code: N2, octave: 0, value: "2", position: Position(0,2), children: [] },
    Dash { pitch_code: Some(N2), octave: Some(0), position: Position(0,3) },
    Note { pitch_code: N3, octave: 0, value: "3", position: Position(0,4), children: [] },
    Space { position: Position(0,5) },  // Beat separator
    Dash { pitch_code: None, octave: None, position: Position(0,7) }, // Rest
    Note { pitch_code: N4, octave: 0, value: "4#", position: Position(0,8), children: [Sharp] },
]
```

**V2 Key Innovations**:
- **Type Safety**: Impossible states eliminated (no Optional fields for required data)
- **Mathematical Precision**: Tuple fractions `(2,3)` not string `"2/3"`
- **Structured Children**: `ParsedChild::Syllable`, `ParsedChild::Ornament` with positioning

### Phase 2: V2 FSM (Mathematical Rhythm Processing)

**Module**: `rhythm_fsm_v2.rs` + `rhythm_fsm_v2_clean.rs`
**Function**: `process_rhythm_v2_clean()` 
**Example**: Continue with `"1-2-3 -4#"` → **5/4 tuplet + regular beat**

```rust
// V1 OLD FSM: Complex state machines, string durations, heuristics
// V2 NEW FSM: Clean mathematical approach with power-of-2 tuplet detection

// Input: ParsedElements from Phase 1
// Processing: Group into beats, detect tuplets mathematically

// Beat 1: "1-2-3" → divisions=5 (NOT power of 2 = tuplet)
BeatV2 {
    divisions: 5,  // Total subdivisions
    elements: [
        ElementV2 { element: Note(N1), subdivisions: 2 },  // "1-" (extended)
        ElementV2 { element: Note(N2), subdivisions: 2 },  // "2-" (extended) 
        ElementV2 { element: Note(N3), subdivisions: 1 },  // "3" (single)
    ],
    // V2 INNOVATION: Mathematical tuplet detection
    is_tuplet: true,  // 5 is NOT power of 2 (2,4,8,16...)
    tuplet_ratio: (5, 4),  // 5 notes in place of 4 (next lower power of 2)
}

// Beat 2: "-4#" → divisions=2 (IS power of 2 = regular beat)
BeatV2 {
    divisions: 2,
    elements: [
        ElementV2 { element: Rest, subdivisions: 1 },      // "-" (leading dash = rest)
        ElementV2 { element: Note(N4#), subdivisions: 1 }, // "4#"
    ],
    is_tuplet: false,  // 2 IS power of 2
    tuplet_ratio: None,
}
```

**V2 FSM Key Innovations**:
- **Power-of-2 Tuplet Detection**: `is_tuplet = (divisions & (divisions-1)) != 0`
- **Mathematical Duration Calculation**: Uses CRITICAL tuplet rule from CLAUDE.md
- **Clean Fractional Arithmetic**: No floating point, only fractions
- **Subdivision Tracking**: Precise counting of note extensions

### Phase 2b: V2 Duration Calculation (CRITICAL TUPLET RULE)

**Module**: Applied in FSM processing
**Example**: Calculate actual durations for "1-2-3" 5/4 tuplet

```rust
// CRITICAL RULE from CLAUDE.md:
// 1. Find next lower power of 2: 5 → 4 (since 4 < 5 < 8)
// 2. Calculate as if divisions=4, then wrap in tuplet 5/4

// Each unit = 1/4 ÷ 4 = 1/16
// Note 1: 2 subdivisions × (1/16) = 1/8 → eighth note  
// Note 2: 2 subdivisions × (1/16) = 1/8 → eighth note
// Note 3: 1 subdivision × (1/16) = 1/16 → sixteenth note

// Final V2 FSM Output with calculated durations:
OutputItemV2::Beat(BeatV2 {
    elements: [
        ElementV2 { 
            element: Note(N1), 
            subdivisions: 2,
            duration: (2, 5),      // Actual fraction of beat: 2/5  
            tuplet_duration: (1, 8), // Visual duration: eighth note
        },
        ElementV2 { 
            element: Note(N2), 
            subdivisions: 2,
            duration: (2, 5),      // Actual fraction: 2/5
            tuplet_duration: (1, 8), // Visual duration: eighth note  
        },
        ElementV2 { 
            element: Note(N3), 
            subdivisions: 1,
            duration: (1, 5),       // Actual fraction: 1/5
            tuplet_duration: (1, 16), // Visual duration: sixteenth note
        },
    ],
    tuplet_ratio: (5, 4),  // Tuplet: 5 in place of 4
})
```

### Phase 3: V2 Dual Converters (Complete Rewrites)

**Both converters completely rewritten for V2 FSM-centric architecture**

#### 3a: V2 LilyPond Converter (Template-Based)

**Module**: `lilypond_converter_v2.rs` + `lilypond_templates.rs`
**Function**: `convert_fsm_output_to_lilypond()`
**Innovation**: **Mustache template system** replacing string concatenation

```rust
// V1 OLD: String building approach
let mut output = String::new();
output.push_str("\\fixed c' {\n");
for note in notes {
    output.push_str(&format!("  {}{}", note.pitch, note.duration));
}
output.push_str("}\n");

// V2 NEW: Template-based with structured context
let context = TemplateContext::builder()
    .title("Test Song")
    .staves("\\tuplet 5/4 { c8 d8 e16 } r4 fs4")  // From FSM
    .build();

let template = mustache::compile_str(STANDARD_TEMPLATE);
let rendered = template.render_to_string(&context)?;

// Template: standard.ly.mustache
\\version "{{version}}"
\\header { 
  {{#title}}title = "{{{title}}}"{{/title}}
}
\\score {
  \\new Staff {
    \\relative c' {
      {{{staves}}}  // FSM output inserted here
    }
  }
}
```

**V2 LilyPond Output for "1-2-3 -4#"**:
```lilypond
\version "2.24.0"
\score {
  \new Staff {
    \relative c' {
      \tuplet 5/4 { c8 d8 e16 } r4 fs4
    }
  }
}
```

#### 3b: V2 VexFlow Converter (FSM-Direct)

**Module**: `vexflow_converter_v2.rs`  
**Function**: `convert_fsm_output_to_vexflow_v2()`
**Innovation**: **Direct FSM processing**, no hierarchical document structure

```rust
// V1 OLD: Expected hierarchical LINE/BEAT structure
document.nodes → looking for node_type == "LINE" → FAILED (empty output [])

// V2 NEW: Direct FSM → VexFlow mapping
pub fn convert_fsm_output_to_vexflow_v2(
    fsm_output: &[OutputItemV2], 
    metadata: &Metadata
) -> Result<Vec<VexFlowStave>, ConversionError> {
    // Process beats directly, no document hierarchy needed
    for item in fsm_output {
        match item {
            OutputItemV2::Beat(beat) => {
                if beat.is_tuplet {
                    // Use FSM-calculated tuplet durations directly
                    for element in &beat.elements {
                        let duration = map_tuplet_duration(element.tuplet_duration);
                        // Create VexFlow note with correct duration
                    }
                } else {
                    // Regular beat processing
                }
            }
        }
    }
}
```

**V2 VexFlow Output for "1-2-3 -4#"**:
```json
[{
  "notes": [
    {"keys": ["c/4"], "duration": "8"},   // Note 1: eighth 
    {"keys": ["d/4"], "duration": "8"},   // Note 2: eighth
    {"keys": ["e/4"], "duration": "16"},  // Note 3: sixteenth
    {"keys": ["r/4"], "duration": "q"},   // Rest: quarter
    {"keys": ["fs/4"], "duration": "q"}   // Note 4#: quarter
  ],
  "tuplet": {"ratio": [5, 4], "notes": [0, 1, 2]}  // First 3 notes in 5/4 tuplet
}]
```

### Phase 4: V2 Web Integration & WASM Success

**Major Innovation**: Complete WASM integration with dual output system

#### 4a: WASM Build Success
```bash
wasm-pack build --target web --out-dir webapp/pkg
# ✅ Successfully built with V2 system - 32 warnings but clean build
# Generated: webapp/pkg/ with V2 VexFlow converter integration
```

#### 4b: Web UI Dual Output System  
**Module**: `webapp/server.js` + `webapp/public/js/main.js`

```javascript  
// V2 Web Integration - Dual output support:
// 1. VexFlow rendering (live interactive)
// 2. LilyPond SVG generation (compact for web)

// Input: "1-2-3 -4#" via web UI
// Processing: Uses WASM V2 parser + FSM + dual converters
// Output 1: VexFlow JSON → Interactive staff rendering
// Output 2: LilyPond → Compact SVG for comparison

// Web UI Flow:
Raw Input → WASM V2 Parser → FSM Output → {
    VexFlow Converter → Interactive Staff
    LilyPond Converter → Server SVG Generation → Compact Display  
}
```

#### 4c: Small LilyPond SVG Optimization
**Module**: `src/templates/standard.ly.mustache`
**Innovation**: **Compact paper settings** optimized for web display

```lilypond
% V2 Template optimization for web UI
\\paper {
  indent = 0\\mm
  top-margin = 0.5\\mm
  bottom-margin = 0.5\\mm  
  paper-height = 50\\mm      % Small height for web
  paper-width = 200\\mm      % Compact width
  page-breaking = #ly:one-page-breaking
}
```

**Result**: Small, web-optimized SVG output perfect for side-by-side display with VexFlow

## V2 Architecture Summary - Complete Transformation

### **Complete Data Flow: "1-2-3 -4#" Example**

```
INPUT: "1-2-3 -4#"
   ↓ V2 Parser (Type-Safe)
[Note{N1}, Dash{N1}, Note{N2}, Dash{N2}, Note{N3}, Space, Dash{Rest}, Note{N4#}]
   ↓ V2 FSM (Mathematical) 
[Beat{divisions:5, tuplet:5/4}, Beat{divisions:2, regular}]
   ↓ V2 Converters (Dual Rewrites)
LilyPond: "\tuplet 5/4 { c8 d8 e16 } r4 fs4"
VexFlow:  [{"notes": [{"keys": ["c/4"], "duration": "8"}, ...], "tuplet": {"ratio": [5,4]}}]
   ↓ WASM + Web UI
Interactive Staff + Compact SVG (side-by-side display)
```

### **V2 Major Innovations Summary**

1. **AI-First Documentation (886+ lines)**
   - Domain knowledge encoding for LLM consumption
   - RHYTHM_SYSTEM.md: Critical LLM reference 
   - System prompts updated so LLMs understand notation

2. **Type-Safe Data Structures**
   - Monolithic Node → ParsedElement enums
   - Impossible states eliminated
   - Mathematical fractions (2,3) not strings "2/3"

3. **Clean-Slate FSM Architecture**
   - Power-of-2 tuplet detection
   - CRITICAL tuplet duration rule 
   - Fractional arithmetic throughout

4. **Complete Converter Rewrites**
   - VexFlow V2: Direct FSM processing
   - LilyPond V2: Mustache template system
   - No V1 technical debt inheritance

5. **Enhanced Lyrics System**
   - Flat string → structured ParsedChild::Syllable
   - Spatial positioning with distance tracking
   - Multiple syllables per note support

6. **Web Integration Success**
   - WASM VexFlow working
   - Small LilyPond SVG optimization
   - Dual output system (VexFlow + LilyPond)

### **Legacy V1 System (Archived)**

*The following sections document the legacy V1 system architecture, preserved for reference. The V2 system above represents the current active implementation.*

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