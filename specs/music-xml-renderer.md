# MusicXML Renderer Specification

## 1. Feasibility Analysis

Based on the project's grammar specification and source code, generating MusicXML is **highly feasible**. The existing multi-stage pipeline (`Parse -> Analyze -> Render`) is well-suited for adding new output formats.

A MusicXML generator can be added as a new module within `src/renderers/`, parallel to the existing `lilypond` and `vexflow` renderers.

### 1.1. Mapping `music-text` Concepts to MusicXML

The `Document` AST in `src/parse/model.rs` contains the necessary semantic information to map to MusicXML's structure.

| `music-text` Concept (in AST) | MusicXML Equivalent | Feasibility |
| :--- | :--- | :--- |
| **Pitches & Octaves** (`Note.pitch_code`, `Note.octave`) | `<pitch>` with `<step>`, `<alter>`, `<octave>` | **High**. The normalized `PitchCode` and `octave` can be directly translated. |
| **Rhythm & Duration** (`Note.numerator`, `Note.denominator`) | `<duration>` and `<type>` (e.g., "quarter") | **High**. The rhythm analyzer already calculates fractional durations. These can be converted to MusicXML's `divisions`-based system. |
| **Tuplets** (`Beat.is_tuplet`, `Beat.tuplet_ratio`) | `<time-modification>` and `<notations><tuplet/></notations>` | **High**. The necessary data is already computed by the rhythm analyzer. |
| **Barlines & Measures** (`ContentElement::Barline`, `ContentLine.elements`) | `<measure>` and `<barline>` | **High**. The parser groups elements between barlines, which maps directly to MusicXML measures. |
| **Staves** (`Document.elements` contains `Stave`s) | `<part>` | **High**. Each `Stave` in the AST can be rendered as a separate `<part>` in MusicXML. |
| **Metadata** (`Document.title`, `Document.author`, `directives`) | `<work>`, `<identification>`, `<defaults>` | **High**. Directives like `Key` and `Tempo` map directly to MusicXML attributes. |
| **Lyrics** (Syllables from spatial analysis) | `<lyric>` | **Medium**. The spatial parser assigns syllables. This logic would feed the MusicXML renderer to create `<lyric>` tags. |
| **Slurs** (From spatial analysis) | `<notations><slur type="..."/></notations>` | **Medium**. The spatial parser identifies slurs, which can be mapped to "start", "stop", and "continue" slur types in MusicXML. |
| **Ornaments** (`mordent`, etc.) | `<ornaments>` | **High**. These can be mapped directly. |

## 2. Implementation Plan

1.  **Add XML Library**: Add a Rust crate like `quick-xml` to `Cargo.toml` for robust and efficient XML generation.
2.  **Create MusicXML Renderer**:
    *   Create a new module: `src/renderers/musicxml/renderer.rs`.
    *   Implement a `render_musicxml(document: &Document) -> Result<String, Error>` function.
3.  **Implement AST Traversal**:
    *   The function will traverse the `Document` AST.
    *   It will build the MusicXML document starting from the `<score-partwise>` root.
    *   It will iterate through staves, measures, and beats, creating corresponding `<part>`, `<measure>`, and `<note>` elements.
4.  **Handle Key Details**:
    *   **Divisions**: Choose a standard "divisions per quarter note" value (e.g., 24 or 480) to accurately represent all rhythms and tuplets.
    *   **Key Signatures**: Convert the text from the `Key:` directive (e.g., "G major") into the number of sharps/flats (`<fifths>`) required by MusicXML.
5.  **Integrate into Pipeline**: Add the new `render_musicxml` function to `pipeline.rs` and expose it as a new output option in `main.rs` and the `web.rs` API handler.

## 3. The `<divisions>` System in MusicXML

MusicXML uses an integer-based timing system rather than fractions directly. The `<divisions>` element is the key to this system.

### 3.1. Establishing a Common Denominator

The `<divisions>` tag sets the number of "ticks per quarter note." It defines a base unit of time for a musical part, and all note durations are expressed as a multiple of this base unit.

**Example:**

With `<divisions>24</divisions>`, a quarter note is made up of 24 ticks.
*   A **quarter note** has `<duration>24</duration>`.
*   An **eighth note** has `<duration>12</duration>`.
*   A **half note** has `<duration>48</duration>`.

This integer-based system avoids floating-point math and makes rhythmic calculations precise.

### 3.2. Representing Complex Rhythms (Tuplets)

This is the most important reason for the `<divisions>` system. Simple fractions are insufficient for representing tuplets cleanly.

**Example: An Eighth-Note Triplet**

An eighth-note triplet fits **three** notes into the space of **two** normal eighth notes (i.e., one quarter note).

*   With `<divisions>24</divisions>`, a quarter note beat lasts for 24 ticks.
*   To fit three triplet notes into that beat, each note must last `24 / 3 = 8` ticks.
*   Each note in the triplet gets `<duration>8</duration>`.

Without the `divisions` system, this `1/3` relationship could not be represented with simple binary fractions.

### 3.3. Visual vs. Timing: `<type>` vs. `<duration>`

MusicXML distinguishes between a note's timing and its visual appearance:

*   `<duration>`: The precise timing in integer "division" units.
*   `<type>`: The visual representation (e.g., "quarter", "eighth").

For the triplet eighth note example, the MusicXML would be:

```xml
<note>
  <pitch>...</pitch>
  <duration>8</duration>  <!-- Timing: 8 divisions -->
  <type>eighth</type>      <!-- Appearance: An eighth note -->
  <time-modification>
    <actual-notes>3</actual-notes>
    <normal-notes>2</normal-notes>
  </time-modification>
  <notations><tuplet type="start"/></notations>
</note>
```

While it *looks* like an eighth note (`<type>eighth</type>`), its actual timing (`<duration>8</duration>`) is shorter than a standard eighth note's duration of 12 ticks.
