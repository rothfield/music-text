# Music Notation Parser Pipeline Architecture

## Core Insight: Beat-Centric Processing

**Key Discovery**: Beats are the fundamental abstraction in music notation processing. This insight drives the entire architecture design.

### Why Beats Are Central

1. **Beaming Rules**: Notes are beamed within beats, never across beat boundaries (e.g., in 4/4 time, never beam across beats 2-3)
2. **Rhythmic Grouping**: Musical time is organized around beat subdivisions
3. **Visual Layout**: Music notation visually groups elements by beats
4. **Processing Efficiency**: Beat-centric loops are cleaner and more maintainable

## Three-Stage Pipeline

```
Raw Text → Tokenizer → Vertical Parser → Horizontal Parser → Converters
                            ↑                    ↑
                        spatial             temporal
```

### Stage 1: Tokenizer (`src/tokenizer.rs`, currently `handwritten_lexer.rs`)
- **Input**: Raw text notation
- **Output**: Flat token stream
- **Function**: Tokenizes input into basic elements (PITCH, BARLINE, DASH, etc.)
- **Example**: `"1-2|3"` → `[Token{PITCH,"1"}, Token{DASH,"-"}, Token{PITCH,"2"}, Token{BARLINE,"|"}, Token{PITCH,"3"}]`

### Stage 2: Vertical Parser (`src/vertical_parser.rs`, currently `region_processor.rs`)
- **Input**: Flat token stream
- **Output**: ParsedElements with spatial markup applied
- **Function**: Handles vertical/spatial relationships (slur regions, octaves, syllables)
- **Domain**: Processes markings above and below the main line
- **Example**: Converts underscore patterns to SlurStart/SlurEnd elements

### Stage 3: Horizontal Parser (`src/horizontal_parser.rs`, currently `rhythm_fsm.rs`)
- **Input**: ParsedElements with spatial markup
- **Output**: Beat-structured Items
- **Function**: Groups elements into beats, detects tuplets, handles rhythmic structure
- **Domain**: Processes left-to-right temporal flow (beats, rhythm, time)
- **Example**: Groups notes and dashes into Beat objects with proper subdivisions

### Stage 4: Converters (`src/converters/`)
- **Input**: Beat-structured Items
- **Output**: Target format (LilyPond, VexFlow JS, etc.)
- **Function**: Convert beat-centric representation to target notation system
- **Architecture**: Loop through `Item::Beat` objects, handle other items naturally

## Beat-Centric Architecture Benefits

### Current Implementation (LilyPond Converter)
```rust
for item in elements.iter() {
    match item {
        Item::Beat(beat) => {
            // Process beat as atomic unit
            let beat_notes = convert_beat_to_lilypond(beat, current_tonic)?;
            lilypond_notes.extend(beat_notes);
        },
        Item::Barline(style) => {
            lilypond_notes.push(format!("\\bar \"{}\"", style));
        },
        // Other items handled simply
    }
}
```

### Recommended VexFlow Architecture
```rust
// Instead of complex note/barline mixing:
for item in elements.iter() {
    match item {
        Item::Beat(beat) => {
            // Process beat as atomic unit, generate VexFlow notes
            notes.extend(convert_beat_to_vexflow_notes(beat));
        },
        Item::Barline(style) => {
            // Handle barlines as separate items, not mixed with notes
            barlines.push(VexFlowBarline::new(current_position, style));
        },
    }
}
```

## Final Naming Convention

### Approved Architecture: Spatial/Temporal Division
- `tokenizer` → `vertical_parser` → `horizontal_parser` → `converters`

**Why This Works:**
- **Visual Clarity**: Immediately understandable spatial model
- **Architectural Symmetry**: Vertical (spatial) vs Horizontal (temporal) processing
- **Domain Appropriate**: Music notation is inherently spatial and temporal
- **Self-Documenting**: Anyone reading code understands the vertical/horizontal division

## Implementation Recommendations

### 1. Module Renaming
**Current → New:**
- `src/handwritten_lexer.rs` → `src/tokenizer.rs`
- `src/region_processor.rs` → `src/vertical_parser.rs`  
- `src/rhythm_fsm.rs` → `src/horizontal_parser.rs`

### 2. VexFlow Generator Refactoring
- Adopt beat-centric main loop (like LilyPond converter)
- Separate note generation from barline placement
- Use beat boundaries for proper beaming decisions

### 3. Vertical Parser Enhancement
- Expand beyond slur regions to handle all vertical relationships
- Process octave assignments based on vertical positioning
- Handle syllable alignment for lyrics
- Manage floating element attachment

### 4. Pipeline Consistency
- Ensure all converters use beat-centric loops
- Maintain clean separation between vertical and horizontal processing
- Keep horizontal parser output as the canonical internal representation

## Key Architectural Principles

1. **Beat Centrality**: All musical processing revolves around beat structures
2. **Stage Separation**: Clear boundaries between lexical, spatial, and temporal processing  
3. **FSM Authority**: The rhythmic parser's output is the authoritative musical representation
4. **Converter Simplicity**: Output generators loop through beats, handle other items simply
5. **Beaming Respect**: All operations respect beat boundaries for proper musical notation