# Grammar Migration Technical Note: Doremi-script EBNF to Pest

**Date**: 2025-01-03  
**Author**: Claude Code Assistant  
**Context**: Complete parser rewrite from multi-stage token-based to unified pest grammar

## Executive Summary

The music-text has been successfully migrated from the original doremi-script Clojure/Instaparse EBNF grammar to a Rust Pest grammar. This migration represents a complete architectural shift that eliminates the complex multi-stage parsing pipeline in favor of direct structural parsing.

## Original Architecture (doremi-script)

### Grammar Structure
- **File**: `doremi-script/resources/doremiscript.ebnf` (generated from template)
- **Parser**: Clojure Instaparse library
- **Entry Point**: `composition = hindi-composition|number-composition|sargam-composition|abc-composition|doremi-composition`

### Key Components
```ebnf
sargam-stave = 
  (sargam-upper-line <newline>)*
  sargam-notes-line 
  (<newline> lower-octave-line)*
  (<newline> lyrics-line)*

sargam-notes-line = 
  line-number?
  <white-space?> 
  barline? 
  sargam-measure 
  (barline sargam-measure)*  
  barline?
```

### Processing Pipeline (Old)
1. **Lexer**: Text → Tokens
2. **Tokenizer**: Tokens → Structured tokens  
3. **Parser**: Structured tokens → Elements
4. **FSM**: Elements → Beats (rhythm analysis)
5. **Spatial**: Beat assignment → Musical structure
6. **Converters**: Musical structure → Output formats

## New Architecture (Pest Grammar)

### Grammar Structure
- **File**: `grammar/notation.pest`
- **Parser**: Rust Pest library  
- **Entry Point**: `document` with system-specific variants

### Key Components
```pest
stave = {
    (upper_line ~ newline)* ~
    content_line ~
    (newline ~ lower_line)* ~
    (newline ~ lyrics_line)*
}

content_line = { 
    line_number? ~
    ws_opt ~
    start_barline? ~
    measure ~
    (barline ~ measure)* ~
    end_barline? ~
    ws_opt 
}
```

### Processing Pipeline (New)
1. **Pest Parser**: Text → Complete AST (measures and beats included!)
2. **Spatial Processing**: Assign slurs, octaves, lyrics
3. **Rhythm FSM**: Optional rhythm enrichment  
4. **Renderers**: AST → Output formats

## Critical Architectural Improvements

### 1. Direct Measure and Beat Parsing
**Before**: FSM created beats from flat element stream  
**After**: Pest grammar produces measures and beats directly

```rust
// Old: Elements processed by FSM to create beats
let elements = parse_elements(tokens);
let beats = fsm.process(elements);

// New: Measures and beats from grammar
Document {
    staves: [Stave {
        content_line: ContentLine {
            measures: [Measure {
                beats: [Beat { elements: [...] }]
            }]
        }
    }]
}
```

### 2. Eliminated Token-First Complexity  
**Before**: Token → Spatial reconstruction → Musical relationships  
**After**: Musical structure built directly during parsing

### 3. Comprehensive Position Tracking
**Before**: Position information lost/reconstructed across stages  
**After**: Every pest node includes `pair.line_col()` position data

## Grammar Comparison Analysis

### ✅ Successfully Ported Features

| Feature | Original EBNF | Pest Grammar | Status |
|---------|---------------|--------------|---------|
| Document structure | `composition` variants | `document` + system entry points | ✅ Complete |
| Stave hierarchy | `sargam-stave` | `stave` | ✅ Complete |
| Measure parsing | `sargam-measure` | `measure` | ✅ Enhanced |
| Beat parsing | `sargam-beat` | `beat` | ✅ Enhanced |
| Barlines | All types | All types | ✅ Complete |
| Attributes | `attribute-section` | `attribute_section` | ✅ Complete |
| Lyrics | `lyrics-section` | `lyrics_section` | ✅ Complete |
| Notation systems | 5 systems | 5 systems | ✅ Complete |

### 🎯 Key Enhancements

1. **Empty Measure Support**: Pest grammar handles `|` and `| |` patterns correctly
2. **Whitespace Handling**: Explicit `ws` rules instead of automatic WHITESPACE
3. **System Detection**: Better auto-detection of notation systems
4. **Error Recovery**: Pest provides better error messages with position info

### 📋 Terminology Updates

| Original | New | Rationale |
|----------|-----|-----------|
| `composition` | `document` | Clearer top-level structure |  
| `sargam-stave` | `stave` | System-agnostic terminology |
| `sargam-notes-line` | `content_line` | Clearer purpose |
| Various `<hidden>` rules | Explicit rules | Better debugging |

## Testing Validation

### Core Functionality Tests
```bash
# Empty measures (previously problematic)
./target/release/cli --input "|"        # ✅ Works
./target/release/cli --input "| |"      # ✅ Works  

# Basic notation
./target/release/cli --input "1 2 3"    # ✅ Works
./target/release/cli --input "1 | 2 |"  # ✅ Works

# Complex patterns (from doremi-script test suite)
# All major test patterns from resources/fixtures/ validate correctly
```

### Rhythm FSM Integration
```rust
// Re-enabled rhythm FSM in parser.rs
let rhythm_items = parser_v2_fsm::group_elements_with_fsm_full(&parsed_elements, &[]);
// ✅ Builds and runs successfully
```

## Performance Impact

### Positive Changes
- **Single-pass parsing** vs multi-stage pipeline
- **Direct AST structure** eliminates reconstruction overhead  
- **Pest optimizations** for grammar matching

### Memory Usage
- **Reduced intermediate representations** (no token/element stages)
- **Direct AST allocation** with proper ownership

## Migration Quality Assessment

### 🟢 Excellent Coverage
- All core musical constructs supported
- All notation systems functional  
- All barline types working
- Position tracking comprehensive

### 🟡 Areas for Future Enhancement
- Advanced ornament parsing (can be added incrementally)
- Complex slur patterns (spatial processing handles basics)
- Extended time signatures (framework supports expansion)

### 🔴 Known Limitations
- Some advanced doremi-script edge cases may need individual porting
- Complex nested bracket patterns may require grammar refinement

## Conclusion

The migration from doremi-script EBNF to Pest grammar represents a **successful architectural evolution**. The new parser:

1. **Eliminates complexity**: Single unified parsing stage
2. **Improves structure**: Direct measure/beat parsing  
3. **Enhances debugging**: Comprehensive position tracking
4. **Maintains compatibility**: All core features ported
5. **Enables growth**: Clean foundation for new features

The pest grammar successfully captures the **musical intent** of the original doremi-script grammar while providing the **engineering benefits** that motivated the rewrite.

### References
- Original grammar: `doremi-script/resources/doremiscript.ebnf`
- New grammar: `grammar/notation.pest`  
- Commit: "feat: Complete parser rewrite using pest grammar ported from doremi-script"
- Related: `PARSER_REWRITE_PROPOSAL.md`, `RENAMING_PLAN.md`