# Proposal Summary & Critique: The Underline-Delimited Block Syntax

This document summarizes and critiques the chosen syntax for grouping multiple staves: a line of underscores (`___`) on a separate line, acting as a fence or delimiter.

This design was selected as it best aligns with the core philosophy of the `music-text` project, which prioritizes structural simplicity and user freedom over semantic rigidity.

---

### The Proposal

The syntax uses a line of three or more underscores (`___`) on its own line to mark the beginning and end of a block of simultaneous musical staves.

**Example:**
```
___
| S R G M |  // Stave 1
| P D N S'|  // Stave 2

| S N D P |  // Stave 3
| M G R S |  // Stave 4
___
```

---

### Core Philosophy & Scope (The "Why")

This syntax is guided by a set of principles that prioritize the musician's workflow.

*   **Structure over Semantics:** The user's primary intent is to state: **"These lines of music happen at the same time."** The syntax captures this structural relationship and nothing more. It is not concerned with what the group *is* (a piano, a choir), only that it *exists*.

*   **Trust the Performer:** The system does not force the user to specify details like clefs or exact octaves. It assumes a musician's context. For example, in a four-part vocal piece, the singers will know their respective ranges. The notation functions as an "open score," where interpretation is left to the performer.

*   **Minimal "Syntax Tax":** The friction of typing non-musical characters is minimized. The underscore is visually clean, unobtrusive, and easy to type.

*   **Decoupling from Staff Notation:** The input format is a distinct system for capturing musical ideas, not a textual replication of staff notation. The renderer's job is to translate this structure into conventional staff notation, but the input syntax is not burdened by those conventions.

---

### How It Works

This design creates a clean separation of concerns:

1.  **The Parser:** Recognizes a starting delimiter (`___`), consumes all subsequent lines as a single group until it finds an ending delimiter, and passes this ordered list of staves to the renderer.
2.  **The Renderer:** Receives the ordered list of staves. It is responsible for applying a minimal set of rendering defaults. For any group, it will:
    *   Render it as a generic `StaffGroup`.
    *   Assign a default clef (e.g., treble) to every stave.
    *   It will **not** attempt any "smart" inference based on the number of staves.

---

### Critique

#### Strengths

*   **Simplicity:** The syntax is simple and intuitive. There is only one rule to learn for grouping.
*   **Philosophical Alignment:** It is a direct expression of the "structure, not semantics" philosophy.
*   **Fluid Editing Workflow:** Managing the number of parts is simple. Adding or deleting a musical line from the middle of a block is a standard text-editing operation that does not require managing the delimiters.
*   **Robust Parsing:** Using distinct lines for delimiters is simple and robust to parse and correctly handles the convention of staves being separated by blank lines.

#### The Primary Trade-off: Collision with Markdown

The main pragmatic weakness is that this syntax collides with a well-established convention in Markdown, where a line of `___` is used to create a horizontal rule (`<hr>`).

**Analysis of the Trade-off:**

This collision is an acceptable trade-off. The `music-text` format is a domain-specific language, not a flavor of Markdown.

*   **Within a `music-text` context** (e.g., a `.mtxt` file or a dedicated editor), the parser will correctly and exclusively interpret `___` as a group delimiter.
*   The risk of misinterpretation only occurs if a user pastes a block of `music-text` into a generic Markdown processor. This is a secondary use case, and the cost of this incompatibility is outweighed by the superior user experience of the underscore syntax *within its primary context*.

### Conclusion

The underline-delimited block is the optimal design choice for the stated scope. It prioritizes the freedom and speed of the writer over the needs of the renderer. Its primary trade-off is sacrificing compatibility with Markdown's horizontal rule syntax for the sake of a more ergonomic and aesthetically fitting delimiter within its own ecosystem.