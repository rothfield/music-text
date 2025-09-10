# Tech Note: Ornaments in doremi-script

This document details the syntax and implementation of musical ornaments in the doremi-script notation system.

## Overview

Ornaments are musical decorations, such as grace notes or melismas, that are not part of the main melodic rhythm. In doremi-script, they are represented as a sequence of pitches attached to a primary note.

## Syntax

Ornaments are defined on an `upper-line`, a dedicated line of text appearing directly above the main `notes-line`. The system relies on vertical (columnar) alignment to associate an ornament with its target note.

There are two supported formats for writing ornaments:

1.  **Undelimited**: A consecutive sequence of sargam pitches.
    ```
    rgm
    | S R G M |
    ```

2.  **Delimited**: A sequence of sargam pitches enclosed in angle brackets `< >`. This is useful for clarity when there are other elements on the upper line.
    ```
    <rgm>
    | S R G M |
    ```

In both examples, the ornament `rgm` is spatially aligned with and attached to the note `S`.

## Implementation Details

The association of an ornament to a note is a two-stage process involving parsing and a post-processing step called "collapsing".

### 1. Parsing (`doremiscript.ebnf`)

The grammar defines `sargam-ornament` as a valid element within a `sargam-upper-line`.

```ebnf
sargam-upper-line ::= ... sargam-ornament ...

sargam-ornament ::= sargam-ornament-pitch+ | <delimited-sargam-ornament>

<delimited-sargam-ornament> ::= "<" sargam-ornament-pitch+ ">"
```

Initially, the parser builds a raw abstract syntax tree where the `upper-line` and `notes-line` are distinct entities within a `stave`. At this stage, the ornament and its target note are not yet semantically linked.

### 2. Post-Processing (`core.cljc`)

The `collapse-stave` function performs the crucial step of associating spatially aligned elements.

1.  **Column Mapping**: The function scans all annotation lines (like `upper-line`) and builds a map where keys are column indices and values are the elements found at that column.
2.  **Association**: It then traverses the `notes-line`. For each note, it looks up its column index in the map. If an ornament is found at that same index, it is moved from the `upper-line` and attached as metadata to the corresponding note vector in the final, collapsed parse tree.

### Octaves and Ornaments

Octave indicators (`.` for one octave, `:` for two) can be applied to the notes within an ornament. These indicators must be placed on another `upper-line` directly above the line containing the ornament, maintaining the same spatial alignment.

**Example:**

```
  . .
  rgm
| S R G M |
```

In this case, the `r` and `g` notes within the ornament `rgm` will be raised by one octave. The `collapse-stave` logic correctly assigns these octave markers to the ornament pitches before the ornament itself is attached to the main note `S`.
