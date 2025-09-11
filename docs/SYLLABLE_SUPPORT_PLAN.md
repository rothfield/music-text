# Minimal Syllable Support Implementation Plan

## Overview
This document outlines the complete plan for implementing minimal syllable support in the music-text. Syllables will be snapped to the nearest pitch before or after them, with lyrics lines reparsed as syllables/words and integrated into both VexFlow and LilyPond output with SOP styling (small, italic).

## Key Insights from Analysis

### Grammar Structure (from doremi-script EBNF)
```ebnf
doremi-stave =
(doremi-upper-line <newline>)*      # Ornaments, octave markers above
doremi-notes-line                   # The actual musical notes  
(<newline> lower-octave-line)*      # Octave markers below
(<newline> lyrics-line)*            # Syllables/lyrics at bottom (per stave)
```

### Critical Architecture Decisions
1. **Lyrics are spatial elements** - same level as octave markers
2. **Per-stave processing** - each stave gets its own lyrics
3. **Position-based snapping** - syllables snap to nearest pitches by 2D distance
4. **Existing infrastructure** - VexFlow support already 90% complete

## 1. Lyrics Detection Requirements & Edge Cases

### Core Requirements
- **Primary rule**: Lyrics are generally the LAST line of a line group
- **Secondary rule**: Lines with syllable patterns (hyphens, common words)
- **Fallback rule**: Pure alphabetic content vs musical notation

### Edge Cases Handled
- **Multiple text lines**: Last line wins
- **Mixed musical content**: Position + content analysis
- **No clear lyrics**: Parse all as musical
- **Single line mixed**: Extract syllables from end
- **Multiple stanzas**: Each group processed independently

### Detection Confidence Levels
- **High**: Lines with multiple hyphens (`"twin-kle lit-tle"`)
- **Medium**: Last line + common words (`"beautiful morning"`)
- **Low**: Pure alphabetic content (`"CDEFG"` could be pitches)

## 2. Data Flow Architecture

### Simplified Pipeline
```
Raw Input → Tokenizer → Parser → Vertical Parser → FSM → Existing Converters → Output
                                       ↓               ↓
                               (Spatial Snapping)  (Passes Through)
                                to Notes          Syllable Data
```

**Key Insight**: No converter enhancement needed - vertical parser adds syllables to notes as `ParsedChild::Syllable`, and existing converters already handle this via `beat_element.syl()` method.

### Data Structures
```rust
struct LineGroup {
    musical_lines: Vec<String>,
    lyrics_line: Option<String>,
    line_positions: Vec<Position>,
}

struct LyricsContext {
    syllables: Vec<String>,
    syllable_positions: Vec<Position>,
    confidence: LyricsConfidence,
}

enum LyricsConfidence { High, Medium, Low }
```

## 3. Lyrics Line Detection Algorithm

### Algorithm Steps
1. **Line Grouping**: Split on empty lines
2. **Per-Group Analysis**: Check last line first, scan for high confidence
3. **Confidence Scoring**: Weighted scoring system

### Confidence Analysis
```rust
fn analyze_lyrics_confidence(line: &str) -> Option<LyricsConfidence> {
    let mut score = 0;
    
    // High confidence indicators
    if words.iter().any(|w| w.ends_with('-')) { score += 3; }
    if words.iter().any(|w| w.contains('-') && w.len() > 2) { score += 3; }
    
    // Medium confidence indicators  
    if words.iter().any(|w| is_common_lyrics_word(w)) { score += 2; }
    if words.iter().any(|w| w.len() > 6) { score += 1; }
    
    // Negative indicators
    if words.iter().any(|w| w.len() == 1 && w.chars().all(|c| c.is_numeric())) { score -= 2; }
    
    match score {
        s if s >= 4 => Some(LyricsConfidence::High),
        s if s >= 2 => Some(LyricsConfidence::Medium),  
        s if s >= 1 => Some(LyricsConfidence::Low),
        _ => None
    }
}
```

### Example Detection
```
Input: ["1 2 3 4", "Twink-le twin-kle lit-tle star"]
Analysis: 
- Last line: "Twink-le twin-kle lit-tle star"
- Score: 3 (ends with -) + 3 (contains -) + 1 (all alphabetic) = 7
- Result: High confidence lyrics at index 1
```

