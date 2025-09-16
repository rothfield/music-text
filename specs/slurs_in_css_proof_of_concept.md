# Slurs in CSS Proof of Concept Specification

## Overview

This specification defines a proof of concept for implementing musical slurs using CSS classes in CodeMirror, with complete state management and backend reverse engineering to convert GUI annotations into spatial music notation format.

## Architecture Principles

### Separation of Concerns
- **Frontend**: State management for both source text and GUI annotations
- **Backend**: Musical intelligence and spatial conversion
- **Text**: Clean music-text source remains portable
- **Annotations**: GUI state persisted and convertible

### Data Flow
```
User Input → Source Text + CSS Classes → State Management → API (Source + Annotations) → Backend Processing → Output
```

## State Management System

### Core Classes

#### MusicTextEditor
```javascript
class MusicTextEditor {
    constructor() {
        this.sourceText = "";           // Pure music-text source
        this.guiAnnotations = [];       // CSS class positions and metadata
        this.editor = null;             // CodeMirror instance
    }

    // Initialize editor and restore state
    init(containerId) {
        this.editor = CodeMirror(document.getElementById(containerId), {
            mode: 'music-text',
            lineWrapping: true,
            theme: 'default'
        });

        this.setupEventListeners();
        this.loadFromLocalStorage();
    }

    // Auto-save on changes
    setupEventListeners() {
        this.editor.on('change', () => {
            this.sourceText = this.editor.getValue();
            this.saveToLocalStorage();
            this.updateSourceTab();
        });

        // Save annotations when marks change
        this.editor.on('cursorActivity', () => {
            this.guiAnnotations = MusicTextConverter.extractAnnotationsFromDisplay(this.editor);
            this.saveToLocalStorage();
            this.updateSourceTab();
        });
    }

    // Persistence
    saveToLocalStorage() {
        const session = {
            sourceText: this.sourceText,
            guiAnnotations: this.guiAnnotations,
            timestamp: Date.now(),
            version: "1.0"
        };
        localStorage.setItem('musicTextSession', JSON.stringify(session));
    }

    loadFromLocalStorage() {
        const saved = localStorage.getItem('musicTextSession');
        if (saved) {
            try {
                const session = JSON.parse(saved);
                this.sourceText = session.sourceText || "";
                this.guiAnnotations = session.guiAnnotations || [];
                this.restoreGUIState();
            } catch (e) {
                console.warn('Failed to load session from localStorage:', e);
            }
        }
    }

    restoreGUIState() {
        if (this.sourceText) {
            this.editor.setValue(this.sourceText);
        }

        // Restore CSS class annotations
        MusicTextConverter.sourceToAnnotatedDisplay(this.editor, this.sourceText, this.guiAnnotations);
    }

    // API integration
    getAPIPayload() {
        return {
            input: this.sourceText,
            gui_annotations: this.guiAnnotations.length > 0 ? {
                slurs: this.guiAnnotations.filter(a => a.type === 'slur')
            } : null
        };
    }
}
```

#### MusicTextConverter
```javascript
class MusicTextConverter {
    // Convert source + annotations → CodeMirror display with CSS classes
    static sourceToAnnotatedDisplay(editor, sourceText, annotations) {
        // 1. Set source text
        editor.setValue(sourceText);

        // 2. Apply annotations as CSS classes
        annotations.forEach(annotation => {
            if (annotation.type === 'slur') {
                // Add slur start mark
                editor.markText(
                    {line: annotation.fromLine, ch: annotation.fromCol},
                    {line: annotation.fromLine, ch: annotation.fromCol + 1},
                    {
                        className: 'slur-start',
                        attributes: {'data-slur-id': annotation.id}
                    }
                );

                // Add slur end mark
                editor.markText(
                    {line: annotation.toLine, ch: annotation.toCol - 1},
                    {line: annotation.toLine, ch: annotation.toCol},
                    {
                        className: 'slur-end',
                        attributes: {'data-slur-id': annotation.id}
                    }
                );
            }
        });
    }

    // Extract annotations from current CodeMirror display
    static extractAnnotationsFromDisplay(editor) {
        const marks = editor.getAllMarks();
        const annotations = [];
        const slurPairs = new Map();

        // Group slur start/end marks by ID
        marks.forEach(mark => {
            if (!mark.className) return;

            const slurId = mark.attributes && mark.attributes['data-slur-id'];
            if (!slurId) return;

            const range = mark.find();
            if (!range) return;

            if (mark.className.includes('slur-start')) {
                if (!slurPairs.has(slurId)) {
                    slurPairs.set(slurId, {});
                }
                slurPairs.get(slurId).start = range.from;
            } else if (mark.className.includes('slur-end')) {
                if (!slurPairs.has(slurId)) {
                    slurPairs.set(slurId, {});
                }
                slurPairs.get(slurId).end = range.to;
            }
        });

        // Convert pairs to annotations
        slurPairs.forEach((pair, id) => {
            if (pair.start && pair.end) {
                annotations.push({
                    id: id,
                    type: 'slur',
                    fromLine: pair.start.line,
                    fromCol: pair.start.ch,
                    toLine: pair.end.line,
                    toCol: pair.end.ch
                });
            }
        });

        return annotations;
    }

    // Generate unique ID for new annotations
    static generateAnnotationId() {
        return 'slur_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
    }
}
```

