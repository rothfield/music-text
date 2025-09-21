# Music Text as Code Editor Specification

## Overview

This specification defines a code editor approach for music-text notation that provides syntax highlighting with perfect columnar alignment. Unlike traditional rich text editors, this system maintains a plain-text data model while providing character-level visual styling through server-generated parse trees.

## Core Problem

Music notation requires **columnar alignment** across multiple lines:
```
|S R G M|P D N S|
 Ly-rics go here
 .   .   .   .
```

With proportional fonts, characters have different widths, breaking alignment. The solution is to wrap every character in fixed-width spans for precise column control.

## Architecture

### High-Level Flow
```
User Types → Editor (Plain Text) → Server Parse → Character Tree → Styled Rendering
     ↑                                                                      ↓
     └─────────────── Round-trip Validation ←─────────────────────────────┘
```

## Consumed Elements in the Editor Pipeline

**TODO**: Explain how to figure out the leaf elements of the document.

### The Challenge: 2D to 1D Rendering
Music-text supports 2D spatial notation where elements on different lines modify the main content:
```
  . .            <- upper line (octave markers, ornaments, etc.)
|1234|           <- content line (notes)
    .            <- lower line (octave markers, beat groups, syllables, etc.)
```

The spatial assignment system uses **move semantics** to consume these spatial elements, transferring ownership to the notes they affect.

### Conceptual Model: Pointers in 2D Grammar

Consumed elements can be thought of as having **pointers** connecting them to their owning objects:

```
Source Text:          Logical Pointers:        Musical Result:
  . .                [dot@2]──┐ [dot@4]──┐
|1234|                         ↓        ↓      Note "1": C in upper octave
    .                        [1+1]    [3+1]    Note "2": D in middle octave
                                 ↑              Note "3": E in upper octave
                            [dot@10]──┘         Note "4": F in lower octave
```

### Implementation Requirements

The renderer must generate spans for **ALL leaf elements** in the document parse tree:

1. **Universal Leaf Element Rendering**:
   - Generate spans for every leaf element
   - Apply standard span generation logic uniformly
   - Add `consumed="true"` attribute only for consumed elements

2. **Unified Span Generation**:
   ```rust
   // For ALL leaf elements:
   Span {
       start: element.source.position.index_in_doc,
       end: start + element.source.value.len(),
       type: element.type,
       content: element.source.value,
       consumed: element.is_consumed()  // true for consumed elements only
   }
   ```

3. **CSS handles visibility**:
   ```css
   /* Consumed elements: invisible but space-preserving */
   [consumed="true"] {
       opacity: 0;
       pointer-events: auto; /* Still editable */
   }

   /* Toggle to show consumed elements */
   .show-consumed [consumed="true"] {
       opacity: 0.3;
       color: red;
   }
   ```

### Key Insight

**Every character in the source gets exactly one span** - whether it's a regular element or a consumed element. The consumed elements are rendered identically to other elements, just marked with `consumed="true"` so CSS can handle their visibility.

### Open Questions

1. **Graceful Fallback**: What renders when spatial elements cannot be consumed?
2. **Editor Attributes**: Additional attributes needed for editor features?

### Components

1. **Client Editor**: CodeMirror with plain-text content
2. **Server Parser**: Rust parser that returns character-level styling trees
3. **Styling Engine**: Client-side renderer that applies character-level spans
4. **Race Condition Handler**: Version control for async styling updates

## Server API Enhancement

### Current Response Structure
```json
{
  "success": true,
  "parsed_document": { ... },
  "vexflow": { ... },
  "lilypond": "..."
}
```

### Enhanced Response Structure
```json
{
  "success": true,
  "parsed_document": { ... },
  "vexflow": { ... },
  "lilypond": "...",
  "editor_tree": {
    "content_hash": "abc123...",
    "original_text": "|S R G M|\nLyrics here",
    "lines": [
      {
        "line_num": 1,
        "line_type": "content",
        "chars": [
          {"col": 1, "char": "|", "type": "barline"},
          {"col": 2, "char": "S", "type": "sargam-note", "octave": 0},
          {"col": 3, "char": " ", "type": "space"},
          {"col": 4, "char": "R", "type": "sargam-note", "octave": 0},
          {"col": 5, "char": " ", "type": "space"},
          {"col": 6, "char": "G", "type": "sargam-note", "octave": 0},
          {"col": 7, "char": " ", "type": "space"},
          {"col": 8, "char": "M", "type": "sargam-note", "octave": 0},
          {"col": 9, "char": "|", "type": "barline"}
        ]
      },
      {
        "line_num": 2,
        "line_type": "lyrics", 
        "chars": [
          {"col": 2, "char": "L", "type": "syllable-start", "note_ref": 2},
          {"col": 3, "char": "y", "type": "syllable-cont", "note_ref": 2},
          {"col": 4, "char": "r", "type": "syllable-start", "note_ref": 4},
          {"col": 5, "char": "i", "type": "syllable-cont", "note_ref": 4},
          {"col": 6, "char": "c", "type": "syllable-cont", "note_ref": 4},
          {"col": 7, "char": "s", "type": "syllable-end", "note_ref": 4},
          {"col": 8, "char": " ", "type": "space"},
          {"col": 9, "char": "h", "type": "syllable-start", "note_ref": 6},
          {"col": 10, "char": "e", "type": "syllable-cont", "note_ref": 6},
          {"col": 11, "char": "r", "type": "syllable-cont", "note_ref": 6},
          {"col": 12, "char": "e", "type": "syllable-end", "note_ref": 6}
        ]
      }
    ]
  }
}
```

## Character Types and Styling

### Content Line Character Types
- `barline`: `|`, `||`, `|:`, `:|`
- `sargam-note`: `S`, `R`, `G`, `M`, `P`, `D`, `N`
- `number-note`: `1`, `2`, `3`, `4`, `5`, `6`, `7`  
- `western-note`: `C`, `D`, `E`, `F`, `G`, `A`, `B`
- `sharp`: `#`
- `flat`: `b`, `♭`
- `octave-up`: `:` (after note)
- `octave-down`: `.` (after note)
- `dash`: `-` (extension)
- `space`: ` ` (spacing)
- `breath`: `'` (breath mark)

