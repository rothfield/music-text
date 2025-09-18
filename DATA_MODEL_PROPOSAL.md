# Architecture Proposal: Immutable Pipeline Data Model

## Executive Summary

This proposal recommends restructuring the music-text data model from the current two-phase mutation approach to an immutable pipeline architecture. This change will improve code maintainability, enable better editor integration, and provide clearer separation of concerns.

## Problem Statement

The current architecture has several issues:

1. **Mutation-based rhythm analysis**: The parser creates incomplete `Stave` objects with `rhythm_items: None`, which are later mutated by `stave_analyzer`
2. **Duplicate representations**: Multiple overlapping types (`ParsedElement`, `BeatElement`, `ContentElement`) represent similar concepts
3. **Mixed concerns**: Parsing, rhythm analysis, and rendering logic are intertwined
4. **Unclear data flow**: The optional `rhythm_items` field creates ambiguity about when data is complete

## Proposed Solution: Immutable Pipeline

### Core Architecture

```rust
// Phase 1: Parsing (pure function)
pub fn parse(input: &str) -> Result<ParsedDocument, ParseError>

// Phase 2: Analysis (pure transformation)
pub fn analyze(parsed: ParsedDocument) -> AnalyzedDocument

// Phase 3: Projection (pure functions)
impl AnalyzedDocument {
    pub fn to_editor_tokens(&self) -> EditorProjection
    pub fn to_lilypond(&self) -> String
    pub fn to_vexflow(&self) -> VexFlowData
}
```

### Data Structures

```rust
// Phase 1: Parsed representation (immutable)
pub struct ParsedDocument {
    pub staves: Vec<ParsedStave>,
    pub source: String,
}

pub struct ParsedStave {
    pub lines: Vec<StaveLine>,
    pub notation_system: NotationSystem,
    pub source_range: Range,
}

// Phase 2: Analyzed representation (immutable)
pub struct AnalyzedDocument {
    pub staves: Vec<AnalyzedStave>,
    pub source: String,  // Preserved for error messages
}

pub struct AnalyzedStave {
    pub parsed: ParsedStave,        // Original preserved
    pub rhythm: RhythmStructure,    // Analysis results
    pub notation_system: NotationSystem,
}

pub struct RhythmStructure {
    pub beats: Vec<Beat>,
    pub time_signature: Option<TimeSignature>,
    pub key: Option<Key>,
}

// Phase 3: Editor projection
pub struct EditorProjection {
    pub tokens: Vec<SemanticToken>,
    pub diagnostics: Vec<Diagnostic>,
    pub folding_ranges: Vec<FoldingRange>,
    pub selection_ranges: Vec<SelectionRange>,
}
```

### Key Design Principles

1. **Immutability**: Each phase produces a new immutable structure
2. **Type safety**: The type system enforces that rhythm analysis happens before rendering
3. **Preservation**: Original parse tree is preserved for error reporting
4. **Single responsibility**: Each phase has a clear, single purpose

## Implementation Plan

### Phase 1: Add New Types (Week 1)
- Create `AnalyzedDocument` and related structures
- Keep existing code working alongside new types

### Phase 2: Port Analysis (Week 2)
- Convert rhythm FSM to pure function returning `RhythmStructure`
- Create `analyze()` function that transforms `ParsedDocument` → `AnalyzedDocument`

### Phase 3: Update Projections (Week 3)
- Port renderers to use `AnalyzedDocument`
- Implement editor projection methods
- Update web API to use new pipeline

### Phase 4: Cleanup (Week 4)
- Remove `rhythm_items: Option<...>` from `Stave`
- Remove duplicate element types
- Update tests and documentation

## Benefits

### For Development
- **Testability**: Each phase can be tested independently
- **Debugging**: Clear data flow makes issues easier to trace
- **Maintainability**: Changes to one phase don't affect others
- **Type safety**: Impossible to render without rhythm analysis

### For Editor Integration
- **Consistent UI**: All editor features use the same analyzed data
- **Performance**: Analysis happens once, projections are fast
- **Rich features**: Easy to add semantic highlighting, smart selection, etc.
- **LSP compatibility**: Clean mapping to Language Server Protocol concepts

### For Future Extensions
- **New output formats**: Just add new projection methods
- **Additional analysis**: Can add harmony analysis, form analysis as new phases
- **Caching**: Each phase output can be cached independently
- **Parallel processing**: Immutable data enables safe parallelization

## Migration Strategy

The migration can be done incrementally without breaking existing code:

1. **Add alongside**: New pipeline runs parallel to existing code
2. **Gradual adoption**: Port one renderer at a time
3. **Feature flag**: Use environment variable to switch between old/new
4. **Validation**: Run both pipelines and compare outputs
5. **Cleanup**: Remove old code once new pipeline is proven

## Example Usage

```rust
// Simple and clear data flow
let input = std::fs::read_to_string("song.mt")?;
let parsed = parse(&input)?;
let analyzed = analyze(parsed);

// Multiple projections from single analysis
let editor_tokens = analyzed.to_editor_tokens();
let lilypond = analyzed.to_lilypond();
let diagnostics = analyzed.to_diagnostics();

// Editor integration
for token in editor_tokens.tokens {
    editor.mark_text(token.range, token.style);
}
```

## Comparison with Current Approach

| Aspect | Current | Proposed |
|--------|---------|----------|
| Mutation | Required | None |
| Type safety | Optional fields | Enforced by types |
| Testing | Difficult (stateful) | Easy (pure functions) |
| Data flow | Unclear | Linear pipeline |
| Memory | Lower | Slightly higher |
| Performance | Similar | Similar |
| Complexity | Hidden | Explicit |

## Risks and Mitigations

### Risk 1: Memory overhead
**Mitigation**: Use string slicing and indices rather than copying strings. The overhead is minimal compared to benefits.

### Risk 2: Breaking changes
**Mitigation**: Incremental migration with parallel implementation allows validation before switching.

### Risk 3: Learning curve
**Mitigation**: The pipeline pattern is well-understood and simpler than current mutation-based approach.

## Decision Requested

Approval to begin implementation of the immutable pipeline architecture, starting with Phase 1 (adding new types alongside existing code) to minimize risk.

## Appendix: Detailed API

### Parser API
```rust
pub mod parser {
    pub fn parse(input: &str) -> Result<ParsedDocument, ParseError> {
        // Pure parsing, no rhythm analysis
    }
}
```

### Analyzer API
```rust
pub mod analyzer {
    pub fn analyze(doc: ParsedDocument) -> AnalyzedDocument {
        // Transform parsed → analyzed with rhythm
    }

    pub fn analyze_with_config(
        doc: ParsedDocument,
        config: AnalysisConfig
    ) -> AnalyzedDocument {
        // Configurable analysis for different styles
    }
}
```

### Projection API
```rust
impl AnalyzedDocument {
    // Editor projections
    pub fn to_editor_tokens(&self) -> EditorProjection;
    pub fn to_semantic_tokens(&self) -> Vec<SemanticToken>;
    pub fn to_codemirror_decorations(&self) -> Vec<Decoration>;

    // Renderer projections
    pub fn to_lilypond(&self) -> String;
    pub fn to_vexflow(&self) -> VexFlowData;
    pub fn to_musicxml(&self) -> String;

    // Analysis projections
    pub fn to_diagnostics(&self) -> Vec<Diagnostic>;
    pub fn get_statistics(&self) -> Statistics;
}
```