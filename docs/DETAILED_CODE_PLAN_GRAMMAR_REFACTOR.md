# Detailed Code-Level Plan: Grammar Refactor with Specialized Grammars

**Date**: 2025-09-04  
**Approach**: Multi-grammar system with position-aware parsing

## Core Insight: Context-Aware Grammar Selection

Instead of generic `annotation_line`, use **specialized grammars** based on position relative to content:

```pest
stave = {
    pre_content_line* ~
    content_line ~
    post_content_line*
}

pre_content_line = {
    !content_line ~ upper_grammar ~ newline
}

post_content_line = {
    !content_line ~ (lower_grammar | lyrics_grammar) ~ newline
}
```

## Detailed Implementation Plan

### Phase 1: Grammar Architecture

#### 1.1 Core Line Classification Grammar
```pest
// Main stave structure - position determines grammar used
stave = {
    pre_content_line* ~
    content_line ~
    post_content_line*
}

// Pre-content lines use upper_grammar
pre_content_line = {
    !content_line ~ upper_grammar ~ newline
}

// Post-content lines use lower_grammar or lyrics_grammar
post_content_line = {
    !content_line ~ post_content_grammar ~ newline
}

post_content_grammar = {
    lower_grammar | lyrics_grammar
}
```

#### 1.2 Content Grammar (Strengthened)
```pest
// CRITICAL: Must be specific enough to only match musical content
content_line = {
    line_number? ~
    barline? ~
    content_measure ~ (barline ~ content_measure)* ~
    barline? ~
    newline  // REQUIRED - no more optional newlines
}

content_measure = {
    ws_opt ~ content_beat ~ (ws_req ~ content_beat)* ~ ws_opt
}

content_beat = {
    musical_element+  // Only actual musical elements
}

musical_element = {
    pitch | dash | begin_slur | end_slur | breath_mark
    // Explicitly excludes: dots, ornaments, chords, etc.
}
```

#### 1.3 Specialized Context Grammars
```pest
// Upper grammar - for lines BEFORE content
upper_grammar = {
    upper_item+
}

upper_item = {
    upper_octave_marker |  // "." | "*" | ":"
    tala_marker |          // "+", digits for rhythm
    ornament |             // "<123>" grace notes
    chord |                // "[Am]" chord symbols
    slur |                 // "___" slur markings
    ending |               // "1.---" section endings
    ws+                    // Whitespace
}

upper_octave_marker = { "." | "*" | ":" }

// Lower grammar - for lines AFTER content
lower_grammar = {
    lower_item+
}

lower_item = {
    lower_octave_marker |  // "." | "*" | ":" (same symbols, different context)
    kommal_indicator |     // "_" flat indicator
    beat_grouping |        // "__" beat groupings
    ws+                    // Whitespace
}

lower_octave_marker = { "." | "*" | ":" }

// Lyrics grammar - for syllables
lyrics_grammar = {
    syllable ~ (ws+ ~ syllable)*
}

syllable = @{
    (ASCII_ALPHA | "'" | "!" | "-")+
}
```

### Phase 2: AST Architecture Redesign

#### 2.1 Raw Parse Result (Phase 1 Output)
```rust
// src/ast/raw.rs - NEW FILE
#[derive(Debug, Clone)]
pub struct RawStave {
    pub pre_content_lines: Vec<RawAnnotationLine>,
    pub content_line: ContentLine,
    pub post_content_lines: Vec<RawAnnotationLine>,
    pub position: Option<Position>,
}

#[derive(Debug, Clone)]
pub struct RawAnnotationLine {
    pub content: RawAnnotationContent,
    pub position: Option<Position>,
}

#[derive(Debug, Clone)]
pub enum RawAnnotationContent {
    Upper(Vec<UpperItem>),
    Lower(Vec<LowerItem>),
    Lyrics(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum UpperItem {
    OctaveMarker { marker: String, position: Option<Position> },
    Tala { marker: String, position: Option<Position> },
    Ornament { pitches: Vec<String>, position: Option<Position> },
    Chord { chord: String, position: Option<Position> },
    Slur { underscores: String, position: Option<Position> },
    Space { count: usize, position: Option<Position> },
}

#[derive(Debug, Clone)]
pub enum LowerItem {
    OctaveMarker { marker: String, position: Option<Position> },
    KommalIndicator { position: Option<Position> },
    BeatGrouping { underscores: String, position: Option<Position> },
    Space { count: usize, position: Option<Position> },
}
```

