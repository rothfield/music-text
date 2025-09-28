# Semantic Markup Rendering Specification

## Overview

Replace character-by-character styling with a normalized array approach that generates semantic styling based on the parsed grammar structure. This creates clean separation between lexical tokens and semantic annotations.

## Core Architecture

**Single Tree Walk → Normalized Array → Two Outputs**

1. **Parse tree**: Rhythm-analyzed document from parser
2. **Normalized array**: Intermediate representation with semantic annotations
3. **Two outputs**: Tokens (lexical) + Styles (semantic)

## Normalized Array Structure

Each element in the normalized array represents one source-mapped token with semantic annotations:

```rust
struct NormalizedElement {
    tag: String,           // Grammar element type: "note", "dash", "barline", etc.
    pos: usize,           // Character position in source
    length: usize,        // Character length
    content: String,      // Original text content
    classes: Vec<String>, // Semantic classes: ["beat-loop-4", "tuplet-3"]
}
```

### Example

**Input**: `"1234 56"`

**Normalized Array**:
```json
[
  {"tag": "note", "pos": 0, "length": 1, "content": "1", "classes": ["beat-loop-4"]},
  {"tag": "note", "pos": 1, "length": 1, "content": "2", "classes": []},
  {"tag": "note", "pos": 2, "length": 1, "content": "3", "classes": []},
  {"tag": "note", "pos": 3, "length": 1, "content": "4", "classes": []},
  {"tag": "whitespace", "pos": 4, "length": 1, "content": " ", "classes": []},
  {"tag": "note", "pos": 5, "length": 1, "content": "5", "classes": ["beat-loop-2"]},
  {"tag": "note", "pos": 6, "length": 1, "content": "6", "classes": []}
]
```

## Output Generation

From the normalized array, generate two separate outputs:

### Tokens Output (Lexical Information)
```json
{
  "tokens": [
    {"pos": 0, "length": 1, "type": "note", "content": "1"},
    {"pos": 1, "length": 1, "type": "note", "content": "2"},
    {"pos": 2, "length": 1, "type": "note", "content": "3"},
    {"pos": 3, "length": 1, "type": "note", "content": "4"},
    {"pos": 5, "length": 1, "type": "note", "content": "5"},
    {"pos": 6, "length": 1, "type": "note", "content": "6"}
  ]
}
```

### Styles Output (Semantic Annotations)
```json
{
  "character_styles": [
    {"pos": 0, "length": 1, "classes": ["cm-music-note", "beat-loop-4"]},
    {"pos": 1, "length": 1, "classes": ["cm-music-note"]},
    {"pos": 2, "length": 1, "classes": ["cm-music-note"]},
    {"pos": 3, "length": 1, "classes": ["cm-music-note"]},
    {"pos": 5, "length": 1, "classes": ["cm-music-note", "beat-loop-2"]},
    {"pos": 6, "length": 1, "classes": ["cm-music-note"]}
  ]
}
```

## Semantic Class Generation

### Beat Loop Classes

For beats with multiple elements, add `beat-loop-N` class to the first element:

- Single note beat: No loop class
- Multi-note beat: First element gets `beat-loop-N` where N = element count

**Algorithm**:
1. Walk rhythm-analyzed document beats
2. For each beat with elements.len() > 1:
   - Add `beat-loop-{elements.len()}` to first element's classes
3. For single-element beats: No additional classes

### Future Semantic Classes

- `tuplet-N`: For tuplet groupings
- `slur-start`, `slur-end`: For slur markings
- `tied`: For tied notes
- `grace`: For grace notes

## CSS Implementation

Pre-generate CSS classes for common beat sizes:

