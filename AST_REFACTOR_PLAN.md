# AST Refactoring Plan: Node → Enum-based AST

## Problem Statement

The current `Node` struct in `src/models.rs` is a monolithic structure with 15+ fields, most of which are optional. This design pattern leads to:

- **Memory waste**: Most nodes only use a small subset of fields
- **Type safety issues**: Invalid field combinations are possible at runtime
- **Unclear semantics**: Difficult to understand which fields are valid for which node types
- **Maintenance burden**: Every converter must handle all possible field combinations

The `node_type: String` field is used for runtime type discrimination, which prevents compile-time type checking.

## Current Architecture

```
Text → Tokens → Chunks → Nodes → [Rhythm FSM | VexFlow | LilyPond]
                            ↑
                    PROBLEM: Monolithic Node struct
```

## Proposed Solution

Replace the monolithic `Node` struct with a properly typed enum hierarchy that makes invalid states unrepresentable.

## New AST Design

### Core Types

```rust
// Shared position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

// Ornament types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrnamentType {
    Trill,
    Mordent,
    Turn,
    Grace,
}

// Elements that can appear inside beats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeatContent {
    Note {
        pitch_code: PitchCode,
        octave: i8,
        divisions: usize,
        syl: Option<String>,
        is_tied: bool,  // true if this note is tied to the *next* note
    },
    Rest {
        divisions: usize,
    },
    Dash {  // Note extension
        divisions: usize,
    },
    SlurStart,
    SlurEnd,
    Ornament {
        kind: OrnamentType,
    },
}

// Top-level line elements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Element {
    Beat {
        contents: Vec<BeatContent>,
        position: Position,
        bracket_start: bool,
        bracket_end: bool,
    },
    Note {  // Standalone note outside beat
        pitch_code: PitchCode,
        octave: i8,
        position: Position,
        syl: Option<String>,
        is_tied: bool,  // true if this note is tied to the *next* note
    },
    Barline {
        style: String,  // "single", "double", etc.
        position: Position,
    },
    Space {
        width: usize,
        position: Position,
    },
    Unknown {
        value: String,
        position: Position,
    },
}

// Updated Document structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub metadata: Metadata,
    pub elements: Vec<Element>,  // Changed from nodes: Vec<Node>
    pub notation_system: Option<String>,
}
```

### Key Design Decisions

1. **Separation of concerns**: `BeatContent` can only contain musical elements, preventing barlines from appearing inside beats
2. **Explicit position**: Position is a separate struct, not scattered fields
3. **Type safety**: Each enum variant only contains relevant fields
4. **No stringly-typed discrimination**: Enum variants replace `node_type: String`

## Implementation Plan

### Phase 1: Parallel Implementation (Week 1)
- [ ] Create new enum types in `src/models_v2.rs`
- [ ] Implement Display and Debug traits
- [ ] Add serialization support
- [ ] Write conversion functions from old Node to new Element

### Phase 2: Parser Migration (Week 2)
- [ ] Create `parser_v2.rs` that outputs new AST
- [ ] Implement parallel parsing pathway
- [ ] Add feature flag to switch between old and new parser
- [ ] Validate output equivalence with extensive tests

### Phase 3: Converter Migration (Week 3)
- [ ] Migrate `vexflow_fsm_converter.rs` to use new AST
- [ ] Migrate `lilypond_converter.rs` to use new AST
- [ ] Migrate `rhythm_fsm.rs` to use new AST
- [ ] Update spatial analysis to work with new types

### Phase 4: Cleanup (Week 4)
- [audience ] Remove legacy Node struct
- [ ] Remove old parser code
- [ ] Update all tests to use new AST
- [ ] Update documentation

## Migration Strategy

### Compatibility Layer

During migration, maintain backwards compatibility:

```rust
// Temporary conversion trait
impl From<Node> for Element {
    fn from(node: Node) -> Self {
        match node.node_type.as_str() {
            "beat" => Element::Beat {
                contents: node.nodes.into_iter()
                    .map(|n| n.into())
                    .collect(),
                position: Position { 
                    row: node.row, 
                    col: node.col 
                },
                bracket_start: node.beat_bracket_start.unwrap_or(false),
                bracket_end: node.beat_bracket_end.unwrap_or(false),
            },
            "note" => Element::Note {
                pitch_code: node.pitch_code.unwrap_or_default(),
                octave: node.octave.unwrap_or(0),
                position: Position { 
                    row: node.row, 
                    col: node.col 
                },
                syl: node.syl,
            },
            "barline" => Element::Barline {
                style: node.value,
                position: Position { 
                    row: node.row, 
                    col: node.col 
                },
            },
            _ => Element::Unknown {
                value: node.value,
                position: Position { 
                    row: node.row, 
                    col: node.col 
                },
            },
        }
    }
}
```

