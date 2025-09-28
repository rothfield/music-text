# Renaming Plan: Musical Terminology Cleanup

## Overview

This document outlines a comprehensive renaming plan to adopt proper musical terminology throughout the codebase, centered around the concept of a **Stave** as the fundamental musical unit.

## Core Philosophy

**Before:** Loose collection of "lines" and "elements"
**After:** Structured **Staves** containing musical content with annotations and lyrics

## Key Terminology Changes

### Primary Concepts

| Current | New | Rationale |
|---------|-----|-----------|
| `musical_line` / `main_line` | `content` (field in Stave) | Cleaner, avoids redundancy |
| `ParsedElement` | `StaveElement` | Elements belong to staves |
| `BeatElement` | `MusicalEvent` | Events occur in musical time |
| `MainLine` (struct) | `Music` (struct) | Clear, concise structure name |

### New Structure Hierarchy

```rust
Document
├── directives: HashMap<String, String>
└── staves: Vec<Stave>

Stave
├── upper_annotations: Vec<AnnotationLine>
├── content: Music                          // The core musical notation
├── lower_annotations: Vec<AnnotationLine>
└── lyrics: Vec<LyricsLine>

Music
├── line_number: Option<usize>
└── elements: Vec<StaveElement>             // Notes, rests, barlines
```

## Visual Example

```
      .    .              ]  upper_annotations
  ________                ]  upper_annotations  
| S R G M | P D N S |     ]  content (Music)
  '   '     *   *         ]  lower_annotations
  do re mi fa sol la ti   ]  lyrics
└─────────────────────────┘
        Stave
```

## Implementation Phases

### Phase 1: Add New Types (Backward Compatible)

**File:** `src/models/parsed.rs`

```rust
// NEW: Core structure
#[derive(Debug, Clone)]
pub struct Stave {
    pub upper_annotations: Vec<AnnotationLine>,
    pub content: Music,
    pub lower_annotations: Vec<AnnotationLine>, 
    pub lyrics: Vec<LyricsLine>,
}

#[derive(Debug, Clone)]
pub struct Music {
    pub line_number: Option<usize>,
    pub elements: Vec<StaveElement>,
}

// NEW: Rename but keep same variants
pub enum StaveElement {
    Note { /* same fields as ParsedElement::Note */ },
    Rest { /* same fields */ },
    // ... all other variants unchanged
}

// COMPATIBILITY: Temporary alias
pub type ParsedElement = StaveElement;
```

### Phase 2: Update Function Signatures

**Priority files:**
1. `src/models/lyrics.rs` - Update `music_lines` → `main_lines` 
2. `src/parser/vertical.rs` - Update element handling
3. `src/parser/horizontal.rs` - Update `BeatElement` → `MusicalEvent`
4. Converter files - Update type references

**Example changes:**
```rust
// OLD
fn has_lyrics(tokens: &[Token], music_lines: &[usize]) -> bool

// NEW  
fn has_lyrics(tokens: &[Token], main_lines: &[usize]) -> bool

// OLD
fn process_elements(elements: Vec<ParsedElement>) -> Vec<BeatElement>

// NEW
fn process_elements(elements: Vec<StaveElement>) -> Vec<MusicalEvent>
```

### Phase 3: Update API Usage

**Before:**
```rust
// Scattered line-based thinking
let music_lines = find_music_lines(&tokens);
for element in parsed_elements {
    // Process individual elements
}
```

**After:**
```rust
// Stave-centric thinking
let document = parse_document(&input);
for stave in document.staves {
    // Process complete staves
    for element in stave.content.elements {
        // Handle musical elements
    }
    
    // Handle lyrics for this stave
    for lyrics_line in stave.lyrics {
        // Assign to musical events
    }
}
```

### Phase 4: Remove Compatibility Aliases

Remove `pub type ParsedElement = StaveElement` and update all references.

## Benefits of New Structure

### 1. Musical Accuracy
- **Stave**: Correct musical term for complete musical unit
- **Content**: Clear distinction from annotations/lyrics
- **Events**: Musical events happen in time

### 2. Better Organization
```rust
// Clean separation of concerns
stave.content.elements        // Musical notation
stave.upper_annotations       // Slurs, ornaments above  
stave.lower_annotations       // Octave markers below
stave.lyrics                  // Text to sing
```

### 3. Simplified Algorithms
```rust
// Lyrics assignment becomes straightforward
fn assign_lyrics_to_stave(stave: &mut Stave) {
    let singable_events = stave.content.elements
        .iter()
        .filter(|e| matches!(e, StaveElement::Note { .. }));
    
    // Auto-assign syllables to notes
    for (event, syllable) in singable_events.zip(stave.lyrics[0].syllables) {
        // Assign syllable to event
    }
}
```

### 4. UI Integration
```rust
// Natural for editing
fn edit_stave(stave_index: usize) {
    let stave = &mut document.staves[stave_index];
    
    // Edit content
    edit_musical_content(&mut stave.content);
    
    // Edit lyrics  
    edit_lyrics(&mut stave.lyrics);
    
    // Edit annotations
    edit_annotations(&mut stave.upper_annotations);
}
```

## Migration Strategy

### Step 1: Preparation
- [x] Design new structure
- [ ] Create this documentation
- [ ] Add new types alongside existing ones

### Step 2: Gradual Migration  
- [ ] Update one module at a time
- [ ] Maintain backward compatibility with type aliases
- [ ] Update tests incrementally

### Step 3: Full Adoption
- [ ] Remove compatibility aliases
- [ ] Update all documentation/comments
- [ ] Verify all functionality works

### Step 4: Testing
- [ ] Run full test suite
- [ ] Test web UI functionality
- [ ] Verify VexFlow and LilyPond output

## Files Requiring Updates

### High Priority
- `src/models/parsed.rs` - Add new structure types
- `src/models/lyrics.rs` - Update parameter names
- `src/parser/vertical.rs` - Update parsing logic
- `src/parser/horizontal.rs` - Update FSM types

### Medium Priority  
- `src/converters/` - Update type references
- `src/lib.rs` - Update main parsing functions
- `src/bin/` - Update CLI usage

### Low Priority
- Documentation files
- Test files
- Comments throughout codebase

## Expected Outcomes

1. **Cleaner Code**: Musical concepts properly modeled
2. **Better UX**: Stave-based editing makes sense to musicians  
3. **Simplified Lyrics**: Clear assignment within stave context
4. **Future-Proof**: Structure ready for advanced features

## Notes

- Maintain full backward compatibility during transition
- Focus on high-impact, low-risk changes first
- Use type aliases to ease migration
- Test thoroughly at each phase

---

This renaming establishes a solid foundation for future development by aligning code structure with musical concepts.