```css
/* Base semantic styling */
.cm-music-note {
    color: #22863a;
    font-weight: bold;
    position: relative;
}

/* Beat loop arcs */
.cm-music-note.beat-loop-2::after { width: 2ch; }
.cm-music-note.beat-loop-3::after { width: 3ch; }
.cm-music-note.beat-loop-4::after { width: 4ch; }
/* ... continue pattern up to reasonable limit */

/* Common arc styling */
.cm-music-note[class*="beat-loop-"]::after {
    content: '';
    position: absolute;
    bottom: -0.8em;
    left: 0;
    height: 0.6em;
    border: 2px solid #ff9800;
    border-top: none;
    border-bottom-left-radius: 50%;
    border-bottom-right-radius: 50%;
    z-index: 1;
    pointer-events: none;
}
```

## Frontend Integration

### CodeMirror Application

Apply both token-based and style-based classes:

```javascript
// Apply base token classes
tokens.forEach(token => {
    const from = editor.posFromIndex(token.pos);
    const to = editor.posFromIndex(token.pos + token.length);
    editor.markText(from, to, {
        className: `cm-music-${token.type}`
    });
});

// Apply semantic style classes
characterStyles.forEach(style => {
    const from = editor.posFromIndex(style.pos);
    const to = editor.posFromIndex(style.pos + style.length);
    editor.markText(from, to, {
        className: style.classes.join(' ')
    });
});
```

### Result in DOM

For "1234":
```html
<span class="cm-music-note beat-loop-4">1</span>
<span class="cm-music-note">2</span>
<span class="cm-music-note">3</span>
<span class="cm-music-note">4</span>
```

CSS automatically creates a 4ch-wide orange arc under the entire beat.

## Implementation Algorithm

### Single Tree Walk

```rust
fn generate_normalized_elements(rhythm_doc: &RhythmAnalyzedDocument) -> Vec<NormalizedElement> {
    let mut elements = Vec::new();

    for beat in &rhythm_doc.beats {
        let beat_size = beat.elements.len();

        for (i, element) in beat.elements.iter().enumerate() {
            let mut classes = Vec::new();

            // Add beat loop class to first element of multi-element beats
            if i == 0 && beat_size > 1 {
                classes.push(format!("beat-loop-{}", beat_size));
            }

            // Add other semantic classes (tuplets, etc.)
            if beat.is_tuplet {
                classes.push(format!("tuplet-{}", beat.tuplet_ratio));
            }

            elements.push(NormalizedElement {
                tag: element.element_type.clone(),
                pos: element.source_position.absolute_offset,
                length: element.content.len(),
                content: element.content.clone(),
                classes,
            });
        }
    }

    elements
}
```

### Output Generation

```rust
fn generate_tokens_and_styles(elements: &[NormalizedElement]) -> (Vec<TokenInfo>, Vec<CharacterStyle>) {
    let tokens = elements.iter()
        .filter(|e| e.tag != "whitespace")  // Skip whitespace in tokens
        .map(|e| TokenInfo {
            pos: e.pos,
            length: e.length,
            token_type: e.tag.clone(),
            content: e.content.clone(),
        })
        .collect();

    let styles = elements.iter()
        .filter(|e| e.tag != "whitespace")  // Skip whitespace in styles
        .map(|e| {
            let mut classes = vec![format!("cm-music-{}", e.tag)];
            classes.extend(e.classes.clone());

            CharacterStyle {
                pos: e.pos,
                length: e.length,
                classes,
            }
        })
        .collect();

    (tokens, styles)
}
```

## Benefits

1. **Single source of truth**: Normalized array contains all information
2. **Clean separation**: Lexical vs semantic concerns separated
3. **Easy to understand**: Clear data flow from tree to outputs
4. **Extensible**: Just add classes to normalized elements
5. **Efficient**: One tree walk, simple transforms
6. **Debuggable**: Can inspect normalized array
7. **Semantic clarity**: Beat boundaries explicit in markup

## Migration

1. **Phase 1**: Implement normalized array generation alongside existing system
2. **Phase 2**: Update API to return both tokens and styles
3. **Phase 3**: Update frontend to use new format
4. **Phase 4**: Remove old character-by-character system