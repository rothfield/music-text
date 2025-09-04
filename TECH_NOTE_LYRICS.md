# Technical Note: Data Modeling for Lyrics

## 1. Executive Summary

This note codifies the final decision on how to model lyrics within the new stave-centric architecture.

**Decision:** Syllables are a direct attribute of the musical notes they are sung on. The data model will store lyrics as an `Option<String>` field directly on the `Note` element. A parallel array of syllables at the `Stave` or `Music` level is an incorrect, denormalized approach that leads to severe data integrity issues and is to be avoided.

## 2. The Core Question: Normalization

During the design of the stave-centric parser, a critical data normalization question arose:

> Should a line of lyrics be stored as a parallel array on the `Stave` object (mirroring the input syntax of systems like LilyPond's `\addlyrics`), or should each syllable be attached directly to its corresponding `Note`?

This note analyzes both approaches and explains why direct attachment is the correctly normalized form.

## 3. Analysis: Input Syntax vs. Internal Data Model

The confusion arises from mistaking a user-friendly **input syntax** for a robust **internal data model**. These two concepts are optimized for different goals.

-   **Input Syntax (e.g., LilyPond's `\addlyrics`):** Optimized for **human convenience**. It is often easier for a user to type all the notes and then all the lyrics in separate, parallel blocks.

    ```lilypond
    % This is a convenient INPUT SYNTAX
    \new Staff { c' d' r e' }
    \addlyrics { do re mi }
    ```

-   **Internal Data Model (e.g., our `Stave` struct):** Optimized for **programmatic correctness, safety, and data integrity**. The program needs an unambiguous, direct link between a note and its syllable to function correctly.

The parser's job is to consume the convenient input syntax and build the robust internal data model.

### 3.1. The Incorrect (Denormalized) Model: Parallel Arrays

Storing lyrics as a separate array on the `Stave` or `Music` object is an anti-pattern.

```rust
// ANTI-PATTERN: DO NOT USE
pub struct Music {
    pub elements: Vec<StaveElement>, // [Note, Note, Rest, Note]
    pub syllables: Vec<String>,      // ["do", "re", "mi"] ???
}
```

This model is fundamentally flawed:

1.  **Synchronization Hell:** The `elements` and `syllables` arrays must be kept in perfect sync manually. How are rests handled in the `syllables` array? With `null`? An empty string? Any logic to handle this is complex and brittle.
2.  **Loss of Direct Relationship:** The fundamental rule "a syllable is sung on a note" is not encoded in the structure. It is only *implied* by matching array indices.
3.  **Data Integrity Nightmares:** If a `Note` is deleted from `elements`, the developer *must remember* to also delete the corresponding entry from `syllables`. Forgetting to do so corrupts the data, shifting all subsequent syllables to the wrong notes.
4.  **Inability to Model Melismas:** A melisma (one syllable sung over multiple slurred notes) cannot be represented. The `elements` array would have multiple notes, but the `syllables` array would only have one entry, breaking the parallel structure.

### 3.2. The Correct (Normalized) Model: Direct Attachment

The correct approach is to model the fundamental relationship directly: a syllable is an attribute of a note.

```rust
// CORRECT, NORMALIZED MODEL
pub enum StaveElement {
    Note {
        degree: Degree,
        octave: i8,
        // ... other fields
        syllable: Option<String>, // The syllable belongs HERE
    },
    Rest { /* No syllable field */ },
    Barline { /* No syllable field */ }
}
```

This model is superior for several reasons:

1.  **Unambiguous Association:** The link between a note and its syllable is explicit and guaranteed by the type system.
2.  **Handles Rests Naturally:** The model makes it impossible to attach a syllable to a rest, enforcing a core musical rule at the data structure level.
3.  **Handles Melismas Elegantly:** A melisma is easily modeled. The first note in a slur receives `syllable: Some("glo-")`, while subsequent notes in the slur receive `syllable: None`. The relationship remains clear and correct.
4.  **Data Integrity is Guaranteed:** If a `Note` object is deleted, its associated syllable is deleted with it. It is impossible to have an "orphan" syllable or a synchronization bug.

## 4. Final Decision

The parser will be responsible for taking lyrics from any input format (whether on the same line or in separate blocks) and performing the assignment logic to build the correctly normalized internal model.

**The internal data model will exclusively use direct attachment of syllables to note objects.**

