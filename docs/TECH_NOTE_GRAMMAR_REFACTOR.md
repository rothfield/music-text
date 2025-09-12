# Tech Note: Grammar Refactor Proposal - Generic Annotation Lines

**Date**: 2025-09-04  
**Issue**: Pest grammar fails to parse spatial octave notation (`1\n.`) due to ambiguous line type detection  
**Root Cause**: Grammar tries to distinguish upper_line vs lower_line vs lyrics_line during parsing phase  

## Current Problem

The existing grammar structure creates parsing ambiguities:

```pest
number_stave = {
    (number_upper_line ~ newline)* ~     // Tries dots as upper first
    number_content_line ~                // Content line parsing
    (newline ~ lower_line)* ~            // Should handle dots as lower
    (newline ~ lyrics_line)*
}
```

**Issues:**
1. `content_line` has `newline?` (optional), leaving newlines unconsumed
2. Parser continues trying `upper_line` patterns after content_line
3. Negative lookaheads (`!content_line`) create complex disambiguation logic
4. Same symbols (dots, colons) valid in multiple contexts cause conflicts

## Proposed Solution: Divide and Conquer

### Phase 1: Parse Structure (Divide)
Simplify grammar to distinguish only **content** vs **annotation**:

```pest
stave = {
    annotation_line* ~
    content_line ~
    annotation_line*
}

annotation_line = {
    !content_line ~ annotation_item+ ~ newline
}

annotation_item = {
    "." | "*" | ":" |           // Octave markers
    syllable |                  // Lyrics  
    chord |                     // [Am], [C7]
    ornament |                  // <123>, grace notes
    tala |                      // +, rhythm markers
    slur |                      // ___
    " "+                        // Whitespace
}

content_line = {
    line_number? ~
    barline? ~
    measure ~ (barline ~ measure)* ~
    barline? ~
    newline                     // REQUIRED newline
}
```

### Phase 2: Semantic Classification (Conquer)
Post-process parsed `annotation_line`s based on position and content:

```rust
fn classify_annotation_lines(stave: &mut Stave) {
    let content_index = find_content_line_index(stave);
    
    for (i, annotation_line) in stave.annotation_lines.iter().enumerate() {
        let line_type = if i < content_index {
            classify_pre_content_line(annotation_line)   // upper_line, etc.
        } else {
            classify_post_content_line(annotation_line)  // lower_line, lyrics
        };
        
        // Convert to appropriate AST node type
        match line_type {
            LineType::UpperOctave => /* create UpperLine with octave markers */,
            LineType::LowerOctave => /* create LowerLine with octave markers */,
            LineType::Lyrics => /* create LyricsLine */,
            LineType::Mixed => /* handle complex cases */,
        }
    }
}

fn classify_pre_content_line(line: &AnnotationLine) -> LineType {
    if line.contains_octave_markers() { LineType::UpperOctave }
    else if line.contains_lyrics() { LineType::Lyrics }
    else { LineType::Mixed }
}

fn classify_post_content_line(line: &AnnotationLine) -> LineType {
    if line.contains_octave_markers() { LineType::LowerOctave }
    else if line.contains_lyrics() { LineType::Lyrics }
    else { LineType::Mixed }
}
```

## Benefits

1. **Eliminates Parsing Ambiguity**: No more negative lookaheads or complex precedence rules
2. **Cleaner Grammar**: Single `annotation_line` rule instead of multiple specialized rules
3. **Flexible Classification**: Can handle edge cases and mixed-content lines in post-processing
4. **Better Error Messages**: Parsing errors are structural, semantic errors are contextual
5. **Easier Testing**: Can test parsing and classification separately

## Example

**Input**: `1\n.`

**Phase 1 Parse Result**:
```
stave: {
  annotation_lines: [],
  content_line: { measures: [{ beats: [{ elements: ["1"] }] }] },
  annotation_lines: [{ items: ["."] }]
}
```

**Phase 2 Classification**:
```
stave: {
  upper_lines: [],
  content_line: { measures: [{ beats: [{ elements: ["1"] }] }] },
  lower_lines: [{ items: [LowerOctaveMarker(".")] }],
  lyrics_lines: []
}
```

## Implementation Plan (Updated)

### Phase 1: Core Grammar Changes
1. **Strengthen `content_line` definition** - Ensure it only matches lines with actual musical pitches/beats, not ornaments with numbers like `<123>`
2. Create new `annotation_line` rule in grammar templates with robust `!content_line` negative lookahead
3. Test `content_line` vs `annotation_line` distinction thoroughly

### Phase 2: AST Architecture 
**Option A (Cleaner)**: Pure transformation pipeline
```rust
RawStave {
    pre_content_annotations: Vec<AnnotationLine>,
    content_line: ContentLine,
    post_content_annotations: Vec<AnnotationLine>,
}
// Transform to:
Stave {
    upper_lines: Vec<AnnotationLine>,
    content_line: ContentLine, 
    lower_lines: Vec<AnnotationLine>,
    lyrics_lines: Vec<LyricsLine>,
}
```

**Option B (Current proposal)**: In-place mutation of `Stave`

### Phase 3: Classification Logic
1. Implement classification functions with **preserved positional information**
2. Ensure semantic errors point to exact source locations
3. Handle edge cases (mixed content, ornaments, future annotation types)

### Phase 4: Integration & Testing
1. Update parser to produce `RawStave` or modified `Stave`
2. Test with existing notation examples
3. Verify error reporting quality

## Risk Assessment

- **Low Risk**: Changes are additive, can be implemented incrementally
- **Critical Success Factor**: Robust `content_line` definition that doesn't match annotation content
- **Backward Compatibility**: Existing functionality preserved through classification
- **Performance**: Minimal impact, classification is O(n) per stave
- **Error Quality**: Must preserve line/column positions through classification

## Key Considerations from Critique

1. **`content_line` Robustness**: Must be specific enough to only match musical pitches, not ornaments with numbers
2. **Error Reporting**: Preserve `Position` data through classification for high-quality semantic error messages  
3. **Architecture Pattern**: Consider pure transformation (`RawStave` → `Stave`) over in-place mutation
4. **Extensibility**: Rust classification logic much more powerful than grammar rules for future features

This approach follows the principle of **separation of concerns**: parsing handles syntax, classification handles semantics. The critique validates this as a standard, robust design pattern for parser architecture.