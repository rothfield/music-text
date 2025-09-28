# Retro UI Interface Specification

## Overview
This specification defines the retro, text-based interface for Music-Text that operates without JavaScript, providing a fully functional experience that works in terminal browsers like Lynx, w3m, and other vintage computing environments.

## Separate File Architecture

### Redirect to Dedicated Retro Page
The retro interface uses a separate HTML file for clean separation:

**Automatic Redirect (No JavaScript):**
```html
<!-- In index.html -->
<noscript>
    <meta http-equiv="refresh" content="0; url=retro.html">
    <p>JavaScript is disabled. <a href="retro.html">Click here for the retro version</a></p>
</noscript>
```

**Manual Redirect (Query Parameter):**
```javascript
// At the top of index.html
const urlParams = new URLSearchParams(window.location.search);
if (urlParams.get('retro') === 'true' || urlParams.get('nojs') === 'true') {
    window.location.href = 'retro.html';
}
```

### File Structure
```
webapp/public/
├── index.html     # Modern JavaScript interface
├── retro.html     # Pure HTML forms interface
├── css/
│   ├── style.css     # Modern styles
│   └── retro.css     # Minimal terminal styles
└── js/            # Only loaded by index.html
```

## Retro Interface Design (`retro.html`)

### Control Bar Layout

The retro interface uses a compact control bar with the following layout:
- **Logo**: Musical note (♪) + color-coded notation systems (ABC 123 SRG)
- **Separator**: Pipe character (|)
- **Load/Save Group**: Load, PDF, MIDI, LilyPond, Save buttons
- **Preview**: Main action button

### Complete Retro HTML Structure

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Music Text - Retro Terminal Interface</title>
    <link rel="stylesheet" href="/css/retro.css">