#### 2.2 Final AST (Phase 2 Output)
```rust
// src/ast/mod.rs - MODIFIED
#[derive(Debug, Clone)]
pub struct Stave {
    pub upper_lines: Vec<AnnotationLine>,
    pub content_line: ContentLine,
    pub lower_lines: Vec<AnnotationLine>,
    pub lyrics_lines: Vec<LyricsLine>,
    pub position: Option<Position>,
}

// Keep existing AnnotationLine, but populate from RawAnnotationLine
```

### Phase 3: Parser Implementation

#### 3.1 Updated Parser Logic
```rust
// src/parser.rs - MAJOR MODIFICATIONS

impl DocumentBuilder {
    fn process_stave(&mut self, pair: Pair<Rule>) -> Result<Stave, ParseError> {
        let position = extract_position(&pair);
        let raw_stave = self.parse_raw_stave(pair)?;
        let final_stave = self.classify_raw_stave(raw_stave)?;
        Ok(final_stave)
    }
    
    fn parse_raw_stave(&mut self, pair: Pair<Rule>) -> Result<RawStave, ParseError> {
        let mut raw_stave = RawStave {
            pre_content_lines: Vec::new(),
            content_line: ContentLine { line_number: None, measures: Vec::new() },
            post_content_lines: Vec::new(),
            position: extract_position(&pair),
        };
        
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::pre_content_line => {
                    let raw_line = self.parse_pre_content_line(inner_pair)?;
                    raw_stave.pre_content_lines.push(raw_line);
                }
                Rule::content_line => {
                    raw_stave.content_line = self.process_content_line(inner_pair)?;
                }
                Rule::post_content_line => {
                    let raw_line = self.parse_post_content_line(inner_pair)?;
                    raw_stave.post_content_lines.push(raw_line);
                }
                _ => {}
            }
        }
        
        Ok(raw_stave)
    }
    
    fn parse_pre_content_line(&mut self, pair: Pair<Rule>) -> Result<RawAnnotationLine, ParseError> {
        // Parse using upper_grammar context
        for inner_pair in pair.into_inner() {
            if inner_pair.as_rule() == Rule::upper_grammar {
                return Ok(RawAnnotationLine {
                    content: RawAnnotationContent::Upper(self.parse_upper_items(inner_pair)?),
                    position: extract_position(&pair),
                });
            }
        }
        Err(/* error */)
    }
    
    fn parse_post_content_line(&mut self, pair: Pair<Rule>) -> Result<RawAnnotationLine, ParseError> {
        // Parse using lower_grammar or lyrics_grammar context
        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::lower_grammar => {
                    return Ok(RawAnnotationLine {
                        content: RawAnnotationContent::Lower(self.parse_lower_items(inner_pair)?),
                        position: extract_position(&pair),
                    });
                }
                Rule::lyrics_grammar => {
                    return Ok(RawAnnotationLine {
                        content: RawAnnotationContent::Lyrics(self.parse_lyrics_items(inner_pair)?),
                        position: extract_position(&pair),
                    });
                }
                _ => {}
            }
        }
        Err(/* error */)
    }
}
```