## Frontend Implementation

### Slur Toggle Function
```javascript
// Global function for slur button
function toggleSlur() {
    const editor = musicTextEditor.editor;
    const selection = editor.getSelection();

    if (!selection) {
        alert('Please select text to add a slur');
        return;
    }

    const from = editor.getCursor('from');
    const to = editor.getCursor('to');

    // Check if selection already has slur marks
    const existingMarks = editor.findMarksAt(from).concat(editor.findMarksAt(to));
    const existingSlur = existingMarks.find(mark =>
        mark.className && (mark.className.includes('slur-start') || mark.className.includes('slur-end'))
    );

    if (existingSlur) {
        // Remove existing slur
        const slurId = existingSlur.attributes && existingSlur.attributes['data-slur-id'];
        if (slurId) {
            removeSlurById(slurId);
        }
    } else {
        // Add new slur
        addSlur(from, to);
    }
}

function addSlur(from, to) {
    const editor = musicTextEditor.editor;
    const slurId = MusicTextConverter.generateAnnotationId();

    // Add visual marks
    editor.markText(from, {line: from.line, ch: from.ch + 1}, {
        className: 'slur-start',
        attributes: {'data-slur-id': slurId}
    });

    editor.markText({line: to.line, ch: to.ch - 1}, to, {
        className: 'slur-end',
        attributes: {'data-slur-id': slurId}
    });

    // Update state will be handled by event listeners
}

function removeSlurById(slurId) {
    const editor = musicTextEditor.editor;
    const marks = editor.getAllMarks();

    marks.forEach(mark => {
        const markSlurId = mark.attributes && mark.attributes['data-slur-id'];
        if (markSlurId === slurId) {
            mark.clear();
        }
    });
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

## Music-Text Source Tab

### HTML Structure
```html
<!-- Add to tab bar -->
<div class="tab" onclick="switchTab('source', this)">Music-Text Source</div>

<!-- Add tab content -->
<div id="source-tab" class="tab-content">
    <div class="source-controls">
        <button onclick="exportSource()" class="secondary">Export Source (.mt)</button>
        <button onclick="exportSession()" class="secondary">Export Session (.json)</button>
        <button onclick="importSession()" class="secondary">Import Session</button>
        <input type="file" id="sessionFileInput" accept=".json" style="display: none;" onchange="handleSessionImport(event)">
    </div>

    <div class="source-sections">
        <div class="source-section">
            <h3>Clean Music-Text Source</h3>
            <pre id="clean-source-display" class="source-display"></pre>
        </div>

        <div class="source-section">
            <h3>GUI Annotations</h3>
            <pre id="annotations-display" class="source-display"></pre>
        </div>

        <div class="source-section">
            <h3>API Payload</h3>
            <pre id="api-payload-display" class="source-display"></pre>
        </div>
    </div>
</div>
```

### Source Tab Functions
```javascript
function updateSourceTab() {
    if (!musicTextEditor) return;

    const payload = musicTextEditor.getAPIPayload();

    // Update clean source display
    document.getElementById('clean-source-display').textContent = payload.input || '';

    // Update annotations display
    document.getElementById('annotations-display').textContent =
        JSON.stringify(musicTextEditor.guiAnnotations, null, 2);

    // Update API payload display
    document.getElementById('api-payload-display').textContent =
        JSON.stringify(payload, null, 2);
}

// Export functions
function exportSource() {
    const source = musicTextEditor.sourceText;
    downloadTextFile('music.mt', source);
}

function exportSession() {
    const session = {
        sourceText: musicTextEditor.sourceText,
        guiAnnotations: musicTextEditor.guiAnnotations,
        timestamp: Date.now(),
        version: "1.0"
    };
    downloadTextFile('music-session.json', JSON.stringify(session, null, 2));
}

function importSession() {
    document.getElementById('sessionFileInput').click();
}

function handleSessionImport(event) {
    const file = event.target.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = function(e) {
        try {
            const session = JSON.parse(e.target.result);
            musicTextEditor.sourceText = session.sourceText || "";
            musicTextEditor.guiAnnotations = session.guiAnnotations || [];
            musicTextEditor.restoreGUIState();
            musicTextEditor.saveToLocalStorage();
            updateSourceTab();
        } catch (error) {
            alert('Error importing session: ' + error.message);
        }
    };
    reader.readAsText(file);
}

function downloadTextFile(filename, content) {
    const element = document.createElement('a');
    element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(content));
    element.setAttribute('download', filename);
    element.style.display = 'none';
    document.body.appendChild(element);
    element.click();
    document.body.removeChild(element);
}
```

## Backend Implementation

### API Request Structure
```rust
// Extend existing ParseRequest in src/web.rs
#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    input: String,
    system: Option<String>,
    gui_annotations: Option<GuiAnnotations>,
}

