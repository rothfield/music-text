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

## Rich Text Enhancement (Experimental)

### Overview

An experimental rich text functionality enhancement is planned for the text-area component to provide advanced text formatting capabilities while maintaining all existing music notation parsing functionality. This enhancement aims to explore rich text input possibilities without compromising the robust existing system.

### Selected Rich Text Editor: Pell

**Pell Editor Characteristics:**
- **Size**: Only 1kB, minimal footprint perfect for lightweight requirements
- **Dependencies**: None (ideal for current architecture)
- **Technology**: Uses contentEditable API for native browser rich text support
- **Customization**: Highly configurable, can disable all controls except specified ones
- **Repository**: https://github.com/jaredreich/pell

### Implementation Strategy

#### Minimal UI Design
- **Text Input**: Replace `<textarea>` with Pell contentEditable div
- **Single Control**: Only a Bold button for text selection formatting
- **No Toolbar**: Clean interface maintaining current aesthetic
- **Font Integration**: Preserve existing font selector functionality

#### Data Handling Adaptation
```javascript
// HTML content storage for rich text
LocalStorage.saveRichTextContent(htmlContent);
LocalStorage.savePlainTextContent(extractPlainText(htmlContent));

// Extract plain text for music notation parsing
function extractPlainText(htmlContent) {
    const temp = document.createElement('div');
    temp.innerHTML = htmlContent;
    return temp.textContent || temp.innerText || '';
}
```

#### Module Updates Required

**localStorage.js:**
- Add `saveRichTextContent()` / `loadRichTextContent()`
- Maintain backward compatibility with plain text storage
- Handle both HTML and plain text formats

**ui.js:**
- Adapt `restoreFocusAndCursor()` for contentEditable ranges
- Update `convertMusicNotation()` to work with rich text content
- Preserve existing symbol conversion (♯, ♭) functionality

**app.js:**
- Modify `handleInput()` for contentEditable events
- Update parsing pipeline to extract plain text from HTML
- Maintain 300ms debounced parsing behavior

**fontManager.js:**
- Apply monospace font classes to contentEditable element
- Ensure font changes work with rich text formatting
- Preserve font selection across rich/plain text modes

#### Backward Compatibility Requirements

1. **Music Notation Parsing**: All existing parsing continues unchanged using plain text extraction
2. **Symbol Conversion**: Maintain automatic conversion of # → ♯ and b → ♭
3. **Cursor Persistence**: Adapt cursor position saving for contentEditable ranges
4. **Font Selection**: All monospace fonts work with rich text content
5. **Tab Management**: Preserve all existing tab switching and focus behavior
6. **Real-time Updates**: Maintain 300ms debounced API calls for live preview

#### Technical Challenges

**Plain Text Extraction:**
```javascript
// Robust text extraction for music parser
function getPlainTextForParsing(richTextElement) {
    const plainText = richTextElement.textContent || richTextElement.innerText || '';
    return plainText.replace(/\u00A0/g, ' '); // Replace &nbsp; with regular spaces
}
```

**Cursor Position Adaptation:**
```javascript
// Save cursor position in contentEditable
function saveCursorPosition() {
    const selection = window.getSelection();
    if (selection.rangeCount > 0) {
        const range = selection.getRangeAt(0);
        // Store range information for restoration
        LocalStorage.saveCursorRange(range);
    }
}
```

**Font Integration:**
```css
/* Ensure monospace fonts apply to rich text */
.rich-text-editor.font-default { 
    font-family: 'SF Mono', Monaco, Consolas, monospace; 
}
.rich-text-editor strong { 
    font-weight: bold; 
    font-family: inherit; /* Preserve monospace */
}
```

### Integration Plan

#### Phase 1: Basic Rich Text Setup
1. Add Pell editor library (CDN or local)
2. Replace textarea with contentEditable div
3. Configure minimal UI with Bold button only
4. Test basic rich text input/output

#### Phase 2: Data Layer Integration  
1. Update localStorage handling for HTML content
2. Implement plain text extraction for parsing
3. Adapt cursor position management
4. Test existing functionality preservation

#### Phase 3: UI/UX Polish
1. Apply font selector to rich text editor
2. Style Bold button to match existing UI
3. Ensure responsive design compatibility
4. Test across all existing workflows

### Experimental Status

This rich text enhancement is marked as **experimental** to:
- **Preserve Stability**: Maintain robust music notation parsing
- **Enable Innovation**: Explore advanced text input capabilities  
- **Gather Feedback**: Test user experience with rich text formatting
- **Risk Management**: Easy rollback if issues arise

The implementation will include feature flags to easily toggle between plain text and rich text modes, ensuring the existing stable functionality remains uncompromised.

## Conclusion

The integration of both font selection and experimental rich text capabilities positions the text-area component as a versatile input interface that can serve both traditional music notation parsing and enhanced text formatting needs. The modular architecture ensures that:

- **Core Functionality**: Music notation parsing remains robust and unchanged
- **Enhanced UX**: Rich text capabilities provide new formatting possibilities
- **Flexible Design**: Users can choose between plain text and rich text modes
- **Future-Ready**: Architecture supports additional text enhancement features

This dual-mode approach maintains backward compatibility while opening new possibilities for music notation input and display, aligning with the project's goal of providing a comprehensive and innovative music text notation tool.