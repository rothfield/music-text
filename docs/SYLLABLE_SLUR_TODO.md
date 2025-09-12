# Syllable Application and Slur Interaction - TODO

## Current Issue

The syllable post-processing added in `src/parser/horizontal.rs` uses simple left-to-right distribution that doesn't respect slur boundaries. This overrides the existing @doremi-script logic that properly handled slur-syllable interaction.

## Problem

Current post-processing logic:
```rust
// Step 3: Redistribute split syllables back to notes
// Strategy: Extra syllables go to the LAST note
```

This naive approach ignores:
1. **Slur boundaries** - notes within a slur should share the first syllable
2. **Melismatic passages** - slurred notes after the first should get placeholder "_" 
3. **Existing slur-aware logic** - @doremi-script likely already handled this correctly

## Correct Musical Behavior

```
Input: 1_2_3 4
Syllables: ka-ta ge-na
Expected: 
- Note 1: "ka-" (first note of slur gets syllable)
- Note 2: "_" (continuation of slur, placeholder)  
- Note 3: "ta" (end of slur, still uses first syllable)
- Note 4: "ge-" (new syllable for non-slurred note)
```

## Solution Strategy

Instead of replacing syllable distribution, modify approach to:

1. **Split hyphens FIRST** - before any distribution logic
2. **Preserve existing slur-aware logic** - let @doremi-script handle slur-syllable interaction
3. **Find the original syllable distribution** - likely in models/lyrics.rs or similar
4. **Apply hyphen splitting at the input level** - so existing logic sees split syllables

## Files to Investigate

- `src/models/lyrics.rs` - likely contains slur-aware distribution
- `src/parser/vertical.rs` - syllable application logic
- Original @doremi-script implementation - proper slur handling

## Action Items

1. Find existing slur-aware syllable distribution logic
2. Move hyphen splitting to earlier in pipeline (before slur processing)  
3. Remove naive left-to-right redistribution from horizontal.rs
4. Test that slurs + syllables work correctly together
5. Ensure placeholders ("_") are properly generated for melismas

## Test Cases Needed

```
1_2_3 4
ka-ta ge-na
-> Note1: "ka-", Note2: "_", Note3: "ta", Note4: "ge-"

1 2_3_4
a-b-c d-e
-> Note1: "a-", Note2: "b-", Note3: "_", Note4: "c", continuing...
```

## google-cli Analysis & TODOs

### High Priority Bugs & Regressions
-   **[BUG]** Input `"1"` is incorrectly parsed as an upper line `Symbol` instead of a `content_line` note. The parser should require barlines for single-note lines to be unambiguous.
-   **[REGRESSION]** The semitone-based transposition system should be purged and replaced with the old, working tonic-based transposition system.
-   **[BUG]** Web UI: The LilyPond SVG generation endpoint (`/api/lilypond-svg`) times out or hangs, while the main `/api/parse` endpoint works. This indicates a problem in the SVG generation flow specifically.

### Feature Implementation
-   **[FEATURE]** Implement planned `MUSIC_TEXT_SPECIFICATION.md` directives: `Key`, `Tonic`, `Tempo`, `TimeSignature`.
-   **[FEATURE]** Implement planned `UpperLine` elements: `Ornament`, `Chord`, `Tala`, `Mordent`, `Ending`.
-   **[FEATURE]** Implement planned `LowerLine` elements: `BeatGroup`, `FlatMarker`.
-   **[FEATURE]** Implement full syllable parsing and alignment with notes as described in `SYLLABLE_SUPPORT_PLAN.md`.
-   **[FEATURE]** Implement repeat phrases syntax, e.g., `(123)3x`.
-   **[FEATURE]** VexFlow Renderer: Replace the current basic renderer with the more advanced implementation from the old project to support tuplets, slurs, ties, beaming, and ornaments.

### Architectural Refactoring & Tech Debt
-   **[REFACTOR]** Create a dedicated `pitch_parser.rs` module to consolidate all pitch parsing logic from various files (`document/model.rs`, `document/document_parser/content_line.rs`, `models/pitch.rs`). This will centralize notation system detection and ambiguous character resolution.
-   **[REFACTOR]** Clarify the purpose of `classifier.rs`. Its name is misleading; it only classifies annotation lines, not content lines or notation systems. Consider renaming to `annotation_classifier.rs` and document its scope.
-   **[TECH DEBT]** Consolidate duplicate `Position` structs and other types between the `document` and `rhythm` modules to enable further refactoring.
-   **[REFACTOR]** Review the `process_rhythm_batch` implementation to ensure it correctly handles context across staves and provides a tangible benefit over individual processing.
-   **[TECH DEBT]** Remove all dead code, unused imports, and fix compiler warnings to improve code health.

### Web UI & Frontend
-   **[REFACTOR]** Refactor `webapp/app.js` to improve modularity, clean up font management, and optimize event handling.
-   **[FEATURE]** Add UI controls for selecting the tonic/key to test the transposition system.
-   **[IMPROVEMENT]** Add hot-reloading for frontend assets to speed up UI development.

### Testing
-   **[TESTS]** Create a comprehensive test suite for the rhythm and tuplet system.
-   **[TESTS]** Add tests for all supported tonics and notation systems to validate the transposition logic.
-   **[TESTS]** Implement the visual regression testing pipeline outlined in `TESTING_ROADMAP.md` to prevent rendering regressions.