### Lyrics Line Character Types
- `syllable-start`: First character of syllable
- `syllable-cont`: Middle characters of syllable
- `syllable-end`: Last character of syllable
- `syllable-sep`: `-` (syllable separator)
- `space`: ` ` (spacing)

### Directive Line Character Types
- `directive-key`: `Title`, `Composer`, etc.
- `directive-sep`: `:` (separator)
- `directive-value`: Value after colon
- `space`: ` ` (spacing)

## CSS Styling Framework

### Brute Force CSS Approach for Beat Group Arcs

The current implementation uses a **brute force CSS approach** for beat group arc styling, which generates 63 individual CSS classes to handle precise character width calculations.

#### Why Brute Force Over Adaptive CSS?

**Considered Approach: CSS `max-content` + `display: grid`**
```css
.beat-group-arc::after {
    display: inline-grid;
    width: max-content;       /* grow to fit content */
    border-radius: 0 0 50% 50%;  /* bottom semi-circle */
}
```

**Problem**: This requires wrapping multiple characters in a container div:
```html
<div class="beat-group-arc"><span>SRG</span></div>
```

**CodeMirror Limitation**: CodeMirror applies classes to individual characters in the token stream:
```html
<span class="cm-music-note">S</span><span class="cm-music-note">R</span><span class="cm-music-note">G</span>
```

**Solution**: Brute force CSS classes with precise monospaced font calculations.

#### Implementation Details

**1. CSS Class Generation (1-63 character widths)**
```css
/* Beat group classes for precise width control */
.cm-music-beat-group-1::after { width: 0.6em; }
.cm-music-beat-group-2::after { width: 1.2em; }
.cm-music-beat-group-3::after { width: 1.8em; }
/* ... continues to 63 */
.cm-music-beat-group-63::after { width: 37.8em; }

/* Base styling for all beat group classes */
.cm-music-beat-group-1, .cm-music-beat-group-2, /* ... all 63 classes */ {
    position: relative;
    color: #22863a;
    font-weight: bold;
}

.cm-music-beat-group-1::after, .cm-music-beat-group-2::after, /* ... all 63 */ {
    content: '';
    position: absolute;
    bottom: 1.05em;
    left: 0;
    border-bottom: 0.1em solid #0366d6;
    border-bottom-left-radius: 0.8em;
    border-bottom-right-radius: 0.8em;
    padding-bottom: 0.75em;
    z-index: 1;
    pointer-events: none;
}
```

**2. Character Width Calculation**
```rust
fn add_beat_group_classes(styles: &mut Vec<CharacterStyle>, positions: &[usize], count: usize) {
    for (i, &pos) in positions.iter().enumerate() {
        if let Some(style) = styles.iter_mut().find(|s| s.pos == pos) {
            if i == 0 {  // Only apply to start note
                // Calculate character span of the beat group
                let start_pos = positions[0];
                let end_pos = positions[positions.len() - 1];
                let char_width = end_pos - start_pos + 1; // +1 to include end character

                // Clamp to available classes (1-63)
                let clamped_width = char_width.min(63).max(1);

                // Apply the specific numbered class
                style.classes.push(format!("beat-group-{}", clamped_width));
            }
        }
    }
}
```

**3. Monospaced Font Precision**
- Each character = 0.6em width in monospaced font
- Arc width = character count × 0.6em
- Perfect alignment guaranteed for any beat group span

#### Benefits of Brute Force Approach

1. **CodeMirror Compatibility**: Works with token-based character styling
2. **Precise Control**: Exact width for each possible character span
3. **Performance**: No JavaScript calculations or DOM measurements
4. **Predictable**: Consistent behavior across all browsers
5. **Monospaced Optimization**: Leverages fixed character widths

#### Trade-offs

**Pros**:
- Perfect precision for monospaced fonts
- No runtime calculations needed
- Works with CodeMirror's architecture
- 63 classes covers practical use cases

**Cons**:
- More CSS code (63 classes vs 1 adaptive rule)
- Fixed to monospaced fonts only
- Upper limit of 63 characters per beat group

#### Alternative Approaches Considered

**1. JavaScript Dynamic Styling**
```javascript
// Measure actual DOM element widths
const arcWidth = endCoords.left - startCoords.left;
markElement.style.setProperty('--beat-group-width', arcWidth + 'px');
```
*Rejected*: More complex, requires DOM measurements, potential performance issues

**2. CSS Custom Properties**
```css
.beat-group-start::after {
    width: var(--beat-group-width, 2em);
}
```
*Rejected*: Still requires JavaScript to set the custom property values

**3. Multiple CSS Classes**
```css
.beat-group-start.beat-group-2::after { width: 1.2em; }
.beat-group-start.beat-group-3::after { width: 1.8em; }
```
*Rejected*: CodeMirror doesn't support multiple classes in token types

### Fixed-Width Character Layout
```css
/* Every character gets same width for columnar alignment */
.music-char {
    display: inline-block;
    width: 1.2em;
    text-align: center;
    font-size: 1.6em;
    position: relative;
}

/* Content line characters - doremi inspired */
.music-char.sargam-note,
.music-char.number-note, 
.music-char.western-note {
    font-family: sans-serif;
    font-weight: normal;
}

.music-char.barline {
    font-family: sans-serif;
    font-weight: bold;
    color: #333;
}

.music-char.sharp,
.music-char.flat {
    font-family: sans-serif;
    font-size: 1.4em;
    color: #666;
}

.music-char.space {
    /* Transparent but maintains column width */
}

.music-char.dash {
    font-family: sans-serif;
    color: #999;
}

/* Lyrics characters */
.music-char.syllable-start,
.music-char.syllable-cont,
.music-char.syllable-end {
    font-family: serif;
    font-size: 1.4em;
    color: #444;
}

.music-char.syllable-sep {
    font-family: serif;
    color: #666;
}

/* Octave indicators using pseudo-elements */
.music-char.sargam-note[data-octave="1"]::before,
.music-char.number-note[data-octave="1"]::before,
.music-char.western-note[data-octave="1"]::before {
    content: "•";
    position: absolute;
    top: -0.3em;
    left: 50%;
    transform: translateX(-50%);
    font-size: 0.6em;
}

.music-char.sargam-note[data-octave="-1"]::after,
.music-char.number-note[data-octave="-1"]::after,
.music-char.western-note[data-octave="-1"]::after {
    content: "•";
    position: absolute;
    bottom: -0.3em;
    left: 50%;
    transform: translateX(-50%);
    font-size: 0.6em;
}

/* Line-level backgrounds */
.content-line {
    background: #f0f8ff;
    padding: 0.2em 0;
}

.lyrics-line {
    background: #fffef5;
    padding: 0.2em 0;
}

.directive-line {
    background: #f1f8e9;
    padding: 0.2em 0;
    font-weight: bold;
}
```

