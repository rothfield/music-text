# Notation Parser Source Code

This document provides a high-level overview of the internal architecture of the Rust-based notation parser. It is intended for developers and AI agents who need to understand, modify, or extend the codebase.

## Core Architecture: A Modular Pipeline

The parser processes raw text into final LilyPond notation through a series of distinct, sequential stages. Each stage is handled by a specific module and produces a well-defined data structure that serves as the input for the next stage.

```
[Raw Text] -> lexer -> [Vec<Token>] -> parser -> [Vec<Node>] -> Document -> lilypond_converter -> [LilyPond String]
```

### Data Flow

1.  **Raw Text**: The input is a string of text containing spatial musical notation.
2.  **`Vec<Token>`**: The `lexer` module consumes the raw text and produces a flat vector of `Token` structs. Each token represents a fundamental unit like a pitch, a barline, a word, or whitespace, with its exact position (line, column).
3.  **`Vec<Node>`**: The `parser` module takes the tokens and transforms them into a hierarchical structure of `Node`s. This is the most critical stage, where:
    *   Spatial relationships (e.g., a lyric below a pitch) are converted into parent-child relationships in the node tree.
    *   Rhythmic groups (beats) are identified and structured.
4.  **`Document`**: The collection of nodes is assembled into a final `Document` struct, which also includes all the extracted `Metadata` (title, directives). This is the complete, structured representation of the musical piece.
5.  **LilyPond String**: The `lilypond_converter` module traverses the `Document` struct, calculates precise rhythmic durations from the spatial data, and generates the final, correctly formatted LilyPond output string.

## Module Responsibilities

-   **`main.rs`**:
    -   Orchestrates the entire pipeline.
    -   Handles command-line argument parsing (`clap`).
    -   Reads the input file and writes the output artifacts (`.clr`, `.json`, `.yaml`, `.ly`).
    -   Calls the various modules in the correct order.

-   **`models/mod.rs`**:
    -   Defines all the core data structures used throughout the application.
    -   Key structs: `Token`, `Node`, `Document`, `Metadata`. This is the canonical source for the shape of the data.

-   **`lexer/mod.rs`**:
    -   **Responsibility**: Text to Token stream conversion.
    -   Splits the raw input text into lines and chunks.
    -   `tokenize_chunk`: Classifies each chunk into a specific `TokenType` (e.g., `PITCH`, `BARLINE`, `WORD`).
    -   `parse_metadata`: Extracts header information like `Title` and `Directives` from the token stream.

-   **`parser/mod.rs`**:
    -   **Responsibility**: Two-phase parsing - spatial analysis and musical structuring.
    -   **Phase 1** - `attach_floating_elements`: Attaches floating elements (ornaments, octave markers, lyrics) to their anchor pitches by analyzing spatial alignment. Like a puzzle game where scattered pieces float up to connect with their proper positions.
    -   **Phase 2** - `group_nodes_into_lines_and_beats`: Takes the hierarchical nodes and groups them into `LINE` and `BEAT` nodes, calculating rhythmic divisions based on character width.

-   **`pitch.rs`**:
    -   **Responsibility**: Music-theoretic logic.
    -   `guess_notation`: Automatically detects the notation system (Western, Sargam, Number).
    -   `lookup_pitch`: Normalizes a text symbol (e.g., "S", "C", "1") into a common `PitchCode` enum for consistent internal processing.

-   **`display/mod.rs`**:
    -   **Responsibility**: Terminal visualization and layout rendering.
    -   `generate_flattened_spatial_view`: Creates the spatially accurate terminal output that shows the result of the parsing.

-   **`colorizer/mod.rs`**:
    -   **Responsibility**: ANSI color formatting for terminal output.
    -   `parse_css_for_ansi`: Reads a simple CSS file (`styles.css`) to configure the colors used in the terminal output.
    -   `colorize_string`, `colorize_title`, `colorize_beat_element`: Apply appropriate colors and formatting to different token types.
    -   `generate_legend_string`: Creates the colorized legend that explains what each token type represents.

-   **`converters/mod.rs`**:
    -   **Responsibility**: Output format generation.
    -   Currently contains the `lilypond` module.
    -   `lilypond_converter.rs`: Contains the logic to convert the final `Document` into a `.ly` file, including the complex fractional math to map spatial divisions to LilyPond note durations.
