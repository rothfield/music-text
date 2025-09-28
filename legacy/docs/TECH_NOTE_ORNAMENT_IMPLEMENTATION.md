# Tech Note: Ornament Implementation Plan

## 1. Goal

This document outlines the implementation plan for adding support for musical ornaments (e.g., grace notes, melismas) to the `music-text` parser and rendering pipeline. The implementation will follow existing architectural patterns for spatial annotation, ensuring consistency and maintainability.

## 2. High-Level Approach

The implementation will be handled in four distinct phases, aligning with the existing processing pipeline:

1.  **Parsing**: Update the document parser to recognize ornament syntax on `upper_line`s.
2.  **Spatial Association**: Spatially link the parsed ornaments to their corresponding notes on the `content_line`.
3.  **Rhythm Processing**: Confirm that ornament data is correctly passed through the rhythm FSM without modification.
4.  **Rendering**: Update the LilyPond and VexFlow renderers to generate the correct output for notes with attached ornaments.

The required data structures for this feature, such as `raw::UpperItem::Ornament` and `models::ParsedChild::Ornament`, are already defined, making this a matter of connecting the existing components.

---

## 3. Implementation Phases

### Phase 1: Parsing Ornament Syntax

The first step is to enable the parser to identify ornaments.

-   **File to Modify**: `src/document/document_parser/upper_line.rs`
-   **Task**: The `parse_upper_line` function currently skips unrecognized characters. This logic will be extended to detect ornament patterns.

**Current Logic:**
```rust
// in parse_upper_line function
_ => {
    // Skip other characters for now (ornaments, chords - 🚧 planned)
    col += 1;
    continue;
}
```

**Proposed Change:**
Add a new match arm to handle digits (for undelimited ornaments) and `<` (for delimited ornaments). This will parse a sequence of pitches and create an `UpperElement::Ornament`.

```rust
// in parse_upper_line function
'1'..='9' | '<' => {
    // Logic to parse a sequence of pitches, e.g., "123" or "<456>"
    // Create an UpperElement::Ornament { pitches, source }
    // Add the new element to the `elements` vector
},
```

This change will leverage the already-existing `raw::UpperItem::Ornament` enum variant, which is converted to `document::model::UpperElement::Ornament`.

### Phase 2: Spatial Association

Once parsed, ornaments must be attached to the correct notes. This will be done by mirroring the existing logic for octave markers.

-   **File to Modify**: `src/document/document_parser/document.rs`
-   **Task**: Create a new function `assign_ornaments_to_stave` and call it from `parse_document`.

**Implementation Steps:**

1.  Create `assign_ornaments_to_stave` modeled directly on the existing `assign_octave_markers_to_stave` function.
2.  This function will iterate through a stave's `upper_lines` and build a `HashMap` mapping column positions to `UpperElement::Ornament` data.
3.  It will then iterate through the `ParsedElement::Note`s in the `content_line`.
4.  When a note's column position matches an ornament's position in the map, it will create a `models::ParsedChild::Ornament` and push it into that note's `children` vector.

This correctly transforms the ornament from a line-level item into a note-level attribute.

### Phase 3: Rhythm Processing (Verification Only)

No changes are required in the rhythm FSM. The architecture is already set up to handle this correctly.

-   **File to Review**: `src/rhythm/analyzer.rs`
-   **Verification**: The `BeatElement::from(ParsedElement)` implementation correctly transfers the `children` vector from a `ParsedElement::Note` to an `Event::Note`. This ensures that any ornaments attached in Phase 2 are automatically carried through the rhythm analysis without affecting duration calculations.

### Phase 4: Rendering

The final step is to update the renderers to translate the attached ornament data into the target output format.

#### LilyPond Renderer

-   **File to Modify**: `src/renderers/lilypond/renderer.rs`
-   **Task**: Update the `convert_beat_element_to_lilypond_with_lyrics` function.

**Implementation Steps:**

1.  Inside the function, inspect the `children` vector of the `Event::Note`.
2.  If a `ParsedChild::Ornament` is found, extract its pitches.
3.  Generate the appropriate LilyPond syntax, which is `\grace { ... }` or `\afterGrace { ... }` prepended to the note. The choice between them can be determined by a property on the ornament if we decide to support pre- and post-note ornaments.

#### VexFlow Renderer

-   **File to Modify**: `src/renderers/vexflow/mod.rs`
-   **Task**: The foundation for this is already present.

**Implementation Steps:**

1.  The `convert_beat_to_vexflow_elements` function already contains logic to extract ornaments from a note's `children`.
2.  This logic populates the `ornaments` field in the `VexFlowElement::Note` struct.
3.  This part of the implementation should work as-is once the preceding pipeline stages are complete. The primary task is to ensure the frontend JavaScript renderer correctly interprets this `ornaments` array.

---

## 4. Summary of Changes

-   `src/document/document_parser/upper_line.rs`: Add parsing logic for ornament syntax.
-   `src/document/document_parser/document.rs`: Add `assign_ornaments_to_stave` function to link ornaments to notes.
-   `src/renderers/lilypond/renderer.rs`: Update rendering logic to generate `\grace` commands.
-   `src/renderers/vexflow/mod.rs`: Verify that existing ornament rendering logic functions correctly with the new data flow.

## 5. Conclusion

The implementation of ornaments is a low-risk, high-value feature. The existing architecture is well-suited for it, and the required data models are already in place. The work primarily involves extending existing patterns from parsing through to rendering.