## Move Semantics Roundtrip Validation

### The Algorithm: Jigsaw Puzzle Pattern

The music-text parser implements **move semantics roundtrip validation** - a pattern that ensures perfect parsing accuracy by tracking source consumption.

#### Core Concept
Think of the original text as a **jigsaw puzzle box** with pieces laid out in a grid:
- **Parsing** = picking up pieces and assembling them into the final picture (parse tree)
- **Perfect parsing** = empty box (all pieces used)
- **Failed parsing** = remaining pieces show exactly what couldn't be processed

#### Implementation

**1. Source Structure with Move Semantics**
```rust
pub struct Source {
    pub value: Option<String>,  // None when consumed/moved
    pub position: Position,     // Original location
}
```

**2. Line-Based Consumption Tracking** 
```rust
pub struct OriginalLine {
    pub content: String,
    pub line_number: usize,
    pub include_in_roundtrip: bool, // False when processed
}
```

**3. Parsing with Consumption**
```rust
// When parsing succeeds, mark the line as processed
if parsing_successful {
    original_line.include_in_roundtrip = false;
}

// Spatial assignment uses .take() to move content
let marker_value = source.value.take(); // Moves, leaves None
```

**4. Reconstruction Logic**
```rust
fn reconstruct_unprocessed(document: &Document) -> String {
    document.lines.iter()
        .filter(|line| line.include_in_roundtrip) // Only unprocessed
        .map(|line| &line.content)
        .collect::<Vec<_>>()
        .join("\n")
}
```

#### Validation Results

**Perfect Parsing:**
```
Input: "S R G"
Reconstruction: ""  // Empty - all content consumed
Result: ✅ PASS
```

**Partial Parsing:**
```  
Input: "S R G\nrandom text"
Reconstruction: "random text"  // Shows unprocessed content
Result: ❌ FAIL - shows exactly what failed
```

#### Benefits

1. **Perfect Accuracy**: Proves parser consumed every claimed character
2. **Precise Diagnostics**: Shows exact unprocessed content and location
3. **No False Positives**: Can't claim success while missing content
4. **Comprehensive**: Works with spatial parsing (octave markers, syllables, etc.)

### Is This a Common Pattern?

**Short Answer**: This exact pattern is **rare** but the underlying concepts are common in different forms.

#### Related Patterns in Parsing

**1. Token Consumption Tracking**
- **Antlr/Yacc**: Track consumed token positions  
- **Rust nom**: Consumes input slices, returns remaining
- **Our approach**: Tracks consumption at source level

**2. Roundtrip Testing** 
- **Prettier/Black**: Format → parse → format should be idempotent
- **Rust rustfmt**: AST → code → AST should be identical  
- **Our approach**: Original → parse → reconstruct should match

**3. Ownership-Based Resource Management**
- **Rust RAII**: Move semantics prevent double-use
- **Linear types**: Resources consumed exactly once
- **Our approach**: Source segments consumed exactly once

#### Why This Pattern is Rare

**Most parsers use different validation approaches:**

1. **AST Roundtrip**: `original → parse → serialize → compare`
   - Problem: Loses formatting, whitespace, comments
   - Our solution: Preserves exact original content

2. **Token-Level Validation**: Check all tokens consumed
   - Problem: Doesn't handle spatial relationships  
   - Our solution: Handles multi-dimensional source consumption

3. **Position Tracking**: Record start/end positions
   - Problem: Complex to verify completeness
   - Our solution: Physical consumption makes gaps obvious

#### Novel Aspects

**Our approach appears to be novel because:**

1. **Spatial Parsing**: Music notation has 2D spatial relationships (octave markers above notes)
2. **Move Semantics**: Physical consumption prevents double-counting
3. **Line Granularity**: Balances precision with simplicity
4. **Perfect Diagnostics**: Shows exactly what parsing couldn't handle

This makes it particularly valuable for **domain-specific languages** with complex spatial relationships.

## Multiple CSS Classes on Single Characters

### How Multiple Classes Are Applied

The music-text editor supports **multiple CSS classes on individual characters** through the `CharacterStyle` system, enabling complex visual effects like simultaneous syntax highlighting, beat grouping, and slur annotations.

#### Backend: CharacterStyle Structure
```rust
#[derive(Debug, Serialize, Clone)]
pub struct CharacterStyle {
    pub pos: usize,           // Character position in document
    pub classes: Vec<String>, // Multiple CSS class names
}
```

#### Class Accumulation Process
```rust
// 1. Start with basic syntax highlighting
CharacterStyle {
    pos: 0,
    classes: vec!["cm-music-note".to_string()],
}

// 2. Add beat group classes
style.classes.push("in-beat-group".to_string());
style.classes.push("beat-group-start".to_string());

// 3. Add implicit beat classes (if applicable)
style.classes.push("in-implicit-beat".to_string());
style.classes.push("implicit-beat-start".to_string());

// 4. Final result
CharacterStyle {
    pos: 0,
    classes: vec![
        "cm-music-note",
        "in-beat-group",
        "beat-group-start",
        "in-implicit-beat",
        "implicit-beat-start"
    ],
}
```

#### Frontend: CSS Class Application
```javascript
// Apply all classes to CodeMirror text mark
const mark = this.editor.markText(pos, endPos, {
    className: style.classes.join(' ')  // "cm-music-note in-beat-group beat-group-start in-implicit-beat implicit-beat-start"
});
```

