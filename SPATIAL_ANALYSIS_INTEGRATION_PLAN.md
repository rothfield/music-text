# Spatial Analysis Integration Plan

## Overview

This document outlines the plan to integrate spatial analysis functionality from the old codebase with the current simple parser architecture, enabling syllable-to-note assignment using move semantics validation.

## Current State Analysis

### What We Have
1. **Current Simple Parser** (`src/parse/recursive_descent_simple.rs`)
   - Single-line parsing only
   - Creates `ParsedElement` tokens
   - No multi-line awareness
   - Perfect roundtrip validation for single lines

2. **Old Spatial Analysis Code** (restored via git checkout)
   - Complete multi-line parser in `src/spatial_parser.rs`
   - `assign_syllables_to_notes()` function
   - Spatial positioning algorithms
   - Multi-dimensional source consumption tracking

3. **Move Semantics Pattern** (from `specs/music-text-as-code-editor.md`)
   - `Source.value: Option<String>` - None when consumed
   - Physical content movement prevents double-counting
   - Perfect roundtrip validation with spatial relationships

## Problem Statement

**User Input**: `"|1\nhi"`
**Expected Result**: Syllable "hi" should be spatially assigned to Note "1"
**Current Behavior**: Simple parser ignores the second line entirely

## Integration Strategy

### Phase 1: Multi-Line Document Structure

#### 1.1 Extend ParsedElement for Multi-Line Content
```rust
// Add to src/parse/recursive_descent_simple.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedElement {
    // Existing single-line elements...
    Note { value: String, position: Position },
    Barline { style: String, position: Position },

    // New multi-line elements
    LyricsLine { syllables: Vec<Syllable>, position: Position },
    Syllable { content: String, source: Source }, // Uses move semantics
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLineDocument {
    pub content_lines: Vec<Vec<ParsedElement>>,    // Musical content
    pub lyrics_lines: Vec<Vec<ParsedElement>>,     // Syllable content
    pub original_lines: Vec<OriginalLine>,         // For roundtrip validation
}
```

#### 1.2 Multi-Line Parser Function
```rust
// New function in src/parse/recursive_descent_simple.rs
pub fn parse_multi_line_document(input: &str) -> Result<MultiLineDocument, String> {
    let lines: Vec<&str> = input.lines().collect();
    let mut content_lines = Vec::new();
    let mut lyrics_lines = Vec::new();
    let mut original_lines = Vec::new();

    for (line_num, line) in lines.iter().enumerate() {
        let original_line = OriginalLine {
            content: line.to_string(),
            line_number: line_num + 1,
            include_in_roundtrip: true, // Will be set to false when processed
        };
        original_lines.push(original_line);

        if line.trim_start().starts_with('|') {
            // Musical content line
            let elements = parse_main_line(line)?;
            content_lines.push(elements);
        } else if !line.trim().is_empty() {
            // Potential lyrics line
            let syllables = parse_lyrics_line(line)?;
            lyrics_lines.push(syllables);
        }
        // Empty lines are preserved in original_lines for roundtrip
    }

    Ok(MultiLineDocument {
        content_lines,
        lyrics_lines,
        original_lines,
    })
}

fn parse_lyrics_line(line: &str) -> Result<Vec<ParsedElement>, String> {
    let mut elements = Vec::new();
    let mut position = 1;

    // Split on whitespace to get syllables
    for word in line.split_whitespace() {
        elements.push(ParsedElement::Syllable {
            content: word.to_string(),
            source: Source {
                value: Some(word.to_string()), // Available for move semantics
                position: Position { row: 1, col: position },
            },
        });
        position += word.len() + 1; // Account for whitespace
    }

    Ok(elements)
}
```

### Phase 2: Spatial Analysis Integration

