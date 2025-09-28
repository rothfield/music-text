# Text Area Specification for Music Text Notation

## Overview

This specification defines the text-area component for the Music Text notation parser web interface with font selection capabilities. The text-area serves as the primary input interface where users type music notation that gets parsed and rendered in real-time.

### Omenad Fonts Investigation (Abandoned)

During development, the Omenad font collection for Indian Classical Music was explored as a potential enhancement. The @omenad/fonts repository contains specialized fonts (OmeBhatkhandeEnglish, OmeSwarlipi, etc.) designed for music notation. However, investigation revealed that the OmeBhatkhandeEnglish font lacked essential characters for representing octave variations (S with dot above/below), making it insufficient for comprehensive music notation display. The approach was abandoned in favor of standard monospace fonts with a flexible font selector.

## Current Implementation

### Basic Structure
- **Element**: `<textarea id="musicInput">`
- **Location**: `/home/john/projects/music-text/webapp/public/index.html:153`
- **Current Font**: `'SF Mono', Monaco, Consolas, monospace`
- **Dimensions**: 80px min-height, 100% width, vertically resizable
- **Placeholder**: `"Enter music notation like: |S R G M|"`

### Existing Features
- **Real-time Parsing**: 300ms debounced API calls to `/api/parse`
- **Cursor Position Persistence**: Saves/restores cursor position via localStorage
- **Focus Management**: Maintains focus across tab switches and button actions
- **Auto-save**: Continuous saving of input text to localStorage
- **Session Restoration**: Restores content and cursor position on page reload

## Font Selection Implementation

### Available Font Options
The font selector provides multiple monospace font options optimized for code/notation display:

1. **Default Mono** - `'SF Mono', Monaco, Consolas, monospace`
2. **Courier New** - `'Courier New', Courier, monospace` 
3. **Source Code Pro** - `'Source Code Pro', 'SF Mono', Monaco, Consolas, monospace`
4. **Fira Code** - `'Fira Code', 'SF Mono', Monaco, Consolas, monospace`
5. **Menlo** - `'Menlo', Monaco, Consolas, monospace`

### Font Selection UI

#### HTML Structure
```html
<div class="font-selector">
    <label for="fontSelect">Font:</label>
    <select id="fontSelect" onchange="changeFontFamily(this.value)">
        <option value="font-default">Default Mono</option>
        <option value="font-courier">Courier New</option>
        <option value="font-source-code">Source Code Pro</option>
        <option value="font-fira-code">Fira Code</option>
        <option value="font-menlo">Menlo</option>
    </select>
</div>
```

#### CSS Font Classes
```css
.font-default { font-family: 'SF Mono', Monaco, Consolas, monospace; }
.font-courier { font-family: 'Courier New', Courier, monospace; }
.font-source-code { font-family: 'Source Code Pro', 'SF Mono', Monaco, Consolas, monospace; }
.font-fira-code { font-family: 'Fira Code', 'SF Mono', Monaco, Consolas, monospace; }
.font-menlo { font-family: 'Menlo', Monaco, Consolas, monospace; }
```

### Dynamic Font Application

#### Real-time Font Switching
```javascript
function updateTextareaFont() {
    const textarea = document.getElementById('musicInput');
    const text = textarea.value;
    const detectedSystem = detectNotationSystem(text);
    
    // Remove all font classes
    textarea.className = textarea.className.replace(/textarea-\w+/g, '');
    
    // Apply new font class
    textarea.classList.add(`textarea-${detectedSystem}`);
    
    console.log('Font switched to:', detectedSystem);
}
```

#### Integration with Existing Debouncing
```javascript
// Modify existing debounced parsing function
let fontUpdateTimeout;
function handleInput() {
    // Immediate font update (no debounce for visual feedback)
    updateTextareaFont();
    
    // Existing debounced parsing (300ms)
    clearTimeout(parseTimeout);
    parseTimeout = setTimeout(() => {
        if (textarea.value.trim()) {
            parseContent(textarea.value);
        }
    }, 300);
}
```

## Technical Implementation

### Font Loading Strategy

#### Preload Critical Fonts
```html
<link rel="preload" href="omenad/fonts/ttf/OmeBhatkhandeEnglish.ttf" as="font" type="font/ttf" crossorigin>
<link rel="preload" href="omenad/fonts/ttf/OmeSwarlipi.ttf" as="font" type="font/ttf" crossorigin>
```

#### Lazy Loading for Other Fonts
```javascript
const fontMap = {
    'bhatkhande-hindi': 'omenad/fonts/ttf/OmeBhatkhandeHindi.ttf',
    'bhatkhande-bangla': 'omenad/fonts/ttf/OmeBhatkhandeBangla.ttf', 
    'bhatkhande-punjabi': 'omenad/fonts/ttf/OmeBhatkhandePunjabi.ttf'
};

function loadFontOnDemand(system) {
    if (!fontMap[system] || loadedFonts.has(system)) return;
    
    const font = new FontFace(system, `url(${fontMap[system]})`);
    font.load().then(() => {
        document.fonts.add(font);
        loadedFonts.add(system);
    }).catch(console.warn);
}
```