## 4. Syllable Parsing & Tokenization

### Tokenization Rules
1. **Split on whitespace** → `["Twink-le", "twin-kle", "lit-tle", "star"]`
2. **Split hyphens** → `["Twink", "le", "twin", "kle", "lit", "tle", "star"]`
3. **Preserve positions** for snapping algorithm
4. **Handle edge cases**: apostrophes, multi-hyphens, musical notation

### Implementation
```rust
struct SyllableToken {
    text: String,
    position: Position,
    word_index: usize,
    syllable_index: usize,
}

fn tokenize_syllables(lyrics_line: &str, line_position: Position) -> Vec<SyllableToken> {
    // Split words, then split hyphens within words
    // Maintain position tracking for spatial snapping
}
```

## 5. Syllable-to-Pitch Snapping Algorithm

### Unified Spatial Processing
Syllables use the **same snapping algorithm** as ornaments and octave markers. The vertical parser already has spatial processing logic that should be extended, not duplicated.

### Integration with Existing Logic
```rust
// In vertical_parser.rs - extend existing spatial processing
fn snap_spatial_elements_to_notes(region: &mut Region) {
    // Handle octave markers (existing)
    process_octave_markers(region);
    
    // Handle ornaments (existing) 
    process_ornaments(region);
    
    // Handle syllables (new - same pattern)
    process_syllables(region);
}

fn process_syllables(region: &mut Region) {
    let syllable_elements = region.elements.iter()
        .filter(|e| matches!(e, ParsedElement::Word { .. }))
        .collect::<Vec<_>>();
        
    let note_elements = region.elements.iter_mut()
        .filter(|e| matches!(e, ParsedElement::Note { .. }))
        .collect::<Vec<_>>();
        
    // Use existing spatial snapping logic (same as ornaments/octaves)
    snap_elements_to_nearest_notes(syllable_elements, note_elements, |word, note| {
        if let (ParsedElement::Word { text, position, .. }, ParsedElement::Note { children, .. }) = (word, note) {
            children.push(ParsedChild::Syllable {
                text: text.clone(),
                distance: calculate_spatial_distance(position, note.position()),
            });
        }
    });
}
```

### Consistency with Existing Elements
This approach ensures **identical behavior** to octave markers and ornaments:
- Same 2D distance calculation
- Same nearest-neighbor algorithm  
- Same spatial relationship processing
- Leverages existing, tested code

### Snapping Examples
```
1    2    3    4
Twink le  star bright
```
- "Twink" → nearest to "1" 
- "le" → nearest to "2"
- "star" → nearest to "3" 
- "bright" → nearest to "4"

## 6. VexFlow Integration

### No Changes Required
✅ **Already complete**: `StaffNotationElement::Note` has `syl: Option<String>` field  
✅ **Already complete**: VexFlow converter calls `beat_element.syl()` method  
✅ **Already complete**: VexFlow renderer has syllable rendering capability  

The vertical parser adds syllables to notes as `ParsedChild::Syllable`, the FSM preserves them in `BeatElement.event`, and the converter extracts them via `beat_element.syl()`. **No VexFlow converter changes needed.**

### SOP Styling Enhancement Only
```javascript
// Only change needed: update styling in vexflow-renderer.js
if (note.syl) {
    const syllable = new VF.Annotation(note.syl);
    
    // SOP Requirements: Small and Italic
    syllable.setFont("Times", 8, "italic");  // 8pt, italic
    syllable.setVerticalJustification(VF.Annotation.VerticalJustify.BOTTOM);
    syllable.setHorizontalJustification(VF.Annotation.HorizontalJustify.CENTER);
    syllable.setOffsetY(12);  // Position below staff
    
    vfNote.addModifier(syllable, 0);
}
```

### Integration Status
**VexFlow Integration: 100% Complete** - only styling updates needed.

## 7. LilyPond Integration with Per-Stave Lyrics

### Per-Stave Structure
Each stave gets its own `\addlyrics` block:
```lilypond
% Stave 1
\new Staff { c4 d4 e4 f4 }
\addlyrics { Twink -- le twin -- kle }

% Stave 2  
\new Staff { g4 a4 b4 c4 }
\addlyrics { lit -- tle bright star }
```