#[derive(Debug, Deserialize)]
pub struct GuiAnnotations {
    slurs: Vec<SlurAnnotation>,
}

#[derive(Debug, Deserialize)]
pub struct SlurAnnotation {
    id: String,
    from_line: usize,
    from_col: usize,
    to_line: usize,
    to_col: usize,
}
```

### Reverse Engineering Logic
```rust
// In src/web.rs - modify parse_text function
async fn parse_text(Json(req): Json<ParseRequest>) -> impl IntoResponse {
    let mut input = req.input;

    // Apply GUI annotations if present
    if let Some(annotations) = req.gui_annotations {
        match apply_gui_annotations(&input, &annotations) {
            Ok(enhanced_input) => {
                input = enhanced_input;
                println!("Applied {} slur annotations", annotations.slurs.len());
            },
            Err(e) => {
                return Json(ParseResponse {
                    success: false,
                    error: Some(format!("Annotation error: {}", e)),
                    ..Default::default()
                });
            }
        }
    }

    // Continue with normal parsing pipeline
    match crate::process_notation(&input) {
        Ok(result) => {
            // Generate normal response with enhanced input
            generate_parse_response(result, &input)
        }
        Err(e) => {
            Json(ParseResponse {
                success: false,
                error: Some(e.to_string()),
                ..Default::default()
            })
        }
    }
}

fn apply_gui_annotations(
    content: &str,
    annotations: &GuiAnnotations
) -> Result<String, String> {
    if annotations.slurs.is_empty() {
        return Ok(content.to_string());
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines = Vec::new();

    for (line_idx, line_content) in lines.iter().enumerate() {
        // Check if this line has slurs
        let line_slurs: Vec<_> = annotations.slurs.iter()
            .filter(|s| s.from_line == line_idx)
            .collect();

        if !line_slurs.is_empty() {
            // Generate upper line with slur markings
            let upper_line = generate_upper_line_with_slurs(line_content, &line_slurs)?;
            result_lines.push(upper_line);
        }

        result_lines.push(line_content.to_string());
    }

    Ok(result_lines.join("\n"))
}

fn generate_upper_line_with_slurs(
    content_line: &str,
    slurs: &[&SlurAnnotation]
) -> Result<String, String> {
    let line_length = content_line.chars().count();
    let mut upper_line = vec![' '; line_length];

    for slur in slurs {
        let start = slur.from_col.min(line_length.saturating_sub(1));
        let end = slur.to_col.min(line_length);

        if start >= end {
            return Err(format!("Invalid slur range: {} to {}", start, end));
        }

        // Add slur marking - use underscores for now
        for i in start..end {
            if i < upper_line.len() {
                upper_line[i] = '_';
            }
        }
    }

    Ok(upper_line.into_iter().collect())
}
```

## Integration Examples

### Complete Workflow
```
1. User Input:    |S R G M P|
2. User Action:   Select "R G M" and click slur button
3. GUI State:     CSS classes applied, sourceText unchanged
4. API Call:      {input: "|S R G M P|", gui_annotations: {slurs: [...]}}
5. Backend:       Converts to spatial format: "  ___\n|S R G M P|"
6. Output:        LilyPond: s4 r( g m) p4
```

### State Persistence
```javascript
// Auto-saved to localStorage:
{
  "sourceText": "|S R G M P|",
  "guiAnnotations": [
    {
      "id": "slur_1234567890_abc123",
      "type": "slur",
      "fromLine": 0,
      "fromCol": 2,
      "toLine": 0,
      "toCol": 5
    }
  ],
  "timestamp": 1640995200000,
  "version": "1.0"
}
```

## Benefits

### Complete State Management
- Source text and GUI annotations both persisted
- Import/export for sharing annotated sessions
- Auto-save prevents data loss
- Clean separation between content and interpretation

### Developer Experience
- Source tab provides full transparency
- Easy debugging of API payloads
- Clear state management patterns
- Extensible for future annotation types

### User Experience
- Rich visual feedback with CSS arcs
- Seamless persistence across sessions
- Export options for different use cases
- No loss of work when switching between text and GUI editing

## Future Extensions

### Additional Annotation Types
```rust
pub struct GuiAnnotations {
    slurs: Vec<SlurAnnotation>,
    dynamics: Vec<DynamicAnnotation>,     // Future
    fingerings: Vec<FingeringAnnotation>, // Future
    ornaments: Vec<OrnamentAnnotation>,   // Future
}
```

### Enhanced Export Formats
- Export to spatial music-text with embedded annotations
- Export to MusicXML with slur markings
- Export to standard notation software formats

### Collaboration Features
- Share sessions via URL encoding
- Merge annotation sets from multiple users
- Version history for annotation changes

## Implementation Priority

1. **Phase 1**: Core state management classes and source tab
2. **Phase 2**: CSS visual rendering and basic slur functionality
3. **Phase 3**: Backend API integration and reverse engineering
4. **Phase 4**: Export/import functionality and persistence
5. **Phase 5**: Polish, error handling, and documentation

This specification provides a complete architecture for slurs using CSS classes while maintaining clean separation between musical content and interpretive annotations, with full state management and persistence capabilities.