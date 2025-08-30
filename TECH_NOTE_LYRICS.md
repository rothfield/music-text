# Technical Note: Lyrics Handling in Notation Parser

## Date: 2025-01-25

## Overview
This document captures the analysis and discussion about lyrics handling in the notation parser compared to music-text's implementation.

## Current Implementation

### Data Model
- Lyrics are stored as `Option<String>` in the `syl` field of `Node` structs
- The `lyrics.rs` module provides dedicated functions for lyrics processing

### Processing Flow
1. **Tokenization**: The lexer identifies `WORD` tokens on lines below musical notation
2. **Lyrics Extraction**: `parse_lyrics_lines()` collects all WORD tokens from lyrics lines
3. **Syllable Distribution**: `distribute_syllables_to_notes()` assigns syllables to PITCH nodes sequentially
4. **Melisma Handling**: Notes within slurs share syllables (first note gets syllable, rest get "_")

### Key Functions
- `parse_text_as_word_tokens()`: Re-tokenizes lyrics lines as words
- `parse_lyrics_lines()`: Extracts lyrics from token stream  
- `distribute_syllables_to_notes()`: Main distribution logic
- `attach_syllables_respecting_slurs()`: Handles melisma via slur boundaries

## DoremiScript Comparison

### Initial Observations
Initially, it appeared that music-text used a different approach with:
- Lyrics placed above musical notation
- Pre-hyphenated syllables
- Spatial alignment for syllable-to-note mapping

### Deeper Analysis
After examining the Clojure source code (`to_lilypond.cljc`), we found:
- Pitches have `:syl` attributes attached directly
- `get-syl` function extracts syllables from pitch nodes
- Sequential distribution similar to our approach
- Dual source system: inline `:syl` attributes OR separate `:lyrics-line` nodes

### Core Similarity
Both systems fundamentally:
1. **Attach syllables to pitch nodes** as optional attributes
2. **Use sequential distribution** for syllable-to-note matching
3. **Handle melismas through slurs** (one syllable across multiple notes)
4. **Parse lyrics separately** then attach to musical elements

## Key Challenges in Lyrics Digitization

### Spatial Alignment Problem
Handwritten notation relies on visual positioning to indicate syllable-to-note correspondence. However, this breaks down when:
- Notes have variable spacing (accidentals, ornaments, readability)
- Barlines interrupt natural spacing
- Multiple verses need to align with same notes

### Current Solutions
- **Sequential Distribution**: Simple, works for most cases
- **Hyphenation Preservation**: "geor-gia" → "geor-" and "gia"
- **Melisma via Slurs**: Slurred notes share syllables
- **Underscore Extension**: "_" indicates held syllables

## Implementation Differences

| Aspect | Notation Parser (Rust) | DoremiScript (Clojure) |
|--------|------------------------|------------------------|
| Data Structure | `Node { syl: Option<String> }` | `[:pitch "S" [:syl "ta"]]` |
| Storage | Struct field | Nested vector attribute |
| Extraction | Direct field access | `get-syl` function |
| Distribution | Imperative loop | Functional reduction |
| Language | Rust | Clojure |

## Potential Improvements

### From Analysis
1. **Explicit Anchoring**: Allow syntax like `S[ta]` to explicitly bind syllables
2. **Multiple Verses**: Support verse numbers/labels
3. **Skip Markers**: Allow instrumental passages without syllables
4. **Better Dash Handling**: Use `-` for continuation vs hyphenation

### From DoremiScript
1. **Dual Source System**: Support both inline and separate lyrics
2. **Lyrics as First-Class**: Ensure syllables survive AST transformations
3. **Tree Traversal**: Use recursive traversal for syllable collection

## Conclusion

The fundamental approach to lyrics handling is the same in both systems: syllables are attributes of pitch nodes, distributed sequentially with special handling for melismas. The differences are primarily in implementation details (functional vs imperative, Clojure vs Rust) rather than conceptual approach.

The key insight is that both systems recognize lyrics must be **attached to notes as attributes** rather than maintained as separate parallel streams, solving the fundamental synchronization problem in musical notation.

## Future Considerations

1. **Explicit Binding Syntax**: Consider allowing explicit syllable-to-note binding for complex cases
2. **Verse Support**: Implement multiple verse handling
3. **Melisma Alternatives**: Consider supporting melisma indication without requiring visible slurs
4. **Spatial Hints**: Use column positions as hints but not sole determinant

---
*Generated from discussion about lyrics handling approaches, comparing current implementation with music-text's Clojure-based system.*