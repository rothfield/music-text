# Visual-to-Spatial Conversion Algorithm

## Overview

This document describes the algorithm for converting WYSIWYG visual slur representations into the 2-line spatial format expected by the existing music-text lexer.

## Problem Statement

**Input:** ContentEditable DOM with visual slur spans
```html
S R G <span class="slur">M P</span> D N
```

**Required Output:** Spatial music-text format
```
     _ _
S R G M P D N
```

## Algorithm Architecture

### Phase 1: Line Extraction and Classification

```javascript
function generateSpatialFormat(editorDOM) {
    // Extract logical lines from contenteditable
    const lines = extractLines(editorDOM);
    
    // Process each line independently
    const outputLines = [];
    for (let line of lines) {
        if (containsSlurs(line)) {
            // Musical line with slurs requires processing
            outputLines.push(...processMusicalLine(line));
        } else {
            // Non-musical or slur-free line passes through
            outputLines.push(extractPlainText(line));
        }
    }
    
    return outputLines.join('\n');
}
```

### Phase 2: Musical Line Processing

```javascript
function processMusicalLine(line) {
    const slurLine = generateSlurLine(line);
    const noteLine = extractNoteLine(line);
    
    return [slurLine, noteLine];  // Insert slur line above note line
}
```

### Phase 3: Slur Line Generation

```javascript
function generateSlurLine(line) {
    let slurLine = '';
    
    walkLineCharacters(line, (char, context) => {
        if (context.isInSlur && isMusicalNote(char)) {
            slurLine += '_';    // Underscore above slurred notes
        } else if (context.isInSlur) {
            slurLine += char;   // Preserve spaces/dashes within slurs
        } else {
            slurLine += ' ';    // Space above non-slurred content
        }
    });
    
    return slurLine.trimRight();
}
```

## Detailed Algorithm Steps

### Step 1: Line Boundary Detection

ContentEditable may contain:
- Natural line breaks (`<br>`, `\n`)
- Block elements (`<div>`, `<p>`)
- Mixed content

**Strategy:** Normalize to logical music-text lines
```javascript
function extractLines(editor) {
    // Handle various contenteditable line representations
    return editor.innerHTML
        .split(/<br>|<\/div>|<\/p>|\n/)
        .map(cleanHTML)
        .filter(line => line.trim());
}
```

### Step 2: Slur Detection

```javascript
function containsSlurs(line) {
    // Check for slur span elements in line DOM
    const tempDiv = document.createElement('div');
    tempDiv.innerHTML = line;
    return tempDiv.querySelector('.slur') !== null;
}
```

### Step 3: Character-Level Processing

```javascript
function walkLineCharacters(line, callback) {
    const lineDOM = parseLineToDOM(line);
    let position = 0;
    let inSlurContext = false;
    
    function processNode(node) {
        if (node.nodeType === Node.TEXT_NODE) {
            for (let char of node.textContent) {
                callback(char, {
                    position: position++,
                    isInSlur: inSlurContext
                });
            }
        } else if (node.classList.contains('slur')) {
            inSlurContext = true;
            processChildNodes(node);
            inSlurContext = false;
        } else {
            processChildNodes(node);
        }
    }
    
    processNode(lineDOM);
}
```

## Example Walkthrough

### Input Processing
```html
Line: "S R G <span class='slur'>M P</span> D N"
```

### Character-by-Character Analysis
| Position | Char | In Slur | Action | Slur Line Output |
|----------|------|---------|--------|------------------|
| 0 | S | false | space | ' ' |
| 1 | ' ' | false | space | ' ' |
| 2 | R | false | space | ' ' |
| 3 | ' ' | false | space | ' ' |
| 4 | G | false | space | ' ' |
| 5 | ' ' | false | space | ' ' |
| 6 | M | **true** | underscore | '_' |
| 7 | ' ' | **true** | preserve | ' ' |
| 8 | P | **true** | underscore | '_' |
| 9 | ' ' | false | space | ' ' |
| 10 | D | false | space | ' ' |
| 11 | ' ' | false | space | ' ' |
| 12 | N | false | space | ' ' |

### Final Output
```
Slur Line: "      _ _    "  (trimmed: "      _ _")
Note Line: "S R G M P D N"
```

## Multi-Line Example

### Input
```html
<div>S R G M</div>
<div>P D <span class="slur">N S</span></div>  
<div>R G M P</div>
```

### Processing
1. **Line 1:** "S R G M" → No slurs → Pass through: `"S R G M"`
2. **Line 2:** "P D N S" with slur → Generate slur line + note line:
   ```
   "     _ _"    (slur line)
   "P D N S"    (note line)
   ```
3. **Line 3:** "R G M P" → No slurs → Pass through: `"R G M P"`

### Final Output
```
S R G M
     _ _
P D N S  
R G M P
```

## Edge Cases Handled

### Multiple Slurs Per Line
```html
<span class="slur">S R</span> G <span class="slur">M P</span>
```
Output:
```
_ _   _ _
S R G M P
```

### Slurs with Rhythm Notation
```html
<span class="slur">S- R</span> G--
```
Output:
```
__ _    
S- R G--
```

### Nested Elements
```html
<span class="slur">S <em>R</em> G</span>
```
Output:
```
_ _ _
S R G
```

## Performance Considerations

### Complexity
- **Time:** O(n) where n = total characters across all lines
- **Space:** O(n) for output generation

### Optimization Strategies
- **Line-level caching:** Skip processing for unchanged lines
- **Incremental updates:** Only reprocess modified lines
- **DOM reuse:** Minimize DOM parsing overhead

## Algorithm Validation

### Test Cases
1. **Empty input** → Empty output
2. **No slurs** → Input passed through unchanged  
3. **Single slur** → Proper underscore positioning
4. **Multiple slurs** → Correct spatial alignment
5. **Complex nesting** → Flattened correctly

### Format Verification
- Character-level alignment between slur and note lines
- Proper handling of spaces and musical elements
- Compatibility with existing lexer expectations

## Integration Points

### Input Interface
```javascript
const visualEditor = document.getElementById('wysiwygEditor');
const spatialFormat = generateSpatialFormat(visualEditor);
```

### Output Interface  
```javascript
const parseResult = unified_parser(spatialFormat);
// Feeds into existing parsing pipeline unchanged
```

## Conclusion

This algorithm provides robust conversion from visual slur editing to the spatial format required by the music-text lexer. The line-by-line processing approach ensures clean output structure while maintaining perfect character alignment for spatial notation requirements.

The key insight is treating slur lines as **insertions above musical content** rather than inline transformations, preserving the natural structure of multi-line music-text documents.

---

*Technical Note: This algorithm is designed to handle the full complexity of contenteditable DOM structures while producing minimal, clean spatial output that integrates seamlessly with existing parsing infrastructure.*