### Performance Considerations

#### Font Loading States
```javascript
class FontManager {
    constructor() {
        this.loadedFonts = new Set();
        this.loadingFonts = new Set();
    }
    
    async ensureFont(system) {
        if (this.loadedFonts.has(system)) return true;
        if (this.loadingFonts.has(system)) return false;
        
        this.loadingFonts.add(system);
        try {
            await this.loadFont(system);
            this.loadedFonts.add(system);
            return true;
        } catch (error) {
            console.warn(`Failed to load font for ${system}:`, error);
            return false;
        } finally {
            this.loadingFonts.delete(system);
        }
    }
}
```

### Fallback Handling

#### Graceful Degradation
```css
textarea {
    font-family: 'SF Mono', Monaco, Consolas, monospace; /* Base fallback */
    font-size: 14px;
    font-feature-settings: normal;
}

/* Progressive enhancement */
.textarea-swarlipi {
    font-family: 'OmeSwarlipi', 'SF Mono', Monaco, Consolas, monospace;
}
```

#### Font Load Detection
```javascript
function waitForFont(fontFamily, timeout = 3000) {
    return new Promise((resolve) => {
        if (document.fonts.check(`14px ${fontFamily}`)) {
            resolve(true);
            return;
        }
        
        const timeoutId = setTimeout(() => resolve(false), timeout);
        
        document.fonts.ready.then(() => {
            clearTimeout(timeoutId);
            resolve(document.fonts.check(`14px ${fontFamily}`));
        });
    });
}
```

## User Experience Enhancements

### Visual Feedback

#### Font Loading States
```javascript
function showFontLoadingState(isLoading) {
    const textarea = document.getElementById('musicInput');
    textarea.style.opacity = isLoading ? '0.7' : '1';
}
```

#### Notation Preview
```css
.textarea-with-notation {
    background: linear-gradient(to right, 
        transparent 0%, 
        rgba(9, 105, 218, 0.05) 100%
    );
}
```

### Accessibility

#### Screen Reader Support
```html
<label for="musicInput" class="sr-only">
    Music notation input area. Type your music notation using systems like Sargam (S R G M), 
    Number notation (1 2 3 4), or Western notation (C D E F).
</label>
```

#### High Contrast Mode
```css
@media (prefers-contrast: high) {
    textarea {
        border: 2px solid currentColor;
        background: Canvas;
        color: CanvasText;
    }
}
```

## Integration Points

### Parser API Integration
```javascript
// Enhanced parseContent function
async function parseContent(text) {
    const detectedSystem = detectNotationSystem(text);
    
    try {
        const response = await fetch('/api/parse', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ 
                input: text,
                notation_system: detectedSystem 
            })
        });
        
        const result = await response.json();
        updateOutputTabs(result);
        
    } catch (error) {
        setStatus('Parse error: ' + error.message, 'error');
    }
}
```

### VexFlow Renderer Coordination
```javascript
function updateVexFlowFont(detectedSystem) {
    if (window.vexFlowRenderer) {
        window.vexFlowRenderer.setNotationFont(detectedSystem);
    }
}
```

## Configuration Options

### Font Preferences
```javascript
const FONT_CONFIG = {
    preloadFonts: ['bhatkhande-english', 'swarlipi'],
    fallbackFont: 'SF Mono, Monaco, Consolas, monospace',
    autoDetection: true,
    fontSwitchingDelay: 100, // ms
    fontLoadTimeout: 3000    // ms
};
```

### User Preferences (Future)
```javascript
// Save to localStorage
function saveUserFontPreference(system) {
    localStorage.setItem('music-text-font-preference', system);
}

// Override auto-detection if user has preference
function getUserPreferredFont() {
    return localStorage.getItem('music-text-font-preference') || null;
}
```

## Testing Requirements

### Font Loading Tests
```javascript
// Test font availability
describe('Omenad Font Loading', () => {
    test('loads English Bhatkhande font', async () => {
        await loadFont('bhatkhande-english');
        expect(document.fonts.check('14px OmeBhatkhandeEnglish')).toBe(true);
    });
    
    test('falls back gracefully on font load failure', async () => {
        // Mock font load failure
        const originalFetch = global.fetch;
        global.fetch = jest.fn().mockRejectedValue(new Error('Network error'));
        
        const result = await fontManager.ensureFont('bhatkhande-hindi');
        expect(result).toBe(false);
        
        global.fetch = originalFetch;
    });
});
```

### Notation Detection Tests
```javascript
describe('Notation System Detection', () => {
    test('detects Sargam notation', () => {
        expect(detectNotationSystem('स र ग म')).toBe('bhatkhande-hindi');
        expect(detectNotationSystem('S R G M')).toBe('bhatkhande-english');
        expect(detectNotationSystem('|স র গ ম|')).toBe('bhatkhande-bangla');
    });
});
```