### Template Enhancement
```mustache
\score {
  <<
    {{#staves}}
    \new Staff {
      \relative c' {
        {{{notes}}}
      }
    }
    {{#has_lyrics}}
    \addlyrics {
      {{{syllables}}}
    }
    {{/has_lyrics}}
    {{/staves}}
  >>
}
```

### Lyrics Collection
```rust
fn extract_stave_syllables(beats: &[Beat]) -> Option<String> {
    let syllables: Vec<String> = beats.iter()
        .flat_map(|beat| &beat.elements)
        .map(|elem| {
            elem.syl().unwrap_or_else(|| {
                // LilyPond requires placeholders for notes without syllables
                if elem.is_note() { "_" } else { "" }
            })
        })
        .filter(|s| !s.is_empty())
        .collect();
        
    if syllables.is_empty() {
        None
    } else {
        Some(syllables.join(" "))
    }
}
```

### SOP Styling
```lilypond
\layout {
  \context {
    \Lyrics
    \override LyricText.font-size = #-2     % Small
    \override LyricText.font-shape = #'italic % Italic  
    \override LyricText.self-alignment-X = #CENTER
  }
}
```

## 8. SOP Styling Requirements Summary

### Small and Italic Requirements
- **VexFlow**: 8pt Times italic font, centered below staff
- **LilyPond**: font-size -2, italic shape, center alignment
- **CSS**: 10px italic serif for web interface

## 9. Comprehensive Test Cases

### Basic Syllable Detection
- **TC1.1**: `"1 2 3 4\nTwink-le twin-kle lit-tle star"` → syllables detected
- **TC1.2**: `"1 2 3 4\nMary had little lamb"` → common words detected
- **TC1.3**: `"1-2 S-R G-M\nLa-la la-la"` → last line as lyrics

### Grammar Compliance
- **TC2.1**: Complete stave structure with spatial snapping
- **TC2.2**: Multiple lyrics lines per stave

### Edge Cases
- **TC3.1**: No lyrics detected, parse as musical notation
- **TC3.2**: More syllables than notes → excess ignored
- **TC3.3**: Fewer syllables than notes → underscore placeholders

### Output Verification
- **TC4.1**: VexFlow JSON with syllable fields
- **TC4.2**: LilyPond with `\addlyrics` blocks

### Integration Tests
- **TC6.1**: Syllables with tuplets
- **TC6.2**: Syllables with ornaments  
- **TC6.3**: Syllables with slurs

## 10. Implementation Sequence

1. **Tokenizer**: Create `ParsedElement::Word` for lyrics lines (detect syllables in tokenization phase)
2. **Vertical Parser**: Extend existing spatial processing to handle syllables (same algorithm as ornaments/octaves)
3. **LilyPond Converter**: Extract syllables via existing `beat_element.syl()` and generate `\addlyrics` per stave
4. **Styling Updates**: 
   - VexFlow: Update font size/style in `vexflow-renderer.js`
   - LilyPond: Add lyrics styling to template
5. **Test Coverage**: Implement comprehensive test cases

**Key Simplification**: No FSM or converter logic changes needed - syllables flow through existing data structures automatically.

## Conclusion

This **revised plan** provides minimal syllable support with maximum code reuse:

### Key Simplifications
- **No converter changes**: Existing `beat_element.syl()` method handles syllables automatically
- **Unified spatial processing**: Same algorithm as ornaments and octave markers
- **Existing infrastructure**: VexFlow support is 100% complete, LilyPond nearly complete
- **Grammar compliance**: Follows doremi-script EBNF structure precisely

### Implementation Requirements
1. **Tokenizer**: Detect lyrics lines and create `ParsedElement::Word` elements
2. **Vertical Parser**: Extend spatial processing (1 new function using existing algorithm)
3. **Styling**: Minor updates for SOP requirements (small, italic)
4. **LilyPond**: Extract syllables and generate per-stave `\addlyrics` blocks

### Architecture Benefits
- **Consistency**: Syllables processed identically to other spatial elements
- **Simplicity**: No new data flow patterns or converter logic
- **Reliability**: Leverages existing, tested spatial snapping algorithms
- **Maintainability**: Minimal new code, follows established patterns

The implementation is much simpler than originally planned because the existing architecture already handles syllables naturally through the spatial processing and data structure pipeline.