#### CSS: Dynamic Width with Custom Properties
```css
/* Base syntax highlighting */
.cm-music-note {
    color: #22863a;
    font-weight: bold;
}

/* Beat group arc (blue, lower position) */
.beat-group-start::after {
    border-bottom: 0.1em solid #0366d6;
    width: var(--beat-group-width, 1em);
}

/* Implicit beat arc (orange, lower position) */
.implicit-beat-start::after {
    border-bottom: 0.1em solid #ff9800;
    width: var(--implicit-beat-width, 1.2em);
}

/* Slur arc (red, upper position) */
.slur-start::after {
    border-top: 0.1em solid #d73a49;
    width: var(--slur-width, 1.5em);
}
```

#### Dynamic Width Calculation
```javascript
// Set CSS custom properties for dynamic widths
setTimeout(() => {
    const markElement = mark.find()?.mark?.element;
    if (markElement) {
        if (style.classes.includes('beat-group-start')) {
            markElement.style.setProperty('--beat-group-width', '2.4em');
        }
        if (style.classes.includes('implicit-beat-start')) {
            markElement.style.setProperty('--implicit-beat-width', '1.8em');
        }
        if (style.classes.includes('slur-start')) {
            markElement.style.setProperty('--slur-width', '3.0em');
        }
    }
}, 0);
```

#### Benefits of Multiple Classes
1. **Layered Visual Effects**: A single character can show syntax highlighting + beat grouping + slur marking simultaneously
2. **No Class Explosion**: Instead of generating `beat-group-2-note-slur-start` permutation classes, use `beat-group-start slur-start`
3. **Dynamic CSS Properties**: Width calculations done in JavaScript, CSS stays clean
4. **Z-Index Layering**: Different visual elements stack properly using CSS positioning

#### Example: Fully Annotated Character
For the character "S" at position 0 in `"SRG"` with a slur over `"SR"` and beat grouping over `"SRG"`:

```html
<span class="cm-music-note in-beat-group beat-group-start in-slur slur-start"
      style="--beat-group-width: 1.8em; --slur-width: 1.2em;">S</span>
```

This renders:
- Green text color (note syntax highlighting)
- Orange background (beat group member)
- Blue arc below (beat group start)
- Red arc above (slur start)

This approach **eliminates the need for 60,000+ redundant CSS classes** and provides unlimited flexibility for visual annotation combinations.

## Race Condition Handling

### The Problem
```
Timeline:
T+0ms:   User types "S R"
T+300ms: Send "S R" to server
T+350ms: User types "G" (editor now has "S RG")
T+400ms: Server returns styling for "S R"
T+405ms: Apply styling → MISMATCH!
```

### Solution: Content Versioning

#### Request Versioning
```javascript
let requestId = 0;
let pendingRequests = new Map();

function sendParseRequest(content) {
    const id = ++requestId;
    const hash = hashContent(content);
    
    pendingRequests.set(id, { content, hash, timestamp: Date.now() });
    
    return fetch('/api/parse', {
        method: 'POST',
        body: JSON.stringify({
            input: content,
            request_id: id,
            content_hash: hash
        })
    });
}
```

#### Response Validation
```javascript
function handleParseResponse(response) {
    const { request_id, editor_tree } = response;
    
    // Check if request is still valid
    if (!pendingRequests.has(request_id)) {
        console.log('Ignoring stale response');
        return;
    }
    
    const request = pendingRequests.get(request_id);
    const currentContent = editor.getValue();
    const currentHash = hashContent(currentContent);
    
    // Only apply styling if content matches
    if (request.hash === currentHash && editor_tree.original_text === currentContent) {
        applyCharacterStyling(editor_tree);
    } else {
        console.log('Content mismatch, using fallback styling');
        applyFallbackStyling();
    }
    
    pendingRequests.delete(request_id);
}
```

#### Fallback Strategy
```javascript
function applyFallbackStyling() {
    // Simple regex-based immediate styling
    const content = editor.getValue();
    const lines = content.split('\n');
    
    lines.forEach((line, lineNum) => {
        if (/\|.*\|/.test(line)) {
            // Content line
            editor.addLineClass(lineNum, 'background', 'content-line');
        } else if (/^(Title|Composer|Key):/i.test(line)) {
            // Directive line  
            editor.addLineClass(lineNum, 'background', 'directive-line');
        }
    });
}
```

## Server Implementation