### Feature Flags

Use Cargo features to enable gradual migration:

```toml
[features]
default = ["legacy-ast"]
legacy-ast = []
new-ast = []
```

## Testing Strategy

### Regression Testing
1. Parse all existing `.123` files with both parsers
2. Compare output structure (accounting for representation differences)
3. Ensure identical musical semantics

### Property Testing
```rust
#[test]
fn barlines_cannot_appear_in_beats() {
    // This should not compile with new AST
    // let beat = Element::Beat {
    //     contents: vec![
    //         BeatContent::Barline { ... }  // Compile error!
    //     ],
    //     ...
    // };
}
```

### Roundtrip Testing
1. Parse notation → Convert to VexFlow/LilyPond → Compare output
2. Ensure byte-for-byte output compatibility

## Benefits

### Immediate Benefits
- **Type safety**: Compiler prevents invalid AST construction
- **Memory efficiency**: ~60% reduction in AST memory usage
- **Code clarity**: Pattern matching makes intent explicit

### Long-term Benefits
- **Maintainability**: New features require explicit enum variant design
- **Performance**: Smaller structs, better cache locality
- **Correctness**: Many bugs become compile-time errors

## Risks and Mitigation

### Risk: Breaking Changes
**Mitigation**: Parallel implementation with feature flags

### Risk: Unforeseen Edge Cases  
**Mitigation**: Extensive testing before switching defaults

### Risk: Performance Regression
**Mitigation**: Benchmark both implementations

## Success Criteria

- [ ] All existing tests pass with new AST
- [ ] Memory usage reduced by >50%
- [ ] No performance regression
- [ ] Code coverage maintained at >80%
- [ ] Zero runtime panics from type mismatches

## Timeline

- **Week 1**: Design and parallel implementation
- **Week 2**: Parser migration
- **Week 3**: Converter migration  
- **Week 4**: Cleanup and documentation
- **Week 5**: Performance optimization and benchmarking
- **Week 6**: Full deployment and legacy code removal

## Design Decisions (Resolved)

### 1. Slur Marker Placement
**Decision**: Keep slur markers in `BeatContent`

**Rationale**: Slurs are musical articulations that can start or end on any note within a beat, or span across beats. Placing `SlurStart` and `SlurEnd` inside `BeatContent` allows accurate modeling by interleaving them with notes and rests. If they were at the `Element` level, we couldn't model a slur that starts mid-beat.

### 2. Lyrics Spanning Multiple Notes (Melisma)
**Decision**: Use `syl: Option<String>` on the first note of a slurred passage

**Rationale**: The standard musical convention is to attach the syllable to the first note of a melisma. The converters know not to assign new syllables to subsequent notes under the same slur. The existing `lyrics.rs` module already handles this by distributing syllables and respecting slurs.

### 3. Ornament Representation
**Decision**: Separate enum variant with `OrnamentType`

**Rationale**: A separate `Ornament` variant in `BeatContent` is cleaner and more extensible than adding optional fields to the `Note` struct. This approach supports multiple ornament types without bloating the note structure:

```rust
Ornament {
    kind: OrnamentType, // Trill, Mordent, Turn, Grace, etc.
},
```

### 4. Tied Notes Across Barlines
**Decision**: Add `is_tied: bool` flag to `Note` struct

**Rationale**: The `Dash` variant handles rhythmic continuation within a beat, but ties across barlines need explicit representation. The `is_tied` boolean on the note preceding the barline indicates a tie to the next note. The parser identifies tie markers (like `~` in LilyPond) and sets `is_tied: true`. The following note after the barline is normal, but converters render the connecting tie.

## References

- [Original Go implementation](../notation_project/textual_music_notation/pkg/parser/)
- [Current Node implementation](src/models.rs)
- [Serde enum representations](https://serde.rs/enum-representations.html)