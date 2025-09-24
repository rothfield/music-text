# CSS Annotation System Plan

## Core Insight: WYSIWYG Music Editor

**Architecture:**
- Music text parsing: handles core notation (pitches, rhythm, beats)
- HTML parsing: extracts CSS classes from annotated HTML
- Simple storage: `classes: Vec<String>` field on all music objects
- Existing overlay system: already handles annotation display

## User Workflow
1. User types music text: `123 456`
2. User selects text and clicks annotation buttons
3. UI adds CSS classes: `<span class="begin-slur octave-up-1">1</span>23 <span class="forte end-slur">4</span>56`
4. User submits annotated HTML to server
5. Server parses text content + extracts CSS classes
6. Server stores classes directly in music objects: `note.classes = vec!["begin-slur", "octave-up-1"]`
7. Server returns JSON with classes included
8. UI uses classes to restore visual annotations (existing overlay code)

## Key Benefits
- ✅ True WYSIWYG: what you see in editor is what you get in output
- ✅ No column tracking needed: CSS classes are position-independent
- ✅ Simple UI: select + click button (like Google Docs)
- ✅ Robust parsing: CSS classes unambiguous vs complex syntax
- ✅ Minimal server processing: just extract and store class strings
- ✅ Use existing overlay system: no new display infrastructure needed

## Implementation: Simple Data Model Addition

**Data Model Changes:**
```rust
struct Note {
    // existing fields...
    classes: Vec<String>,  // ["begin-slur", "octave-up-1", "forte"]
}

struct Dash {
    // existing fields...
    classes: Vec<String>,
}

struct Beat {
    // existing fields...
    classes: Vec<String>,  // ["begin-delimited-beat", "accelerando"]
}
```

**Server Processing:**
1. Parse HTML input to extract text + CSS classes
2. Store classes directly (no interpretation needed)
3. Return classes in JSON response
4. UI restores annotations using existing overlay code

## CSS Class Examples
- **Octave:** `octave-up-1`, `octave-down-2`
- **Slurs:** `begin-slur`, `end-slur`
- **Dynamics:** `forte`, `piano`, `crescendo`
- **Articulation:** `staccato`, `accent`, `tenuto`
- **Beat grouping:** `begin-delimited-beat`, `end-delimited-beat`
- **Ornaments:** `mordent`, `trill`, `grace-note`

## Architecture Notes
- CSS classes are stored as-is (no mapping/validation needed)
- Backward compatible: existing text-only parsing still works
- Extensible: any CSS class can be added without server changes
- Simple: minimal "teeny bit of processing" to extract classes

*Plan updated: 2025-09-23*