### Character Expansion Algorithm
```rust
#[derive(Serialize, Debug)]
struct CharElement {
    col: usize,
    char: char,
    char_type: String,
    octave: Option<i32>,
    note_ref: Option<usize>, // For lyrics alignment
}

#[derive(Serialize, Debug)]
struct EditorLine {
    line_num: usize,
    line_type: String, // "content", "lyrics", "directive", "text"
    chars: Vec<CharElement>,
}

#[derive(Serialize, Debug)]
struct EditorTree {
    content_hash: String,
    original_text: String,
    lines: Vec<EditorLine>,
}

fn expand_parse_tree_for_editor(document: &Document, original_text: &str) -> EditorTree {
    let mut lines = Vec::new();
    let content_hash = calculate_hash(original_text);
    
    // Process directives
    for directive in &document.directives {
        let line_num = directive.source.position.line;
        let chars = expand_directive_to_chars(directive);
        lines.push(EditorLine {
            line_num,
            line_type: "directive".to_string(),
            chars,
        });
    }
    
    // Process staves
    for stave in &document.staves {
        // Content line
        if let Some(content_line) = &stave.content_line {
            let line_num = stave.source.position.line;
            let chars = expand_content_line_to_chars(content_line);
            lines.push(EditorLine {
                line_num,
                line_type: "content".to_string(),
                chars,
            });
        }
        
        // Lyrics lines
        for lyrics_line in &stave.lyrics_lines {
            let line_num = lyrics_line.source.position.line;
            let chars = expand_lyrics_line_to_chars(lyrics_line);
            lines.push(EditorLine {
                line_num,
                line_type: "lyrics".to_string(),
                chars,
            });
        }
    }
    
    EditorTree {
        content_hash,
        original_text: original_text.to_string(),
        lines,
    }
}

fn expand_content_line_to_chars(content_line: &[ParsedElement]) -> Vec<CharElement> {
    let mut chars = Vec::new();
    
    for element in content_line {
        match element {
            ParsedElement::Note { value, position, octave, .. } => {
                let start_col = position.col;
                for (i, ch) in value.chars().enumerate() {
                    chars.push(CharElement {
                        col: start_col + i,
                        char: ch,
                        char_type: classify_note_char(ch, i == 0),
                        octave: *octave,
                        note_ref: None,
                    });
                }
            },
            ParsedElement::Barline { style, position, .. } => {
                let start_col = position.col;
                for (i, ch) in style.chars().enumerate() {
                    chars.push(CharElement {
                        col: start_col + i,
                        char: ch,
                        char_type: "barline".to_string(),
                        octave: None,
                        note_ref: None,
                    });
                }
            },
            ParsedElement::Whitespace { value, position } => {
                let start_col = position.col;
                for (i, ch) in value.chars().enumerate() {
                    chars.push(CharElement {
                        col: start_col + i,
                        char: ch,
                        char_type: "space".to_string(),
                        octave: None,
                        note_ref: None,
                    });
                }
            },
            // Handle other element types...
        }
    }
    
    chars
}

fn classify_note_char(ch: char, is_first: bool) -> String {
    match ch {
        'S' | 'R' | 'G' | 'M' | 'P' | 'D' | 'N' |
        's' | 'r' | 'g' | 'm' | 'p' | 'd' | 'n' => "sargam-note".to_string(),
        '1'..='7' => "number-note".to_string(),
        'C' | 'D' | 'E' | 'F' | 'G' | 'A' | 'B' => "western-note".to_string(),
        '#' => "sharp".to_string(),
        'b' | '♭' => "flat".to_string(),
        ':' => "octave-up".to_string(),
        '.' => "octave-down".to_string(),
        '-' => "dash".to_string(),
        '\'' => "breath".to_string(),
        _ => "unknown".to_string(),
    }
}
```

### API Integration
```rust
// Add to ParseResponse struct
#[derive(Serialize)]
struct ParseResponse {
    // ... existing fields
    editor_tree: Option<EditorTree>,
}

// Update parse_text function
async fn parse_text(Query(params): Query<HashMap<String, String>>) -> Response {
    // ... existing parsing logic
    
    let editor_tree = if parsed_doc.is_some() {
        Some(expand_parse_tree_for_editor(&document, &converted_input))
    } else {
        None
    };
    
    json_with_no_cache(ParseResponse {
        // ... existing fields
        editor_tree,
    })
}
```

## Client Integration

### Styling Application
```javascript
function applyCharacterStyling(editorTree) {
    // Clear existing styling
    clearAllStyling();
    
    editorTree.lines.forEach(line => {
        const lineElement = getEditorLine(line.line_num - 1);
        
        // Apply line-level background
        lineElement.classList.add(`${line.line_type}-line`);
        
        // Replace line content with styled spans
        const styledHTML = line.chars.map(char => {
            let classes = ['music-char', char.type];
            let attributes = '';
            
            if (char.octave !== null) {
                attributes += ` data-octave="${char.octave}"`;
            }
            if (char.note_ref !== null) {
                attributes += ` data-note-ref="${char.note_ref}"`;
            }
            
            return `<span class="${classes.join(' ')}"${attributes}>${escapeHtml(char.char)}</span>`;
        }).join('');
        
        // This is tricky with CodeMirror - might need custom widget
        replaceLineWithHTML(line.line_num - 1, styledHTML);
    });
}

function replaceLineWithHTML(lineNum, html) {
    // CodeMirror doesn't directly support HTML in content
    // Options:
    // 1. Use line widgets for overlay
    // 2. Create custom mode with character-level tokens
    // 3. Use separate styled preview pane
    // 4. Investigate CodeMirror 6 decorations
}
```

## Implementation Phases

### Phase 1: Basic Server Character Tree
- Add `editor_tree` field to ParseResponse
- Implement character expansion for content lines only
- Basic character type classification
- No octave indicators yet

### Phase 2: Client Integration  
- Create character styling CSS framework
- Implement basic styling application
- Add content versioning and race condition handling
- Fallback to simple line highlighting

### Phase 3: Full Feature Support
- Expand to lyrics lines with note alignment
- Add octave indicators (dots above/below)
- Handle all notation systems (Sargam, Western, Number)
- Directive line styling

### Phase 4: Performance Optimization
- Optimize character tree generation
- Minimize JSON payload size
- Implement efficient styling updates
- Consider WASM migration if needed

## Testing Strategy

### Unit Tests
- Character expansion accuracy
- Position calculation correctness
- Type classification for all character types
- Hash generation consistency

### Integration Tests  
- End-to-end parse → style → render flow
- Race condition handling
- Fallback behavior
- Performance with large documents

### Browser Tests
- Visual alignment verification
- Cross-browser compatibility
- Mobile responsiveness
- Keyboard navigation

## Performance Considerations

### Server-Side
- Character tree generation: O(n) where n = character count
- Expected overhead: ~2-5x original parse time
- Memory usage: ~3-5x original parse tree size
- JSON serialization: Minor overhead

### Client-Side  
- Styling application: O(n) character updates
- DOM manipulation: Consider virtual scrolling for large docs
- Network transfer: Character JSON ~3-10x larger than plain text

### Optimization Strategies
- Incremental updates (only changed lines)
- Character tree caching
- Compressed JSON transfer
- Lazy styling for off-screen content

## Implementation Details: CodeMirror Token Position Mapping

### How CodeMirror Knows Character Positions

The position mapping between server tokens and CodeMirror highlighting works through a precise coordination based on **content elements only**:

