# Lyrics Syllable System Architecture

## Overview

The lyrics system in music-text implements sophisticated text annotation assignment with spatial analysis, slur-aware behavior, and cross-renderer support. Lyrics are processed after spatial analysis to ensure proper dependency ordering with slurs and other musical elements.

## Architecture Components

### 1. Spatial Analysis Dependency Chain

The lyrics system operates within a strict dependency hierarchy in the document parser:

```rust
// Apply spatial analysis - assign slurs to notes
assign_slurs_to_document(&mut document);
// Apply spatial analysis - assign lyrics to notes (must be after slurs)  
assign_lyrics_to_document(&mut document);
```

**Key Principle**: Lyrics assignment occurs after slur assignment because melismatic behavior (multiple notes per syllable) depends on slur groupings.

### 2. Text Annotation Architecture

Lyrics utilize the `ParsedChild` hierarchical system for text annotations:

```rust
pub enum ParsedChild {
    // ... other variants
    TextAnnotation(TextAnnotation),
}

pub struct TextAnnotation {
    pub text: String,
    pub annotation_type: TextAnnotationType,
    pub spatial_info: SpatialInfo,
}
```

This allows lyrics to be positioned spatially relative to musical content while maintaining semantic meaning.

### 3. Syllable Assignment Algorithm

#### Core Logic
- **One-to-one mapping**: Each syllable targets one note by default
- **Melismatic behavior**: Slurred note groups receive single syllables
- **Hyphenated splitting**: Multi-word syllables are split across notes

#### Slur-Aware Processing
When a slur spans multiple notes, the entire group is treated as a single syllabic target:
- First note of slur group receives the syllable
- Remaining slurred notes receive no syllable assignment
- Maintains proper melismatic vocal line representation

### 4. Hyphenated Syllable Splitting

The system handles complex syllable structures:

```rust
// Example: "hap-py birth-day" 
// Splits into: ["hap", "py", "birth", "day"]
```

**Algorithm**:
1. Split input text on whitespace to get words
2. Split each word on hyphens to get syllables  
3. Flatten into syllable sequence
4. Assign syllables sequentially to available notes/slur-groups

### 5. Cross-Renderer Implementation

#### VexFlow Renderer
- Uses staff-relative positioning for syllable placement
- Implements sophisticated text positioning relative to note heads
- Supports dynamic positioning based on stem direction

```javascript
// Staff-relative syllable positioning
const syllableY = staff.getYForNote(0) + syllableOffset;
```

#### LilyPond Renderer  
- Integrates syllables into LilyPond's native lyrics syntax
- Handles hyphenated syllables with proper `--` notation
- Maintains compatibility with LilyPond's lyric alignment

```lilypond
\lyricmode { hap -- py birth -- day }
```

## Implementation Details

### Parser Integration
Located in `src/parse/document_parser/document.rs`, the lyrics system:
- Scans for text annotations following musical content
- Builds syllable sequences from annotation text
- Maps syllables to note targets using spatial analysis
- Respects slur groupings for melismatic assignment

### Spatial Information
Each assigned syllable maintains:
- **Target note reference**: Which note/chord receives the syllable
- **Position data**: Horizontal and vertical positioning hints
- **Semantic context**: Relationship to surrounding musical elements

### Error Handling
The system gracefully handles:
- **Syllable shortage**: Extra notes receive no lyrics
- **Syllable overflow**: Extra syllables are discarded with warnings
- **Invalid spatial relationships**: Fallback to sequential assignment

## Usage Patterns

### Basic Lyrics
```
|C D E F| "do re mi fa"
```
Assigns one syllable per note in sequence.

### Melismatic Lines
```  
|C(D E)F| "do re mi"
```
Slurred notes D-E share the syllable "re", with "mi" assigned to F.

### Hyphenated Words
```
|C D E F| "hap-py birth-day" 
```
Splits into ["hap", "py", "birth", "day"] for four-note assignment.

## Benefits

1. **Musical Accuracy**: Respects vocal line conventions with proper melismatic behavior
2. **Cross-Platform**: Consistent behavior across VexFlow and LilyPond outputs  
3. **Extensibility**: Spatial analysis foundation supports future text annotation types
4. **Robustness**: Handles edge cases and complex syllable structures gracefully

## Future Considerations

- Multi-verse support for different lyric sets
- Syllable stress marking and pronunciation guides  
- Integration with other text annotation types (chord symbols, performance directions)
- Advanced positioning controls for complex lyrical layouts