## Performance Metrics

### Target Benchmarks
- **Initial Font Load**: < 200ms for preloaded fonts
- **Font Switch Time**: < 50ms for detection + application
- **Memory Usage**: < 2MB total for all 5 fonts
- **Bundle Impact**: No impact on main JS bundle (fonts loaded separately)

### Monitoring
```javascript
// Performance tracking
function trackFontPerformance(system, startTime) {
    const loadTime = performance.now() - startTime;
    console.log(`Font ${system} loaded in ${loadTime}ms`);
    
    // Send to analytics if needed
    if (window.analytics) {
        window.analytics.track('font_load_time', {
            system,
            load_time_ms: loadTime
        });
    }
}
```

## Migration Strategy

### Phase 1: Base Implementation
1. Add @font-face declarations
2. Implement basic font detection
3. Add font switching logic

### Phase 2: Performance Optimization  
1. Add preloading for critical fonts
2. Implement lazy loading for other fonts
3. Add loading states and fallbacks

### Phase 3: Enhanced Features
1. User font preferences
2. Advanced notation detection
3. Integration with VexFlow font matching

## Syntax-Aware Editor Enhancement

### Overview

After initial exploration into rich text editors, a fundamental architectural conflict was identified. The `music-text` notation system is semantic, where characters and their positions (e.g., `_`, `.`, `|`) are meaningful tokens for the parser. Standard rich text editors (like Pell or Quill) treat formatting as a separate, visual layer (e.g., a `<u>` tag for underline), which is then stripped during plain-text extraction, losing the user's intent. This creates a critical disconnect between what the user sees and what the parser receives.

To resolve this, the enhancement strategy has been pivoted from a rich text editor to a **syntax-aware code editor**. This approach preserves the plain-text data model essential for the parser while providing a rich, interactive, and visually informative editing experience.

### Selected Editor Framework: CodeMirror

**CodeMirror** has been selected as the ideal framework for this task. It is a versatile, extensible code editor component for the web.

**Key Advantages:**
- **Plain-Text Core**: The editor's underlying data model is always the pure, plain text required by the `doremi-script` parser. There is no risk of semantic information being lost.
- **Syntax Highlighting**: CodeMirror's primary feature is its ability to parse text in real-time and apply styling. We can create a custom language mode based on `doremiscript.ebnf` to color-code pitches, barlines, ornaments, slurs, and other tokens, providing excellent visual feedback.
- **Extensibility**: The API allows for creating custom commands. We can add UI buttons (e.g., "Add Slur") that programmatically manipulate the text, helping users write correct syntax (e.g., wrapping a selection in `()`).
- **Spatial Awareness**: As a code editor, it naturally handles monospaced fonts, line-based structures, and character alignment, which is critical for the spatial nature of the music notation.

### Alternatives Considered

- **Monaco Editor**: The editor that powers VS Code. It is extremely powerful but is a much heavier library and more complex to integrate than CodeMirror, making it overkill for this project's current needs.
- **Pell / Quill (Re-evaluated)**: Discarded due to the semantic mismatch. A bolded note in Quill is visually bold but semantically just a plain character to our parser. An underlined note is visually underlined, but the parser never sees the `_` character it expects for a *kommal*. This approach is fundamentally incompatible with the `doremi-script` grammar.

### Implementation Strategy

The integration will be phased to ensure stability and functionality at each step.

#### Phase 1: Basic Integration
1.  Add the CodeMirror library (CSS and JS) from a CDN to `index.html`.
2.  Replace the `<textarea id="musicInput">` with a host element for the editor.
3.  Initialize a basic CodeMirror instance, ensuring its content is synchronized with the application's state.
4.  Wire up CodeMirror's "change" event to the existing debounced `handleInput()` function to preserve the live-preview functionality.

#### Phase 2: Custom Language Mode (Syntax Highlighting)
1.  Analyze the `doremiscript.ebnf` grammar to define the core tokens of the language (pitches, barlines, slurs, ornaments, attributes, comments).
2.  Implement a custom CodeMirror language mode to apply distinct CSS classes to these tokens within the editor.
3.  Add CSS rules to style these tokens, providing immediate visual feedback to the user as they type.

#### Phase 3: UI Enhancements & Editor Commands
1.  Add UI buttons for common notation tasks (e.g., "Add Slur," "Add Ornament").
2.  Implement corresponding CodeMirror commands that manipulate the selected text to insert the correct syntax.
3.  Adapt existing features like cursor position persistence to work with CodeMirror's API.

### Conclusion

This new approach using a syntax-aware editor is a significant improvement over the previous rich text plan. It is architecturally sound, aligns perfectly with the project's existing parsing and rendering pipeline, and provides a clear path to a much richer and more intuitive user experience without compromising the integrity of the plain-text data model. This strategy directly supports the user in writing valid `music-text` notation while providing the immediate visual feedback characteristic of a modern editor.