#### 1. Server Token Generation Strategy
```rust
pub fn generate_syntax_tokens(document: &Document, original_input: &str) -> Vec<SyntaxToken> {
    let mut tokens = Vec::new();

    // ONLY process content elements with position: {row, col}
    // Skip upper/lower line elements - they don't have row/col positions
    for element in document.elements {
        match element {
            DocumentElement::Stave(stave) => {
                for line in &stave.lines {
                    match line {
                        StaveLine::Content(parsed_elements) => {
                            // Process content line elements
                            for element in parsed_elements {
                                process_parsed_element(element, &mut tokens, original_input);
                            }
                        }
                        StaveLine::Upper(_) => {
                            // Skip - no row/col positions
                        }
                        StaveLine::Lower(_) => {
                            // Skip - no row/col positions
                        }
                        // ... other line types
                    }
                }
            }
        }
    }

    tokens
}

fn process_parsed_element(element: &ParsedElement, tokens: &mut Vec<SyntaxToken>, original_input: &str) {
    match element {
        ParsedElement::Note { value, position: pos, .. } => {
            // Convert (row, col) to absolute position in document
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                tokens.push(SyntaxToken {
                    token_type: "note".to_string(),
                    start: start_pos,
                    end: start_pos + value.len(),
                    content: value.clone(),
                });
            }
        }
        ParsedElement::Barline { style, position: pos, .. } => {
            if let Some(start_pos) = position_to_absolute_offset(pos, original_input) {
                tokens.push(SyntaxToken {
                    token_type: "barline".to_string(),
                    start: start_pos,
                    end: start_pos + style.len(),
                    content: style.clone(),
                });
            }
        }
        // ... other content elements with position: {row, col}
    }
}

fn position_to_absolute_offset(position: &Position, original_input: &str) -> Option<usize> {
    let lines: Vec<&str> = original_input.split('\n').collect();

    if position.row == 0 || position.row > lines.len() {
        return None;
    }

    let mut offset = 0;
    // Add lengths of all previous lines (including newlines)
    for i in 0..(position.row - 1) {
        offset += lines[i].len() + 1; // +1 for newline
    }

    // Add column offset within the current line
    if position.col > 0 && position.col <= lines[position.row - 1].len() + 1 {
        offset += position.col - 1; // Convert 1-based to 0-based
    }

    Some(offset)
}
```

#### 2. Token Array Sent to Frontend
```json
{
  "syntax_tokens": [
    {"token_type": "note", "start": 0, "end": 1, "content": "S"},
    {"token_type": "whitespace", "start": 1, "end": 2, "content": " "},
    {"token_type": "note", "start": 2, "end": 3, "content": "R"},
    {"token_type": "barline", "start": 8, "end": 9, "content": "|"}
  ]
}
```

#### 3. CodeMirror Mode Creation
```javascript
createTokenBasedMode(tokens) {
    return {
        token: function(stream, state) {
            // stream.pos is current character position in line
            const pos = stream.pos;
            
            // Find token covering this position
            const token = tokens.find(t => pos >= t.start && pos < t.end);
            
            if (token) {
                // Consume characters for this token
                const remaining = token.end - pos;
                for (let i = 0; i < remaining; i++) {
                    stream.next(); // Advances stream.pos
                }
                return `music-${token.token_type}`;
            }
            
            stream.next();
            return null;
        }
    };
}
```

#### Key Concepts:
- **Content elements only**: Only elements with `position: {row, col}` generate tokens
- **Absolute positioning**: Convert (row, col) to absolute character offsets
- **Spatial semantics**: Beat groups come from `in_beat_group` flags, not separate tokens
- **`stream.pos`**: CodeMirror's current character position (0-based)
- **Token lookup**: For each position, find which token covers it
- **CSS class generation**: `music-note` → CodeMirror adds `cm-` prefix → `cm-music-note`

#### Example Walkthrough for `"SSSS-\n____"`:

**Input Analysis:**
```
SSSS-     <- Content line (generates tokens)
____      <- Lower line (skipped - no row/col positions)
```

**Generated Tokens:**
1. **Position 0**: Note `S` at row=1, col=1 → absolute pos 0
   - Token: `{start: 0, end: 1, type: "note", content: "S"}`
2. **Position 1**: Note `S` at row=1, col=2 → absolute pos 1
   - Token: `{start: 1, end: 2, type: "note", content: "S"}`
3. **Position 4**: Dash `-` at row=1, col=5 → absolute pos 4
   - Token: `{start: 4, end: 5, type: "dash", content: "-"}`

**Beat Group Information:**
- Notes with `in_beat_group: true` get beat group styling via other mechanisms
- No separate `beat_group` tokens are generated for `____` in lower line

The positions work because we **convert (row, col) to absolute offsets** that match CodeMirror's linear character indexing, but only for content elements that have semantic meaning for syntax highlighting.

## Visual Editing Features Using CSS Classes

### Core Concept

Modern music notation editors require interactive editing features beyond syntax highlighting. The music-text code editor implements these features using **CSS-based visual annotations** that preserve the plain-text data model while providing rich interactive capabilities.

#### Key Principles
1. **Data Model Purity**: Visual annotations never modify the underlying plain-text content
2. **CSS-Only Rendering**: All visual effects achieved through CSS classes and pseudo-elements
3. **Toggle-Based Interaction**: Features can be added/removed without changing source text
4. **Layered Visualization**: Multiple annotation types can coexist (slurs + beat groups + character styling)

### Implementation Strategy

#### CodeMirror Text Markers
```javascript
// Add slur to selected range
const slurMark = editor.markText(
    {line: 0, ch: 2},
    {line: 0, ch: 8},
    {
        className: "slur-span",
        inclusiveLeft: true,
        inclusiveRight: true
    }
);

// Add CSS classes to first and last characters for arc endpoints
const startMark = editor.markText(
    {line: 0, ch: 2},
    {line: 0, ch: 3},
    {className: "slur-start"}
);

const endMark = editor.markText(
    {line: 0, ch: 7},
    {line: 0, ch: 8},
    {className: "slur-end"}
);
```

