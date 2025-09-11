# Module Naming Refactor Plan

## Current State Analysis

### Current Pipeline Modules:
1. `handwritten_lexer.rs` - ✅ Clear name (tokenization)
2. `region_processor.rs` - ❓ Ambiguous (spatial/vertical markup)  
3. `rhythm_fsm.rs` - ❓ Technical name (rhythmic/temporal parsing)

### Usage Analysis:
- **rhythm_fsm** imported in 4 files: `lib.rs`, `vexflow_js_generator.rs`, `converters/vexflow.rs`, `converters/lilypond.rs`
- Primary module declaration in `lib.rs:12`
- Core functionality: Beat detection, tuplet analysis, rhythmic structure

## Recommended Renaming

### Option A: Function-Based Names
```
handwritten_lexer.rs     → tokenizer.rs
region_processor.rs      → spatial_parser.rs  
rhythm_fsm.rs           → rhythmic_parser.rs
```

### Option B: Processing Stage Names
```
handwritten_lexer.rs     → lexer.rs (already exists as separate module)
region_processor.rs      → markup_processor.rs
rhythm_fsm.rs           → beat_parser.rs  
```

### Option C: Data Flow Names
```
handwritten_lexer.rs     → tokenizer.rs
region_processor.rs      → vertical_parser.rs
rhythm_fsm.rs           → temporal_parser.rs
```

## Final Naming Convention

**User-Approved Choice:**
```
handwritten_lexer.rs  → tokenizer.rs
region_processor.rs   → vertical_parser.rs  
rhythm_fsm.rs        → horizontal_parser.rs
```

**Rationale:**
- `tokenizer.rs` - Clear tokenization function
- `vertical_parser.rs` - Processes vertical spatial relationships (markings above/below the main line)
- `horizontal_parser.rs` - Processes horizontal temporal relationships (beats, rhythm, time flow)
- Creates perfect architectural symmetry: spatial (vertical) analysis followed by temporal (horizontal) analysis
- Visual clarity: anyone reading the code immediately understands the spatial/temporal division

## Implementation Steps

### 1. File Renames
```bash
mv src/handwritten_lexer.rs src/tokenizer.rs
mv src/region_processor.rs src/vertical_parser.rs
mv src/rhythm_fsm.rs src/horizontal_parser.rs  
```

### 2. Module Declaration Updates (lib.rs)
```rust
// Current:
pub mod handwritten_lexer;
mod region_processor;
pub mod rhythm_fsm;

// New:
pub mod tokenizer;
mod vertical_parser; 
pub mod horizontal_parser;
```

### 3. Import Statement Updates

**lib.rs** (multiple locations):
```rust
// Current:
use crate::rhythm_fsm;
region_processor::apply_slurs_and_regions_to_elements(&mut elements, &remaining_tokens);
let mut elements = rhythm_fsm::group_elements_with_fsm_full(&elements, &lines_of_music);
elements.insert(0, rhythm_fsm::Item::Tonic(tonic_degree));

// New:
use crate::horizontal_parser;
vertical_parser::apply_slurs_and_regions_to_elements(&mut elements, &remaining_tokens);
let mut elements = horizontal_parser::group_elements_with_fsm_full(&elements, &lines_of_music);
elements.insert(0, horizontal_parser::Item::Tonic(tonic_degree));
```

**converters/lilypond.rs**:
```rust
// Current: 
use crate::rhythm_fsm::{Item, Beat};

// New:
use crate::horizontal_parser::{Item, Beat};
```

**converters/vexflow.rs**:
```rust
// Current:
use crate::rhythm_fsm::{Item, Beat, BeatElement};

// New: 
use crate::horizontal_parser::{Item, Beat, BeatElement};
```

**vexflow_js_generator.rs**:
```rust
// Current:
use crate::rhythm_fsm::{Item, Beat, BeatElement};

// New:
use crate::horizontal_parser::{Item, Beat, BeatElement};
```

## Benefits of This Refactoring

1. **Clarity**: Module names clearly indicate their function in the pipeline
2. **Consistency**: Both parsers use "parser" suffix
3. **Self-Documenting**: New developers can understand the pipeline flow
4. **Maintainability**: Easier to locate functionality when debugging
5. **Architecture Alignment**: Names reflect the beat-centric architecture insights

## Pipeline After Refactoring

```
Raw Text → tokenizer → vertical_parser → horizontal_parser → converters
                            ↑                    ↑
                        spatial             temporal
```

This creates a clear, understandable processing pipeline with perfect architectural symmetry between spatial (vertical) and temporal (horizontal) processing.