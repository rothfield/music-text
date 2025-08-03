# Synthetic Data Generation Requirements

This document outlines the requirements for the synthetic data generator.

## Core Features

The generator should produce text-based output with the following characteristics:

1.  **Text Generation:** It must generate lines of random text, composed of words from a predefined vocabulary.
2.  **Word Loops:** It must be able to draw "loops" under randomly selected words. These are visualized as underlines (e.g., `───`).
3.  **Inter-Word Arcs:** It must draw arcs that connect individual letters of *different* words.

## Constraints and Rules

-   **Arc Length:** The horizontal distance of an arc, from its start character to its end character, must be between 3 and 8 characters, inclusive.
-   **Annotation Overlap:** The system must handle cases where loops and arcs might overlap. The implemented solution is to place conflicting annotations on separate lines below the main text line to ensure they are all visible and do not collide.

## Implementation Details

-   **Language:** The generator was implemented in Rust.
-   **Dependencies:** It utilizes the `rand` crate for all random generation (selecting words, choosing which words get loops, and placing arcs).
-   **Output:** The final rendered text and annotations are printed to the standard output.