#### CSS Arc Generation
```css
/* Slur start - left bracket with upper arc */
.slur-start {
    position: relative;
}

.slur-start::before {
    content: '';
    position: absolute;
    top: -0.3em;
    left: 0;
    border-top: 0.1em solid #d73a49;
    border-left: 0.1em solid #d73a49;
    border-top-left-radius: 0.5em;
    width: 0.3em;
    height: 0.3em;
    z-index: 1;
}

/* Slur end - right bracket with upper arc */
.slur-end {
    position: relative;
}

.slur-end::after {
    content: '';
    position: absolute;
    top: -0.3em;
    right: 0;
    border-top: 0.1em solid #d73a49;
    border-right: 0.1em solid #d73a49;
    border-top-right-radius: 0.5em;
    width: 0.3em;
    height: 0.3em;
    z-index: 1;
}

/* Optional: Spanning arc for continuous slur line */
.slur-span {
    border-top: 0.05em solid rgba(215, 58, 73, 0.3);
    border-radius: 20px 20px 0 0;
    margin-top: -0.2em;
    padding-top: 0.2em;
}
```

#### Interactive Button Implementation
```javascript
// Global function for slur toggle button
function toggleSlur() {
    const editor = app.codeMirrorManager.getEditor();
    const selection = editor.getSelection();

    if (!selection) {
        alert('Please select text to add a slur');
        return;
    }

    const from = editor.getCursor('from');
    const to = editor.getCursor('to');

    // Check if selection already has slur marks
    const existingMarks = editor.findMarksAt(from).concat(editor.findMarksAt(to));
    const hasSlur = existingMarks.some(mark =>
        mark.className && mark.className.includes('slur')
    );

    if (hasSlur) {
        // Remove existing slur marks
        existingMarks.forEach(mark => {
            if (mark.className && mark.className.includes('slur')) {
                mark.clear();
            }
        });
    } else {
        // Add new slur marks
        const spanMark = editor.markText(from, to, {
            className: "slur-span",
            title: "Slur"
        });

        const startMark = editor.markText(from, {line: from.line, ch: from.ch + 1}, {
            className: "slur-start",
            title: "Slur start"
        });

        const endMark = editor.markText({line: to.line, ch: to.ch - 1}, to, {
            className: "slur-end",
            title: "Slur end"
        });
    }
}
```

### Parsing Challenges Introduced

#### 1. Data Model Purity vs. Visual State

**Challenge**: Visual annotations exist only in the editor's DOM but not in the plain-text data model.

**Problem Scenarios**:
```javascript
// User adds slur via button
toggleSlur(); // Adds CSS marks to editor

// User types new content
editor.replaceSelection("R G M"); // Marks may become misaligned

// Server re-parses and applies new styling
applyCharacterStyles(serverStyles); // May clear or conflict with slur marks
```

**Solutions**:
- **Mark Persistence**: Store mark positions in editor state
- **Reapplication Logic**: Automatically restore marks after server updates
- **Position Tracking**: Update mark positions when content changes

#### 2. Roundtrip Validation Conflicts

**Challenge**: Parser's roundtrip validation assumes perfect plain-text reconstruction, but visual marks don't affect the source text.

**Potential Issues**:
```rust
// Server reconstructs from parse tree
let reconstructed = reconstruct_unprocessed(document);
// reconstructed == "S R G M P"

// Client editor contains
let editor_content = editor.getValue();
// editor_content == "S R G M P" (same text)
// BUT editor has visual slur marks that parser doesn't know about
```

**Solutions**:
- **Orthogonal Systems**: Keep visual annotations completely separate from parsing
- **Metadata Storage**: Store visual annotations in localStorage or separate data structure
- **Server-Side Awareness**: Extend parser to optionally handle visual annotation metadata

#### 3. Synchronization Issues

**Challenge**: Server-generated character styling may conflict with client-side visual marks.

**Race Condition Example**:
```javascript
// T+0: User adds slur marks
toggleSlur(); // Marks positions 2-8

// T+100: User types new content
editor.replaceSelection("S R | G M |"); // Content changes

// T+200: Server responds with new character styling
applyCharacterStyles(characterStyles); // May clear slur marks

// T+300: Slur marks are lost or mispositioned
```

**Solutions**:
- **Mark Preservation**: Save marks before applying server styles, restore after
- **Layered Styling**: Apply server styles to text, visual marks to overlay elements
- **Smart Reapplication**: Use content diffing to update mark positions intelligently

#### 4. Performance Implications

**Challenge**: Large numbers of text markers can impact editor performance.

**Performance Concerns**:
- **DOM Overhead**: Each mark creates additional DOM elements
- **Re-rendering Cost**: Frequent mark updates trigger layout recalculation
- **Memory Usage**: Marks store references that prevent garbage collection

**Solutions**:
```javascript
// Efficient mark management
class MarkManager {
    constructor(editor) {
        this.editor = editor;
        this.marksByType = new Map(); // Group marks by feature type
        this.markPool = []; // Reuse cleared marks
    }

    addSlur(from, to) {
        // Reuse existing marks when possible
        const recycledMark = this.markPool.pop();
        if (recycledMark) {
            // Reposition existing mark
            recycledMark.doc.markText(from, to, recycledMark.options);
        } else {
            // Create new mark
            const mark = this.editor.markText(from, to, {className: "slur-span"});
            this.marksByType.set('slur', mark);
        }
    }

    clearMarks(type) {
        const marks = this.marksByType.get(type) || [];
        marks.forEach(mark => {
            mark.clear();
            this.markPool.push(mark); // Recycle for reuse
        });
        this.marksByType.delete(type);
    }
}
```

#### 5. Conflict Resolution

**Challenge**: Multiple annotation types may overlap or conflict visually.

**Overlap Scenarios**:
- **Slur + Beat Group**: Both create visual arcs (upper vs lower)
- **Slur + Character Styling**: Color/background conflicts
- **Multiple Slurs**: Nested or overlapping slur ranges

**Resolution Strategies**:
```css
/* Z-index layering for visual priority */
.slur-start::before, .slur-end::after {
    z-index: 3; /* Above beat groups */
}

.beat-group-start::after {
    z-index: 2; /* Above character styling */
}

.cm-music-note {
    z-index: 1; /* Base layer */
}

/* Color coordination to avoid conflicts */
.slur-span.in-beat-group {
    border-top-color: #8B4A9C; /* Blend slur + beat group colors */
}

/* Alternative positioning for nested slurs */
.slur-start.nested::before {
    top: -0.6em; /* Higher position for inner slur */
}
```