#### 3.2 Classification Logic (Phase 2)
```rust
// src/classifier.rs - NEW FILE

pub fn classify_raw_stave(raw_stave: RawStave) -> Result<Stave, ClassificationError> {
    let mut stave = Stave {
        upper_lines: Vec::new(),
        content_line: raw_stave.content_line,
        lower_lines: Vec::new(),
        lyrics_lines: Vec::new(),
        position: raw_stave.position,
    };
    
    // Convert pre-content lines to upper_lines
    for raw_line in raw_stave.pre_content_lines {
        match raw_line.content {
            RawAnnotationContent::Upper(items) => {
                let annotation_line = convert_upper_items_to_annotation_line(items, raw_line.position)?;
                stave.upper_lines.push(annotation_line);
            }
            RawAnnotationContent::Lyrics(syllables) => {
                let lyrics_line = LyricsLine { syllables };
                stave.lyrics_lines.push(lyrics_line);
            }
            _ => return Err(ClassificationError::InvalidPreContentLine),
        }
    }
    
    // Convert post-content lines to lower_lines/lyrics_lines
    for raw_line in raw_stave.post_content_lines {
        match raw_line.content {
            RawAnnotationContent::Lower(items) => {
                let annotation_line = convert_lower_items_to_annotation_line(items, raw_line.position)?;
                stave.lower_lines.push(annotation_line);
            }
            RawAnnotationContent::Lyrics(syllables) => {
                let lyrics_line = LyricsLine { syllables };
                stave.lyrics_lines.push(lyrics_line);
            }
            _ => return Err(ClassificationError::InvalidPostContentLine),
        }
    }
    
    Ok(stave)
}

fn convert_upper_items_to_annotation_line(
    items: Vec<UpperItem>, 
    position: Option<Position>
) -> Result<AnnotationLine, ClassificationError> {
    let mut annotation_items = Vec::new();
    
    for item in items {
        let annotation_item = match item {
            UpperItem::OctaveMarker { marker, position } => {
                AnnotationItem::UpperOctaveMarker { marker, position }
            }
            UpperItem::Tala { marker, position } => {
                AnnotationItem::Tala { marker, position }
            }
            UpperItem::Ornament { pitches, position } => {
                AnnotationItem::Ornament { pitches, position }
            }
            UpperItem::Chord { chord, position } => {
                AnnotationItem::Chord { chord, position }
            }
            UpperItem::Slur { underscores, position } => {
                AnnotationItem::Slur { underscores, position }
            }
            UpperItem::Space { count, position } => {
                AnnotationItem::Space { count, position }
            }
        };
        annotation_items.push(annotation_item);
    }
    
    Ok(AnnotationLine { items: annotation_items })
}

// Similar function for convert_lower_items_to_annotation_line...

#[derive(Debug)]
pub enum ClassificationError {
    InvalidPreContentLine,
    InvalidPostContentLine,
    PositionLost,
}
```

### Phase 4: Grammar Template Updates

#### 4.1 Update Main Template
```pest
// grammar/notation.pest.template

// Replace existing stave rule
stave = {
    pre_content_line* ~
    content_line ~
    post_content_line*
}

pre_content_line = {
    !content_line ~ upper_grammar ~ newline
}

post_content_line = {
    !content_line ~ post_content_grammar ~ newline
}

post_content_grammar = {
    lower_grammar | lyrics_grammar
}

// Strengthen content_line definition
content_line = {
    line_number? ~
    barline? ~
    content_measure ~ (barline ~ content_measure)* ~
    barline? ~
    newline  // REQUIRED
}

content_measure = {
    ws_opt ~ content_beat ~ (ws_req ~ content_beat)* ~ ws_opt
}

content_beat = {
    musical_element+
}

musical_element = {
    pitch | dash | begin_slur | end_slur | breath_mark
}

// Define specialized grammars
upper_grammar = { upper_item+ }
lower_grammar = { lower_item+ }
lyrics_grammar = { syllable ~ (ws+ ~ syllable)* }

upper_item = {
    upper_octave_marker | tala | ornament | chord | mordent | ending | slur | " "+
}

lower_item = {
    lower_octave_marker | kommal_indicator | beat_grouping | " "+
}

upper_octave_marker = { "." | "*" | ":" }
lower_octave_marker = { "." | "*" | ":" }
```

