# File Operations Implementation Specification

## Overview
This specification defines how file operations (save, load, export, import) are implemented in the Music-Text web interface, including both client-side and server-side components.

## File Formats

### Native Formats

#### Music-Text Document (.mt, .music-text)
- **Type**: Plain text
- **Encoding**: UTF-8
- **Content**: Raw music notation with spatial lines
- **Metadata**: Optional header comments
```
# Title: My Composition
# Author: John Doe
# Date: 2025-01-20

    . .
| S R G M |
  :
```

### Export Formats

#### 1. PDF Export (via LilyPond)
- **Process**: Server-side LilyPond compilation
- **Quality**: Professional engraving quality
- **Use Cases**: Printing, sharing, archival

**Implementation Flow:**
```
Client → Parse notation → Generate LilyPond → Compile to PDF → Download
```

#### 2. MIDI Export
- **Format**: Standard MIDI File (SMF) Type 1
- **Content**: Note events, tempo, time signature
- **Library**: Use Tone.js MIDI export or custom implementation

**Data Structure:**
```javascript
{
  header: {
    format: 1,
    numTracks: 2,
    ticksPerQuarter: 480
  },
  tracks: [
    {
      name: "Music-Text Export",
      events: [
        { time: 0, type: "tempo", bpm: 120 },
        { time: 0, type: "noteOn", pitch: 60, velocity: 80 },
        { time: 480, type: "noteOff", pitch: 60 }
      ]
    }
  ]
}
```

#### 3. LilyPond Source (.ly)
- **Format**: LilyPond markup language
- **Content**: Generated LilyPond code
- **Use Case**: Manual editing in LilyPond IDE

#### 4. SVG Export
- **Source**: Either VexFlow or LilyPond renderer
- **Format**: Scalable Vector Graphics
- **Use Case**: Web embedding, vector editing

## Client-Side Implementation

### File API Usage

#### Save/Download Operations
```javascript
class FileOperations {
  // Download text file (Music-Text, LilyPond source)
  downloadTextFile(content, filename, mimeType = 'text/plain') {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  // Download binary file (PDF, MIDI)
  async downloadBinaryFile(endpoint, params, filename) {
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(params)
    });

    if (!response.ok) throw new Error('Export failed');

    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }
}
```

#### Load/Import Operations
```javascript
class FileImporter {
  // File input handling
  async handleFileSelect(event) {
    const file = event.target.files[0];
    if (!file) return;

    const extension = file.name.split('.').pop().toLowerCase();
    const content = await this.readFile(file);

    switch(extension) {
      case 'mt':
      case 'music-text':
        return this.loadMusicText(content);
      case 'xml':
      case 'musicxml':
        return this.importMusicXML(content);
      case 'mid':
      case 'midi':
        return this.importMIDI(content);
      case 'abc':
        return this.importABC(content);
      default:
        throw new Error(`Unsupported file type: ${extension}`);
    }
  }

  readFile(file) {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = e => resolve(e.target.result);
      reader.onerror = reject;

      // Read as text or binary based on file type
      if (file.type.includes('text') ||
          file.name.match(/\.(mt|music-text|abc|xml|ly)$/i)) {
        reader.readAsText(file);
      } else {
        reader.readAsArrayBuffer(file);
      }
    });
  }
}
```

### UI Integration

#### File Menu Implementation
```javascript
class FileMenu {
  constructor() {
    this.setupEventListeners();
    this.fileOps = new FileOperations();
    this.importer = new FileImporter();
  }

  setupEventListeners() {
    // Save shortcuts
    document.addEventListener('keydown', (e) => {
      if (e.ctrlKey || e.metaKey) {
        switch(e.key) {
          case 's':
            e.preventDefault();
            this.save();
            break;
          case 'S': // Shift+S
            e.preventDefault();
            this.saveAs();
            break;
          case 'o':
            e.preventDefault();
            this.open();
            break;
        }
      }
    });
  }

  save() {
    const content = document.getElementById('input').value;
    const filename = this.currentFilename || 'untitled.mt';
    this.fileOps.downloadTextFile(content, filename);
    this.currentFilename = filename;
  }

  saveAs() {
    const filename = prompt('Enter filename:', this.currentFilename || 'untitled.mt');
    if (filename) {
      this.currentFilename = filename;
      this.save();
    }
  }

  async exportPDF() {
    const parsedDoc = await this.getParsedDocument();
    await this.fileOps.downloadBinaryFile(
      '/api/export/pdf',
      { document: parsedDoc },
      'export.pdf'
    );
  }
}
```

## Server-Side Implementation

### Export Endpoints

#### PDF Export Endpoint
```rust
// Pseudo-code for PDF generation
async fn export_pdf(document: ParsedDocument) -> Result<Vec<u8>, Error> {
    // 1. Generate LilyPond source
    let ly_source = generate_lilypond(&document)?;

    // 2. Write to temp file
    let temp_dir = TempDir::new("music-text-pdf")?;
    let ly_path = temp_dir.path().join("score.ly");
    fs::write(&ly_path, ly_source)?;

    // 3. Run LilyPond
    let output = Command::new("lilypond")
        .arg("-o")
        .arg(temp_dir.path())
        .arg("--pdf")
        .arg(ly_path)
        .output()?;

    // 4. Read PDF
    let pdf_path = temp_dir.path().join("score.pdf");
    let pdf_data = fs::read(pdf_path)?;

    Ok(pdf_data)
}
```