### Integration with Existing Systems

#### Character Styling Coordination
```javascript
// Apply server styles while preserving visual marks
applyCharacterStylesWithMarks(characterStyles) {
    // 1. Save current visual marks
    const savedMarks = this.saveAllMarks();

    // 2. Apply server character styling
    this.applyCharacterStyles(characterStyles);

    // 3. Restore visual marks with updated positions
    this.restoreMarks(savedMarks);
}

saveAllMarks() {
    const marks = this.editor.getAllMarks();
    return marks.map(mark => ({
        range: mark.find(),
        className: mark.className,
        options: mark.options
    })).filter(mark => mark.range); // Only valid marks
}

restoreMarks(savedMarks) {
    savedMarks.forEach(markData => {
        if (this.isValidRange(markData.range)) {
            this.editor.markText(
                markData.range.from,
                markData.range.to,
                {className: markData.className, ...markData.options}
            );
        }
    });
}
```

#### Server Communication Enhancement
```javascript
// Optional: Send visual annotation metadata to server
async parseWithAnnotations(input) {
    const annotations = this.extractVisualAnnotations();

    const result = await API.parse(input, {
        annotations: annotations,
        preserve_visual_state: true
    });

    // Server can optionally preserve annotation context
    if (result.preserved_annotations) {
        this.restoreAnnotations(result.preserved_annotations);
    }

    return result;
}
```

### Future Extensions

#### Planned Visual Features
- **Dynamics**: Crescendo/diminuendo markings with CSS gradients
- **Articulations**: Staccato dots, accents using pseudo-elements
- **Ornaments**: Trills, mordents with symbol fonts
- **Chord Symbols**: Above-staff text positioning
- **Fingering**: Below-note number annotations

#### Advanced Interaction Patterns
- **Drag-and-Drop**: Moving slurs by dragging endpoints
- **Context Menus**: Right-click to modify annotation properties
- **Keyboard Shortcuts**: Quick annotation toggle (Ctrl+S for slur)
- **Multi-Selection**: Apply annotations to multiple discontinuous ranges

## Implementation TODOs

### High Priority
- [ ] **Implicit Beat Grouping in Code Editor**: Extend character styling to automatically group consecutive musical elements with same beat timing
  - Use rhythm analyzer output to detect same-duration sequences
  - Apply lower arc styling for implicit groups (2+ consecutive notes/rests/dashes with identical duration)
  - Distinguish from explicit beat groups marked with `___`
  - Ensure proper visual hierarchy: explicit groups (blue) vs implicit groups (lighter blue)

### Current Implementation Status

**🎉 PROOF OF CONCEPT ACHIEVED - WYSIWYM Music Editor Working**

**Core WYSIWYM Architecture:**
- [x] **LSP-style semantic feedback loop** - Parser ↔ editor round-trip working
- [x] **Unified semantic pipeline** - `generate_syntax_spans()` and `DocumentNode` system
- [x] **Multi-notation system support** - Sargam, Western, Number, Bhatkhande, Tabla
- [x] **Unicode accidental support** - ♯, ♭, ♯♯, ♭♭ alongside ASCII variants
- [x] **CodeMirror integration** - Textarea compatibility layer with span overlays
- [x] **Real-time syntax highlighting** - Live semantic analysis with proper colors
- [x] **Beat group detection** - Both explicit `___` markers and implicit spatial analysis
- [x] **CSS custom properties** - Dynamic styling with `--beat-loop-4`, `--show-divisions`

**Visual Rendering System:**
- [x] **Basic syntax highlighting** with token-based approach
- [x] **Explicit beat group detection** via spatial assignment
- [x] **Beat group arc visualization** with brute force CSS (63 classes)
- [x] **Slur visualization** with upper arcs
- [x] **Octave marker positioning** above/below notes
- [x] **Monospaced font optimization** for perfect arc alignment

**Next Phase - Production Ready:**
- [ ] **Implicit beat grouping** for consecutive same-duration elements
- [ ] **Character-level fixed-width styling** for perfect columnar alignment
- [ ] **Race condition handling** for async styling updates
- [ ] **Fallback styling** when server parsing fails

### Medium Priority
- [ ] **Enhanced token position mapping** for complex spatial relationships
- [ ] **Performance optimization** for large notation documents
- [ ] **Mobile responsiveness** with touch-friendly controls
- [ ] **Accessibility support** for screen readers

### Low Priority
- [ ] **Custom font loading** for music-specific typefaces
- [ ] **GPU acceleration** for smooth arc rendering
- [ ] **Incremental updates** for efficient re-styling

## Conclusion

**WYSIWYM Music Editor Proof of Concept Successfully Achieved!**

This specification documented the implementation of a keystroke-driven, text-first music editor that round-trips to a parser and projects semantic meaning back as live visual overlays. The core architectural vision has been validated:

**Proven Concepts:**
1. **LSP-style feedback loop** - Parser ↔ editor communication working
2. **Semantic overlay rendering** - CSS custom properties for beat loops and styling
3. **Multi-notation system support** - Unified pipeline for Sargam, Western, Number systems
4. **Text-first paradigm** - Musicians can type plain text and see meaning projected visually

**Production-Ready Features:**
- Real-time syntax highlighting with semantic analysis
- Beat group detection and arc visualization
- CodeMirror integration with textarea compatibility
- Unicode accidental support across notation systems

**Next Development Phase:**
The proof of concept validates the architectural approach. Future work focuses on:
1. **Character-level fixed-width styling** for perfect columnar alignment
2. **Race condition handling** for robust async updates
3. **Implicit beat grouping** for enhanced visual organization
4. **Performance optimization** for production deployment

The system successfully maintains the plain-text data model essential for parsing while providing the rich visual experience users expect from modern editors. Most importantly, it honors how musicians naturally think in text and numbers (solfege, sargam, CDE notation) while providing semantic meaning projected as visual overlays.