#### 4.2 System-Specific Template Updates
```pest
// grammar/system-specific.template

// Replace system-specific stave rules
{{SYSTEM}}_stave = {
    pre_content_line* ~
    {{SYSTEM}}_content_line ~
    post_content_line*
}

{{SYSTEM}}_content_line = {
    line_number? ~
    barline? ~
    {{SYSTEM}}_content_measure ~ (barline ~ {{SYSTEM}}_content_measure)* ~
    barline? ~
    newline
}

{{SYSTEM}}_content_measure = {
    ws_opt ~ {{SYSTEM}}_content_beat ~ (ws_req ~ {{SYSTEM}}_content_beat)* ~ ws_opt
}

{{SYSTEM}}_content_beat = {
    {{SYSTEM}}_musical_element+
}

{{SYSTEM}}_musical_element = {
    {{PITCH_RULE}} | dash | begin_slur | end_slur | breath_mark
}
```

### Phase 5: Testing Strategy

#### 5.1 Grammar Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_line_specificity() {
        // Should match
        assert_parses!(content_line, "1 2 3");
        assert_parses!(content_line, "S R G");
        assert_parses!(content_line, "C D E");
        
        // Should NOT match (avoid false positives)
        assert_fails!(content_line, ".");
        assert_fails!(content_line, "<123>");
        assert_fails!(content_line, "[Am]");
        assert_fails!(content_line, "___");
        assert_fails!(content_line, "la ti do");
    }
    
    #[test]
    fn test_position_aware_parsing() {
        let input = ".\n1 2\n.";
        let stave = parse_stave(input).unwrap();
        
        // First dot should be upper octave
        assert_eq!(stave.upper_lines.len(), 1);
        assert!(matches!(
            stave.upper_lines[0].items[0],
            AnnotationItem::UpperOctaveMarker { .. }
        ));
        
        // Second dot should be lower octave
        assert_eq!(stave.lower_lines.len(), 1);
        assert!(matches!(
            stave.lower_lines[0].items[0],
            AnnotationItem::LowerOctaveMarker { .. }
        ));
    }
    
    #[test]
    fn test_position_preservation() {
        let input = "1\n.";
        let stave = parse_stave(input).unwrap();
        
        // Verify position data preserved
        let lower_marker = &stave.lower_lines[0].items[0];
        if let AnnotationItem::LowerOctaveMarker { position: Some(pos), .. } = lower_marker {
            assert_eq!(pos.row, 2);
            assert_eq!(pos.col, 1);
        } else {
            panic!("Expected LowerOctaveMarker with position");
        }
    }
}
```

#### 5.2 Integration Tests
```rust
#[test]
fn test_original_problem_case() {
    let input = "1\n.";
    let result = parse_notation(input, "number");
    
    assert!(result.is_ok());
    let document = result.unwrap();
    
    // Should have one stave
    assert_eq!(document.staves.len(), 1);
    let stave = &document.staves[0];
    
    // Content line should have "1"
    assert_eq!(stave.content_line.measures.len(), 1);
    // Lower line should have "." as octave marker
    assert_eq!(stave.lower_lines.len(), 1);
    assert!(matches!(
        stave.lower_lines[0].items[0],
        AnnotationItem::LowerOctaveMarker { marker, .. } if marker == "."
    ));
}
```

## Implementation Timeline

1. **Week 1**: Grammar architecture and templates
2. **Week 2**: Raw AST structures and basic parser updates
3. **Week 3**: Classification logic and final AST conversion
4. **Week 4**: Integration, testing, and error handling refinement

## Success Metrics

- `"1\n."` parses correctly as content + lower octave marker
- All existing notation examples continue to work
- Error messages maintain line/column precision
- Grammar is more maintainable with less duplication
- Classification logic handles edge cases gracefully

This approach leverages **position-aware grammar selection** while maintaining the benefits of specialized parsing contexts and clean separation of concerns.