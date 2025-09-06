# TECH NOTE: True Generic Parser Refactor

## 1. Overview

This document outlines a significant architectural refactoring of the Pest parser to a "True Generic" design. This change simplifies the grammar by removing all positional context and premature classification, moving that responsibility to a more powerful and explicit Rust-based classifier.

The goal is to make the grammar responsible only for recognizing valid tokens and basic line structures, while the Rust code handles the semantic interpretation of those structures based on their content and their position relative to the `content_line`.

---

## 2. The "True Generic" Grammar Specification

The core principle is to define a single, unified `annotation_line` that can appear anywhere, and a unified `annotation_item` that can contain any valid non-pitch token.

### High-Level Grammar (Conceptual)

```pest
// The content_line remains the unambiguous anchor, parsed in detail.
content_line = { ... }

// A generic annotation_item can be ANY valid non-content token.
annotation_item = {
    octave_marker | tala | ornament | chord | slur | ending | mordent |
    kommal_indicator | beat_grouping | syllable | whitespace
}

// An annotation_line is simply a collection of these items.
// It makes NO assumption about whether it is an upper, lower, or lyrics line.
annotation_line = { !content_line ~ annotation_item+ ~ NEWLINE }

// A stave is now defined by its anchor and surrounding generic annotation lines.
stave = { annotation_line* ~ content_line ~ annotation_line* }
```

### Key Changes from Previous Design

-   **Elimination of `upper_grammar`, `lower_grammar`, `lyrics_grammar` from the main structural parse.** These will become standalone rules used by the Rust classifier, not by the main document parser.
-   **Elimination of `pre_content_line` and `post_content_line` rules.** These are replaced by the generic `annotation_line`.
-   The main parser's output for annotation lines will be a generic list of `annotation_item`s, not pre-classified `UpperItem`s or `LowerItem`s.

---

## 3. The Rust Classifier Process (Pseudo-code)

The Rust classifier's role is elevated. It receives a `RawStave` containing lists of `RawAnnotationLine`s (which are themselves just lists of generic `annotation_item`s). It then applies logic to classify these lines.

### Data Structures

```rust
// The RawStave from Pest will now contain generic items
struct RawStave {
    pre_content_lines: Vec<Vec<GenericAnnotationItem>>,
    content_line: ContentLine,
    post_content_lines: Vec<Vec<GenericAnnotationItem>>,
}

// The final, classified Stave remains the same
struct Stave {
    upper_lines: Vec<AnnotationLine>,
    lower_lines: Vec<AnnotationLine>,
    lyrics_lines: Vec<LyricsLine>,
    content_line: ContentLine,
}
```

### Classification Logic (Pseudo-code)

```rust
fn classify_raw_stave(raw_stave: RawStave) -> Stave {
    let mut final_stave = Stave::new();
    final_stave.content_line = raw_stave.content_line;

    // --- Phase 1: Classify Pre-Content Lines ---
    for line_items in raw_stave.pre_content_lines {
        // First, check if the line exclusively contains lyrics-related items.
        if is_lyrics_line(&line_items) {
            let lyrics_line = convert_to_lyrics_line(line_items);
            final_stave.lyrics_lines.push(lyrics_line);
        }
        // Otherwise, assume it's an upper annotation line.
        else {
            let upper_line = convert_to_upper_annotation_line(line_items);
            final_stave.upper_lines.push(upper_line);
        }
    }

    // --- Phase 2: Classify Post-Content Lines ---
    for line_items in raw_stave.post_content_lines {
        // Check for lyrics first again.
        if is_lyrics_line(&line_items) {
            let lyrics_line = convert_to_lyrics_line(line_items);
            final_stave.lyrics_lines.push(lyrics_line);
        }
        // If not lyrics, assume it's a lower annotation line.
        else {
            let lower_line = convert_to_lower_annotation_line(line_items);
            final_stave.lower_lines.push(lower_line);
        }
    }

    return final_stave;
}

// Helper function to determine if a line's content is lyrical.
fn is_lyrics_line(items: &Vec<GenericAnnotationItem>) -> bool {
    // A line is considered lyrics if all its non-whitespace items
    // are of the 'syllable' variant.
    return items.iter().all(|item| {
        matches!(item, GenericAnnotationItem::Syllable(_)) ||
        matches!(item, GenericAnnotationItem::Whitespace(_))
    });
}

// Conversion functions will map GenericAnnotationItem to the specific
// types needed for AnnotationLine and LyricsLine, raising errors
// if an invalid item is found (e.g., a 'beat_grouping' item in a line
// being converted to an upper_line).
```

This design creates a clean separation of concerns, resulting in a simpler, more robust, and more maintainable parser.