</head>
<body>
    <div class="control-bar">
        <a href="/retro" class="logo">
            <div class="logo-graphic">
                <span class="logo-note">♪</span>
                <span class="logo-text"><span class="abc">ABC</span> <span class="numbers">123</span> <span class="sargam">SRG</span></span>
            </div>
        </a>
        <span class="separator">|</span>
        <div class="save-load-group">
            <form method="POST" action="/retro/load" enctype="multipart/form-data" class="load-form" id="loadForm">
                <input type="file" id="musicfile" name="musicfile" accept=".mt,.txt,.ly,.music-text" style="display: none;" onchange="document.getElementById('loadForm').submit()">
                <button type="button" onclick="document.getElementById('musicfile').click()">Load</button>
            </form>
            <form method="POST" action="/retro" class="save-form" id="saveForm">
                <input type="hidden" name="input" id="saveInput" value="{{preserved_input}}">
                <div class="buttons">
                    <button type="submit" name="action" value="save_pdf">PDF</button>
                    <button type="submit" name="action" value="save_midi">MIDI</button>
                    <button type="submit" name="action" value="save_lily">LilyPond</button>
                    <button type="submit" name="action" value="save_mt">Save</button>
                </div>
            </form>
        </div>
        <form method="POST" action="/retro" class="main-form" id="mainForm">
            <div class="buttons">
                <button type="submit" name="action" value="preview">Preview</button>
            </div>
        </form>
    </div>
    <textarea name="input" rows="3" cols="80" placeholder="Example: | S R G M |" form="mainForm" onchange="copyToSaveForm()">{{preserved_input}}</textarea>

    {{#if_results}}
    <div class="preview-results">
        {{#if_svg}}
        <div class="tabs">
            <button type="button" class="tab-button active" onclick="showTab('staff')">Staff</button>
            {{#if_lilypond}}<button type="button" class="tab-button" onclick="showTab('lilypond')">LilyPond</button>{{/if_lilypond}}
            <button type="button" class="tab-button" onclick="showTab('learn')">Learn</button>
        </div>
        <div id="staff-tab" class="tab-content active">
            <div class="staff-notation">{{svg_content}}</div>
        </div>
        {{#if_lilypond}}
        <div id="lilypond-tab" class="tab-content">
            <pre class="lilypond-source">{{lilypond_content}}</pre>
        </div>
        {{/if_lilypond}}
        <div id="learn-tab" class="tab-content">
            <pre class="help">
Notation Systems:
  Sargam:     | S R G M P D N |
  Number:     | 1 2 3 4 5 6 7 |
  Western:    | C D E F G A B |
  Devanagari: | स र ग म प ध न |
  Tabla:      | dha ge na ta |

Octave Markers:
  Higher octaves (above notes):
    . . .
  | S R G |  (+1 octave)

    : : :
  | S R G |  (+2 octaves)

  Lower octaves (below notes):
  | S R G |
    . . .   (-1 octave)

  | S R G |
    : : :   (-2 octaves)

Examples:
  Twinkle twinkle little star
  | 1 1 5 5 | 6 6 5 - |

  Mary had a little lamb
  | 3 2 1 2 | 3 3 3 - |

  Row row row your boat (polyphonic)
  ###
  | 1 1 1 2 | 3 - - - |

  | 3 3 2 2 | 1 - - - |
  ###

Rhythm:
  Default: Quarter notes
  Tuplets: | [S R G] |
  Ties:    | S - R - |

Bar Lines:
  Single:  | S R | G M |
  Double:  | S R || G M |
  Repeat:  |: S R :| G M |
            </pre>
        </div>
        {{/if_svg}}

        {{#if_error}}
        <pre class="error">{{error_message}}</pre>
        {{/if_error}}
    </div>
    {{/if_results}}

    {{#if_results}}
    {{#if_success}}
    <hr>
    <p>{{success_message}}</p>
    {{/if_success}}
    {{/if_results}}

    <hr>
    <a href="/index.html">Modern Interface</a> | <a href="/">Home</a>

    <script>
    function showTab(tabName) {
        const tabContents = document.querySelectorAll('.tab-content');
        tabContents.forEach(tab => tab.classList.remove('active'));
        const tabButtons = document.querySelectorAll('.tab-button');
        tabButtons.forEach(btn => btn.classList.remove('active'));
        document.getElementById(tabName + '-tab').classList.add('active');
        event.target.classList.add('active');
    }

    function copyToSaveForm() {
        const mainInput = document.querySelector('textarea[name="input"]');
        const saveInput = document.getElementById('saveInput');
        if (mainInput && saveInput) {
            saveInput.value = mainInput.value;
        }
    }
    window.addEventListener('load', copyToSaveForm);
    </script>
</body>
</html>
```

### Retro CSS Styling (`retro.css`)

```css
/* Modern Clean CSS - Compatible with all browsers */

* {
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 0.5rem;
    background: #ffffff;
    color: #333333;
    line-height: 1.4;
    font-size: 16px;
}

/* Control bar layout */
.control-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 0.5rem;
}

.separator {
    color: #6c757d;
    font-weight: normal;
    padding: 0 0.5rem;
}

/* Logo styling */
.logo {
    font-size: 1.1em;
    font-weight: 600;
    text-decoration: none;
    white-space: nowrap;
    transition: all 0.2s ease;
}

.logo:hover {
    text-decoration: none;
    transform: scale(1.05);
}

.logo-graphic {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.logo-note {
    font-size: 1.4em;
    color: #e74c3c;
    font-weight: bold;
    text-shadow: 0 1px 2px rgba(0,0,0,0.2);
}

.logo-text {
    color: #2c3e50;
}

.abc {
    color: #3498db;
    font-weight: bold;
    background: linear-gradient(45deg, #3498db, #2980b9);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
}

.numbers {
    color: #e67e22;
    font-weight: bold;
    background: linear-gradient(45deg, #e67e22, #d35400);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
}

.sargam {
    color: #27ae60;
    font-weight: bold;
    background: linear-gradient(45deg, #27ae60, #229954);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
}

/* Button container */
.buttons {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
}

/* Main form in control bar */
.main-form {
    display: flex;
    align-items: center;
    margin: 0;
    padding: 0;
    border: none;
    background: none;
    box-shadow: none;
}

/* Save/Load group */
.save-load-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.save-form {
    display: flex;
    align-items: center;
    margin: 0;
    padding: 0;
    border: none;
    background: none;
    box-shadow: none;
}

/* Load form in control bar */
.load-form {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin: 0;
    padding: 0;
    border: none;
    background: none;
    box-shadow: none;
}

/* Button styling */
button {
    background: #3498db;
    color: #ffffff;
    border: none;
    padding: 0.35rem 0.7rem;
    margin: 0.2rem 0.2rem 0.3rem 0;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.8rem;
    border-radius: 3px;
    font-weight: 500;
    transition: all 0.2s ease;
}

button:hover {
    background: #2980b9;
    transform: translateY(-1px);
    box-shadow: 0 4px 8px rgba(0,0,0,0.15);
}

/* Tab interface */
.tabs {
    display: flex;
    border-bottom: 1px solid #ddd;
    margin-bottom: 0;
}

.tab-button {
    background: #f8f9fa;
    color: #495057;
    border: 1px solid #ddd;
    border-bottom: none;
    padding: 0.5rem 1rem;
    margin: 0 0 0 0;
    cursor: pointer;
    font-family: inherit;
    font-size: 0.9rem;
    border-radius: 4px 4px 0 0;
    font-weight: 500;
    transition: all 0.2s ease;
}

.tab-button:hover {
    background: #e9ecef;
    transform: none;
    box-shadow: none;
}

.tab-button.active {
    background: #ffffff;
    color: #3498db;
    border-bottom: 1px solid #ffffff;
    margin-bottom: -1px;
}

.tab-content {
    display: none;
    margin-top: 0;
    border: 1px solid #ddd;
    border-top: none;
    border-radius: 0 0 6px 6px;
}

.tab-content.active {
    display: block;
}

/* Textarea styling */
textarea {
    width: 100%;
    background: #ffffff;
    color: #333333;
    border: 2px solid #ddd;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 14px;
    padding: 1rem;
    resize: vertical;
    border-radius: 4px;
    transition: border-color 0.2s ease;
}

textarea:focus {
    outline: none;
    border-color: #3498db;
    box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

/* Results display */
.staff-notation {
    background: #ffffff;
    padding: 1.5rem;
    margin: 1rem 0;
    border: 2px solid #ddd;
    overflow-x: auto;
    color: #000;
    border-radius: 6px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.lilypond-source {
    background: #f8f9fa;
    border: 1px solid #ddd;
    padding: 1.5rem;
    overflow-x: auto;
    color: #495057;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    white-space: pre-wrap;
    border-radius: 6px;
}

.help {
    background: #f8f9fa;
    border: 1px solid #ddd;
    padding: 1.5rem;
    color: #495057;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    white-space: pre;
    overflow-x: auto;
    border-radius: 6px;
    border-left: 4px solid #17a2b8;
}

.error {
    color: #e74c3c;
    background: #fdf2f2;
    border: 1px solid #f5c6cb;
    padding: 1.5rem;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    white-space: pre-wrap;
    border-radius: 6px;
    border-left: 4px solid #e74c3c;
}

/* Links */
a {
    color: #3498db;
    text-decoration: none;
    transition: color 0.2s ease;
}

a:visited {
    color: #9b59b6;
}

a:hover {
    color: #2980b9;
    text-decoration: underline;
}

/* Responsive design */
@media (max-width: 80ch) {
    body {
        padding: 0.5em;
        max-width: 100%;
    }

    textarea {
        cols: 60;
    }
}
```

### Server Response Behavior

For `retro.html` requests:

1. **Preview Action**: Returns full HTML page with:
   - Populated textarea with original input
   - Rendered notation as inline SVG or ASCII art
   - All form controls preserved

2. **Save Actions**: Returns appropriate file download:
   - `Content-Disposition: attachment; filename="score.pdf"`
   - Proper MIME types for each format

3. **Load Action**: Returns HTML with:
   - Textarea populated with loaded file content
   - Success/error message
   - Form state preserved

## Progressive Enhancement Strategy

### Detection and Fallback
```html
<noscript>
    <!-- Show message and redirect to retro version -->
    <meta http-equiv="refresh" content="0; url=/?retro=true">
    <p>JavaScript is disabled. <a href="/?retro=true">Click here for the retro version</a></p>
</noscript>
```

### Dual-Mode Support
The same `index.html` serves both modes:
- **Modern Mode**: Full real-time JavaScript interface
- **Retro Mode** (`?retro=true`): Form-based terminal interface

## Lynx Browser Compatibility

### Requirements for Terminal Browser Support
1. **Semantic HTML**: Proper heading hierarchy
2. **Alt Text**: For any images or visual elements
3. **Form Labels**: Explicit labels for all inputs
4. **Tab Order**: Logical navigation flow
5. **ASCII Fallback**: Text representation of notation

### ASCII Art Notation Preview
For terminal browsers, provide text-based preview:
```
    .   .
|---S---R---G---M---|
  :

Octave markers: . (higher), : (lower)
Bar lines: |
Notes: S, R, G, M, P, D, N
```

## Form Handling Endpoints

### POST /api/parse
**Parameters:**
- `input`: Music notation text
- `action`: preview|save_pdf|save_lily|save_mt

**Response:**
- For preview: HTML page with results
- For save: File download with appropriate headers

### POST /api/load
**Parameters:**
- `musicfile`: Uploaded file

**Response:**
- HTML page with textarea populated
- Error message if parsing fails

## State Management

### Hidden Fields for State
```html
<input type="hidden" name="session_id" value="abc123">
<input type="hidden" name="last_action" value="preview">
<input type="hidden" name="cursor_position" value="42">
```

### Server-Side Session
- Store recent inputs in session
- Preserve cursor position across requests
- Remember user preferences

## Testing Strategy

### Browser Testing Matrix
1. **Lynx**: Full text-mode functionality
2. **w3m**: Japanese text-mode browser
3. **Links**: Another text browser
4. **Firefox with JS disabled**: Modern browser fallback
5. **curl**: API endpoint testing

### Test Commands
```bash
# Test with Lynx
lynx http://localhost:3000/?retro=true

# Test with w3m
w3m http://localhost:3000/?retro=true

# Test with curl
curl -X POST http://localhost:3000/api/parse \
     -d "input=| S R G M |&action=preview"

# Test file upload
curl -X POST http://localhost:3000/api/load \
     -F "musicfile=@test.mt"
```

## Performance Considerations

### Server-Side Rendering
- Cache parsed results for 60 seconds
- Pre-render common patterns
- Minimize HTML size for slow connections

### Bandwidth Optimization
- Inline critical CSS
- No external dependencies
- Compress responses with gzip

## Accessibility Benefits

### Screen Reader Support
- Works perfectly with screen readers
- No dynamic content confusion
- Clear form structure

### Keyboard Navigation
- Full keyboard access
- No JavaScript traps
- Standard form controls

## Implementation Priority

### Phase 1: Core Forms
- Basic textarea input
- Preview and save buttons
- Simple file upload

### Phase 2: Enhanced Features
- Session management
- ASCII art preview
- Multiple export formats

### Phase 3: Polish
- Improved error messages
- Progress indicators
- Help documentation

## Benefits of Retro Mode

1. **Universal Access**: Works on any device with HTML support
2. **Reliability**: No JavaScript errors or browser incompatibilities
3. **Speed**: Instant response, no client processing
4. **Testing**: Easy to automate and debug
5. **Fallback**: Always works when JS fails
6. **Retro Cool**: Full Lynx and vintage computing compatibility!
7. **Terminal Friendly**: Perfect for SSH sessions and text-only environments

## URL Structure

### Query Parameter Options
- `/?retro=true` - Retro terminal mode
- `/?nojs=true` - Alias for retro mode
- `/?retro=true&format=ascii` - ASCII art output
- `/?retro=true&format=svg` - SVG output (default)
- `/?retro=true&session=abc123` - Resume session