#### 2.1 Adapt Old Spatial Logic
```rust
// New function combining old logic with new data structures
pub fn assign_syllables_with_move_semantics(
    content_lines: &mut [Vec<ParsedElement>],
    lyrics_lines: &mut [Vec<ParsedElement>],
    original_lines: &mut [OriginalLine]
) -> Result<(), String> {

    // Collect all available syllables (before consumption)
    let mut available_syllables = Vec::new();
    for (line_idx, lyrics_line) in lyrics_lines.iter_mut().enumerate() {
        for element in lyrics_line.iter_mut() {
            if let ParsedElement::Syllable { content, source } = element {
                if source.value.is_some() {
                    available_syllables.push((line_idx, content.clone(), source));
                }
            }
        }
    }

    // Assign syllables to notes spatially
    let mut syllable_index = 0;
    for content_line in content_lines.iter_mut() {
        for element in content_line.iter_mut() {
            if let ParsedElement::Note { value, position } = element {
                if syllable_index < available_syllables.len() {
                    let (lyrics_line_idx, syllable_content, syllable_source) =
                        &mut available_syllables[syllable_index];

                    // MOVE SEMANTICS: Transfer content from syllable to note
                    if let Some(moved_content) = syllable_source.value.take() {
                        // Add syllable field to Note (extend ParsedElement::Note)
                        // This requires modifying the Note variant to include syllable

                        // Mark the original lyrics line as processed
                        original_lines[*lyrics_line_idx].include_in_roundtrip = false;

                        syllable_index += 1;
                    }
                }
            }
        }
    }

    Ok(())
}
```

#### 2.2 Extend Note Structure for Syllables
```rust
// Modify ParsedElement::Note to include syllable assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedElement {
    Note {
        value: String,
        position: Position,
        syllable: Option<String>, // Moved from lyrics line using move semantics
        syllable_source: Option<Source>, // Track where syllable came from
    },
    // ... other variants
}
```

### Phase 3: Roundtrip Validation with Move Semantics

#### 3.1 Enhanced Reconstruction Function
```rust
// Update src/web_server.rs reconstruction function
fn reconstruct_text_from_multi_line_document(document: &MultiLineDocument) -> String {
    let mut result = String::new();

    // Only include lines that weren't fully processed (include_in_roundtrip = true)
    for original_line in &document.original_lines {
        if original_line.include_in_roundtrip {
            result.push_str(&original_line.content);
            result.push('\n');
        }
    }

    // Remove trailing newline if present
    if result.ends_with('\n') {
        result.pop();
    }

    result
}
```

#### 3.2 Perfect Validation Example
```
Input: "|1\nhi"

After Parsing:
- content_lines: [Note{value: "1", syllable: None}]
- lyrics_lines: [Syllable{content: "hi", source.value: Some("hi")}]
- original_lines: [
    {content: "|1", include_in_roundtrip: true},
    {content: "hi", include_in_roundtrip: true}
  ]

After Spatial Analysis:
- content_lines: [Note{value: "1", syllable: Some("hi")}]
- lyrics_lines: [Syllable{content: "hi", source.value: None}] // MOVED!
- original_lines: [
    {content: "|1", include_in_roundtrip: true},
    {content: "hi", include_in_roundtrip: false} // CONSUMED!
  ]

Reconstruction: "|1" (only unprocessed content)
Validation: ✅ PASS - "hi" was successfully consumed and assigned to note
```

### Phase 4: API Integration

#### 4.1 Update Parse Entry Point
```rust
// Modify src/lib.rs parse function
pub fn parse(input: &str, system: Option<&str>) -> NotationResult {
    let system = system.unwrap_or("auto");

    // Use multi-line parser instead of single-line
    match parse_multi_line_document(input) {
        Ok(mut document) => {
            // Apply spatial analysis with move semantics
            assign_syllables_with_move_semantics(
                &mut document.content_lines,
                &mut document.lyrics_lines,
                &mut document.original_lines
            )?;

            // Convert to old AST format for compatibility
            let compatible_document = convert_to_legacy_ast(&document);

            // Continue with existing pipeline...
            NotationResult {
                success: true,
                document: Some(compatible_document),
                // ... rest of fields
            }
        }
        Err(e) => {
            NotationResult {
                success: false,
                error_message: Some(e),
                // ... rest of fields
            }
        }
    }
}
```

