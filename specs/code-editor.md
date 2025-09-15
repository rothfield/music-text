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

## Conclusion

This specification provides a comprehensive approach to implementing music-text as a code editor with perfect columnar alignment and rich syntax highlighting. The key innovations are:

1. **Server-generated character trees** for semantic accuracy
2. **Fixed-width character spans** for columnar alignment
3. **Race condition handling** for async styling updates
4. **Fallback strategies** for robustness
5. **Precise token position mapping** for syntax highlighting

The system maintains the plain-text data model essential for parsing while providing the rich visual experience users expect from modern editors.