#### MIDI Export Endpoint
```rust
async fn export_midi(document: ParsedDocument) -> Result<Vec<u8>, Error> {
    let midi_file = MidiFile::new();

    // Add tempo track
    midi_file.add_tempo(0, 120);

    // Convert notes to MIDI events
    for element in document.elements {
        if let Element::Note(note) = element {
            let pitch = pitch_to_midi(note.pitch_code);
            let time = calculate_time(note.position);
            let duration = rational_to_ticks(note.duration);

            midi_file.add_note(time, pitch, duration, 80);
        }
    }

    Ok(midi_file.to_bytes())
}
```

### Import Processing

#### MusicXML Import
```rust
fn import_musicxml(xml_content: &str) -> Result<String, Error> {
    let doc = parse_xml(xml_content)?;
    let mut output = String::new();

    for measure in doc.measures() {
        output.push_str("| ");
        for note in measure.notes() {
            let pitch = convert_pitch(note.pitch);
            output.push_str(&format!("{} ", pitch));
        }
        output.push_str("|\n");
    }

    Ok(output)
}
```

## Local Storage Integration

### Auto-save Implementation
```javascript
class AutoSave {
  constructor() {
    this.saveKey = 'music-text-autosave';
    this.metaKey = 'music-text-metadata';
    this.setupAutoSave();
  }

  setupAutoSave() {
    const input = document.getElementById('input');
    let saveTimer;

    input.addEventListener('input', () => {
      clearTimeout(saveTimer);
      saveTimer = setTimeout(() => this.save(), 1000);
    });

    // Save before page unload
    window.addEventListener('beforeunload', () => this.save());
  }

  save() {
    const content = document.getElementById('input').value;
    const metadata = {
      timestamp: Date.now(),
      filename: this.currentFilename,
      cursor: {
        start: input.selectionStart,
        end: input.selectionEnd
      }
    };

    localStorage.setItem(this.saveKey, content);
    localStorage.setItem(this.metaKey, JSON.stringify(metadata));
  }

  restore() {
    const content = localStorage.getItem(this.saveKey);
    const metadata = JSON.parse(localStorage.getItem(this.metaKey) || '{}');

    if (content) {
      document.getElementById('input').value = content;
      if (metadata.cursor) {
        input.setSelectionRange(metadata.cursor.start, metadata.cursor.end);
      }
    }
  }
}
```

## Error Handling

### Common Error Scenarios

1. **Export Failures**
   - LilyPond compilation errors
   - Insufficient server resources
   - Network timeouts

2. **Import Failures**
   - Unsupported file format
   - Corrupted file data
   - Encoding issues

3. **File System Errors**
   - Browser security restrictions
   - Storage quota exceeded
   - File access denied

### Error Recovery
```javascript
class ErrorHandler {
  handleExportError(error, format) {
    console.error(`Export failed for ${format}:`, error);

    const fallbacks = {
      'pdf': () => this.suggestLilyPondSource(),
      'midi': () => this.suggestManualMidiExport(),
      'svg': () => this.offerScreenshot()
    };

    const fallback = fallbacks[format];
    if (fallback) fallback();

    this.showUserMessage(`Export to ${format.toUpperCase()} failed. ${error.message}`);
  }

  handleImportError(error, filename) {
    console.error(`Import failed for ${filename}:`, error);

    if (error.message.includes('Unsupported')) {
      this.showSupportedFormats();
    } else if (error.message.includes('Parse')) {
      this.offerManualCorrection();
    }

    this.showUserMessage(`Failed to import ${filename}. ${error.message}`);
  }
}
```

## Security Considerations

1. **File Size Limits**
   - Client-side: 10MB max for imports
   - Server-side: 1MB max for processing

2. **Content Validation**
   - Sanitize all imported content
   - Validate music notation syntax
   - Prevent script injection

3. **CORS and CSP**
   - Restrict file downloads to same origin
   - Content Security Policy headers
   - Validate MIME types

## Performance Optimizations

1. **Lazy Loading**
   - Load import converters on demand
   - Defer PDF generation libraries

2. **Caching**
   - Cache generated PDFs (5 min TTL)
   - Store parsed documents in memory

3. **Streaming**
   - Stream large MIDI files
   - Progressive PDF rendering

## Testing Requirements

### Unit Tests
- File format conversions
- Import/export accuracy
- Error handling paths

### Integration Tests
- Full export pipeline
- Import → Parse → Export cycle
- Browser compatibility

### Performance Tests
- Large file handling (>1000 measures)
- Concurrent export requests
- Memory usage monitoring

## Future Enhancements

1. **Cloud Storage Integration**
   - Google Drive API
   - Dropbox integration
   - OneDrive support

2. **Batch Operations**
   - Multiple file exports
   - Folder imports
   - Bulk conversions

3. **Version Control**
   - File history tracking
   - Diff visualization
   - Collaborative editing

## Implementation Priority

1. **Phase 1 (Core)**
   - Save/Load Music-Text files
   - Basic export (PDF, MIDI)

2. **Phase 2 (Enhanced)**
   - Import from MusicXML
   - Auto-save with recovery
   - Keyboard shortcuts

3. **Phase 3 (Advanced)**
   - Cloud storage
   - Batch operations
   - Version history