#### 4.2 Legacy AST Conversion
```rust
// Convert new multi-line structure to old AST for renderer compatibility
fn convert_to_legacy_ast(document: &MultiLineDocument) -> ast::Document {
    let mut staves = Vec::new();

    for content_line in &document.content_lines {
        let mut measures = Vec::new();
        let mut current_measure = Vec::new();

        for element in content_line {
            match element {
                ParsedElement::Note { value, syllable, .. } => {
                    current_measure.push(ast::BeatElement::Pitch {
                        value: value.clone(),
                        syllable: syllable.clone(), // Transferred via move semantics!
                        octave: 0, // Default, can be enhanced later
                        accidental: None,
                        slur_type: None,
                        subdivisions: None,
                        is_tied: None,
                    });
                }
                ParsedElement::Barline { .. } => {
                    if !current_measure.is_empty() {
                        measures.push(ast::Beat { elements: current_measure });
                        current_measure = Vec::new();
                    }
                }
                _ => {} // Handle other element types
            }
        }

        // Add final measure if present
        if !current_measure.is_empty() {
            measures.push(ast::Beat { elements: current_measure });
        }

        staves.push(ast::Stave {
            content_line: ast::ContentLine {
                measures: measures.into_iter().map(|beat| ast::Measure {
                    beats: vec![beat]
                }).collect(),
            },
            lyrics_lines: Vec::new(), // Empty - syllables moved to notes
            upper_lines: Vec::new(),
            lower_lines: Vec::new(),
        });
    }

    ast::Document {
        staves,
        // ... other fields
    }
}
```

## Implementation Order

### Step 1: Multi-Line Document Structure
- [ ] Add `MultiLineDocument` struct to `src/parse/recursive_descent_simple.rs`
- [ ] Implement `parse_multi_line_document()` function
- [ ] Add `parse_lyrics_line()` function
- [ ] Extend `ParsedElement` enum with `Syllable` variant

### Step 2: Move Semantics Integration
- [ ] Add `syllable` field to `ParsedElement::Note`
- [ ] Implement `assign_syllables_with_move_semantics()` function
- [ ] Update `OriginalLine.include_in_roundtrip` logic

### Step 3: Roundtrip Validation
- [ ] Update `reconstruct_text_from_document()` in `src/web_server.rs`
- [ ] Test roundtrip validation with multi-line input
- [ ] Verify move semantics prevent double-counting

### Step 4: API Integration
- [ ] Modify `parse()` function in `src/lib.rs`
- [ ] Implement `convert_to_legacy_ast()` function
- [ ] Update web server response handling

### Step 5: Testing & Validation
- [ ] Test with input `"|1\nhi"` - verify "hi" attaches to Note "1"
- [ ] Test roundtrip validation shows only unprocessed content
- [ ] Test complex cases: multiple notes, multiple syllables
- [ ] Verify existing single-line functionality still works

## Expected Outcome

**Input**: `"|1\nhi"`

**Parse Result**:
```json
{
  "success": true,
  "document": {
    "staves": [{
      "content_line": {
        "measures": [{
          "beats": [{
            "elements": [{
              "Pitch": {
                "value": "1",
                "syllable": "hi",  // ← Moved from lyrics line!
                "octave": 0
              }
            }]
          }]
        }]
      },
      "lyrics_lines": []  // ← Empty - content moved to notes
    }]
  }
}
```

**Roundtrip Validation**: `"|1"` (only unprocessed content - "hi" was consumed)

**Verification**: ✅ Perfect spatial assignment with move semantics validation

## Benefits

1. **Spatial Accuracy**: Syllables correctly assigned to notes based on positioning
2. **Move Semantics**: Physical content transfer prevents double-counting
3. **Perfect Validation**: Roundtrip shows exactly what content wasn't processed
4. **Backward Compatibility**: Existing single-line parsing continues to work
5. **Legacy Integration**: Converts to old AST format for renderer compatibility

This plan bridges the old spatial analysis capabilities with the current simple parser architecture while implementing the move semantics pattern for perfect roundtrip validation.