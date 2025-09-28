# Slurs in CSS Proof of Concept Specification

## Overview

This specification defines a proof of concept for implementing musical slurs using CSS classes in CodeMirror for visual display only, with a simple source tab to show the plain text that the API processes.

## Architecture Principles

### Separation of Concerns
- **Frontend**: Visual slur display via CSS classes only
- **Backend**: Unchanged - processes plain text normally
- **Text**: Clean music-text source remains unchanged
- **Annotations**: Pure GUI visual effects, not persisted

### Data Flow
```
User Input → Plain Text → API → Backend Processing → Output (with plain_text field)
           ↓
         CSS Classes → Visual Slur Display
```

## Frontend Implementation

### Slur Toggle Function
```javascript
// Simple slur toggle function
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
        mark.className && (mark.className.includes('slur-start') || mark.className.includes('slur-end'))
    );

    if (hasSlur) {
        // Remove existing slur marks
        existingMarks.forEach(mark => {
            if (mark.className && (mark.className.includes('slur-start') || mark.className.includes('slur-end'))) {
                mark.clear();
            }
        });
    } else {
        // Add new slur marks
        editor.markText(from, {line: from.line, ch: from.ch + 1}, {
            className: 'slur-start',
            title: 'Slur start'
        });

        editor.markText({line: to.line, ch: to.ch - 1}, to, {
            className: 'slur-end',
            title: 'Slur end'
        });
    }
}
```

### CSS Visual Rendering
```css
/* Visual slur arcs */
.slur-start {
    position: relative;
}

.slur-start::before {
    content: '';
    position: absolute;
    top: -0.4em;
    left: 0;
    border-top: 2px solid #d73a49;
    border-left: 2px solid #d73a49;
    border-top-left-radius: 0.6em;
    width: 0.4em;
    height: 0.4em;
    z-index: 3;
    pointer-events: none;
}

.slur-end {
    position: relative;
}

.slur-end::after {
    content: '';
    position: absolute;
    top: -0.4em;
    right: 0;
    border-top: 2px solid #d73a49;
    border-right: 2px solid #d73a49;
    border-top-right-radius: 0.6em;
    width: 0.4em;
    height: 0.4em;
    z-index: 3;
    pointer-events: none;
}

/* Highlight slurred notes */
.slur-start,
.slur-end {
    background-color: rgba(215, 58, 73, 0.1);
    border-radius: 2px;
}
```

## Source Tab

### HTML Structure
```html
<!-- Add to tab bar -->
<div class="tab" onclick="switchTab('source', this)">Source</div>

<!-- Add tab content -->
<div id="source-tab" class="tab-content">
    <div id="source-output" class="pipeline-section">Plain text will appear here after parsing</div>
</div>
```

### Source Tab Display
```javascript
// Simple function to update source tab with API response
function updateSourceOutput(result) {
    const sourceOutput = document.getElementById('source-output');
    if (result.plain_text) {
        sourceOutput.innerHTML = `<pre>${escapeHtml(result.plain_text)}</pre>`;
    } else {
        sourceOutput.innerHTML = '<p>No plain text in response</p>';
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
```

## Backend Implementation (Minimal Change)

### API Response Change
```rust
// Add to ParseResponse in src/web.rs
#[derive(Debug, Serialize)]
pub struct ParseResponse {
    success: bool,
    plain_text: Option<String>,  // NEW: Echo back the original input
    parsed_document: Option<crate::parse::Document>,
    rhythm_analyzed_document: Option<crate::parse::Document>,
    // ... existing fields
}

// In parse_text function, add:
async fn parse_text(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let input = params.get("input").cloned().unwrap_or_default();

    // Process normally...
    match crate::process_notation(&input) {
        Ok(result) => {
            Json(ParseResponse {
                success: true,
                plain_text: Some(input.clone()),  // NEW: Include original input
                parsed_document: Some(result.document),
                // ... other fields
            })
        }
        Err(e) => {
            Json(ParseResponse {
                success: false,
                plain_text: Some(input.clone()),  // NEW: Include even on error
                error: Some(e.to_string()),
                ..Default::default()
            })
        }
    }
}
```

## Example Workflow

### Complete User Experience
```
1. User Input:    |S R G M P|
2. User Action:   Select "R G M" and click slur button
3. GUI Effect:    Visual slur arc appears via CSS classes
4. API Call:      Normal API call with plain text input
5. API Response:  Includes plain_text field with original input
6. Source Tab:    Displays the plain_text from API response
```

### API Integration
```javascript
// Updated parse functions call updateSourceOutput
async function parseAndUpdatePreview() {
    // ... existing code ...
    const result = await API.parseForPreview(input);

    // Update all outputs including new source tab
    UI.updatePipelineData(result);
    UI.updateLilyPondOutput(result);
    updateSourceOutput(result);  // NEW: Update source tab
    // ... rest of existing code ...
}
```

## Benefits

### Simple Implementation
- Visual slurs via CSS only - no complex state management
- Source tab shows what backend actually received
- Minimal backend changes (just echo input back)
- No grammar or parser modifications needed

### User Experience
- Rich visual feedback with CSS arcs
- Easy toggle on/off with slur button
- Source transparency - see exactly what was processed
- Clean separation between visual effects and actual content

### Developer Experience
- Easy to implement and debug
- No complex annotation persistence
- Backend stays focused on core music processing
- Foundation for future annotation features

## Future Extensions

### Additional Visual Annotations
- Dynamics markings (forte, piano) as CSS overlays
- Fingering numbers positioned above/below notes
- Ornament symbols (trills, mordents) as visual decorations
- Tempo markings and expression text

### Enhanced Backend Integration
- Convert CSS annotations to spatial format for output generation
- Persist annotation preferences in user settings
- Export options with/without visual annotations

### Advanced GUI Features
- Multiple slur layers for complex passages
- Keyboard shortcuts for quick annotation
- Undo/redo for annotation changes
- Visual feedback for overlapping annotations

## Implementation Priority

1. **Phase 1**: Slur button, CSS arcs, and source tab
2. **Phase 2**: Polish slur interaction and visual styling
3. **Phase 3**: Additional annotation types (dynamics, fingerings)
4. **Phase 4**: Backend integration for output generation
5. **Phase 5**: Advanced features and keyboard shortcuts

This specification provides a simple foundation for visual annotations while keeping the core music-text processing unchanged.