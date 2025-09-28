# LilyPond Rendering Specification

## Overview

This document specifies how music-text notation is converted to LilyPond source code for musical score generation. The rendering pipeline processes rhythm-analyzed documents and produces LilyPond code that can be compiled to PDF, SVG, or other formats.

### Historical Context

Music-Text has its roots in **Sargam notation**, which is inherently **monophonic** (single melodic line). Recent development on **9/12/2025** added support for **multi-stave input**, expanding the system to handle polyphonic compositions while maintaining its core simplicity.

## Architecture

### Pipeline Flow

```
Input Text → Parser → Document → Rhythm Analyzer → LilyPond Renderer → LilyPond Source → LilyPond Engine
```

#### Detailed Pipeline Stages

1. **Text Parsing**: Converts music-text notation into structured Document with multiple staves
2. **Rhythm Analysis**: Processes each stave's content through FSM to create beat structures
3. **LilyPond Rendering**: Generates complete LilyPond source using Mustache templates
4. **LilyPond Compilation**: External LilyPond engine processes the full document

### Multi-Stave Architecture (Added 9/12/2025)

The system now supports multiple staves in a single document:

```
Document {
  directives: Vec<Directive>,
  elements: Vec<DocumentElement>  // Can contain multiple Stave elements
}

// Multi-stave input example:
|1 2 3    // First stave
           // Blank line separator
|4 5 6    // Second stave
```

#### Historical Design Evolution

**Previous Versions**: Followed the **"one line = one staff"** principle, where each line of input text corresponded directly to a musical staff in the output.

**Current Version**: **Experimental departure** from this approach. The current implementation uses a more complex parsing strategy that groups content into staves based on musical structure rather than strict line boundaries.

**Trade-offs**:
- **Previous**: Simple, predictable mapping (line 1 → staff 1, line 2 → staff 2)
- **Current**: More flexible but potentially less intuitive for users familiar with the original system

Each stave is processed independently through the rhythm analyzer, then combined in the final LilyPond score.

### Key Components

1. **Document Structure**: Parsed representation with staves, directives, and elements
2. **Rhythm Analyzer**: Processes parsed elements into beat structures with durations
3. **LilyPond Renderer**: Converts rhythm-analyzed data to LilyPond notation

## Data Structures

### Input: Rhythm-Analyzed Document

```rust
Document {
    directives: Vec<Directive>,  // title, author, key, etc.
    elements: Vec<DocumentElement>,
    source: Source
}

DocumentElement::Stave {
    lines: Vec<StaveLine>,
    rhythm_items: Option<Vec<Item>>,  // Beat structures from rhythm FSM
    notation_system: NotationSystem,
    source: Source
}

Item = Beat(Beat) | Barline(BarlineType) | Breathmark | Tonic(Degree)

Beat {
    divisions: usize,
    elements: Vec<BeatElement>,
    tied_to_previous: bool,
    is_tuplet: bool,
    tuplet_ratio: Option<(usize, usize)>
}
```

### Output: LilyPond Source

```lilypond
\version "2.24.0"
\language "english"

\header {
  title = "..."
  composer = "..."
}

\score {
  \new Staff {
    \fixed c' {
      \key c \major
      \time 4/4
      % Musical content here
    }
  }
}
```

## Pitch Mapping

### Degree to LilyPond Pitch

| Degree | LilyPond | Note |
|--------|----------|------|
| N1     | c'       | C    |
| N1s    | cs'      | C#   |
| N1b    | cf'      | C♭   |
| N2     | d'       | D    |
| N2s    | ds'      | D#   |
| N2b    | df'      | D♭   |
| N3     | e'       | E    |
| N3s    | es'      | E#   |
| N3b    | ef'      | E♭   |
| N4     | f'       | F    |
| N4s    | fs'      | F#   |
| N4b    | ff'      | F♭   |
| N5     | g'       | G    |
| N5s    | gs'      | G#   |
| N5b    | gf'      | G♭   |
| N6     | a'       | A    |
| N6s    | as'      | A#   |
| N6b    | af'      | A♭   |
| N7     | b'       | B    |
| N7s    | bs'      | B#   |
| N7b    | bf'      | B♭   |

### Octave Modifiers

- Octave 0: Base pitch (c')
- Octave +1: Add apostrophe (c'')
- Octave -1: Add comma (c)
- Multiple octaves: Multiple modifiers (c''' or c,,)

## Duration Mapping

### Fraction to LilyPond Duration

| Fraction | LilyPond | Duration            |
|----------|----------|---------------------|
| 1/1      | 1        | Whole note          |
| 1/2      | 2        | Half note           |
| 1/4      | 4        | Quarter note        |
| 1/8      | 8        | Eighth note         |
| 1/16     | 16       | Sixteenth note      |
| 1/32     | 32       | Thirty-second note  |
| 3/2      | 1.       | Dotted whole        |
| 3/4      | 2.       | Dotted half         |
| 3/8      | 4.       | Dotted quarter      |
| 3/16     | 8.       | Dotted eighth       |
| 3/32     | 16.      | Dotted sixteenth    |
| 7/4      | 1..      | Double-dotted whole |
| 7/8      | 2..      | Double-dotted half  |
| 7/16     | 4..      | Double-dotted quarter|
| 7/32     | 8..      | Double-dotted eighth|
| 7/64     | 16..     | Double-dotted sixteenth|

## Special Elements

### Barlines

| Type   | LilyPond | Symbol |
|--------|----------|--------|
| Single | \|       | \|     |
| Double | \|\|     | \|\|   |
| Final  | \|.      | \|.    |
| RepeatStart | \|: | \|:    |
| RepeatEnd | :\|   | :\|    |

### Ties

Tied notes are indicated by appending `~` to the first note:
- Input: `|1 -`
- Output: `c'4~ c'4`

### Tuplets

```lilypond
\tuplet 3/2 { c'8 c'8 c'8 }  // Triplet
\tuplet 5/4 { c'16 d'16 e'16 f'16 g'16 }  // Quintuplet
```

### Rests

| Event | LilyPond |
|-------|----------|
| Rest  | r4       |

## Rendering Rules

### 1. Document Structure

1. Start with version and language declarations
2. Add header block if directives present (title, author)
3. Create score block with staff
4. Use `\fixed c'` for relative pitch notation
5. Set default key and time signature

### 2. Beat Processing

```rust
for item in rhythm_items {
    match item {
        Beat(beat) => {
            if beat.is_tuplet {
                // Wrap in \tuplet n/m { ... }
            } else {
                // Output elements directly
            }
        },
        Barline(type) => // Output barline
        Breathmark => // Output \breathe
        Tonic(_) => // Skip or set key
    }
}
```

### 3. Note Generation

1. Convert degree to pitch name
2. Apply octave modifiers
3. Add duration from fraction
4. Handle ties with `~`
5. Process ornaments and articulations

### 4. Automatic Beaming Strategy

**Design Decision**: Music-Text uses **LilyPond's built-in automatic beaming** rather than manual beam markup.

#### Rationale

1. **LilyPond Expertise**: LilyPond has sophisticated beaming algorithms that understand musical context
2. **Simplicity**: Avoids complex beam calculation logic in Music-Text
3. **Consistency**: Ensures professional-quality beaming following engraving standards

#### Beaming Behavior

- **Eighth notes and smaller**: Automatically beamed within beat boundaries
- **Quarter notes and larger**: Never beamed
- **Special Case**: A composition of solely 4 sixteenth notes displays as **unbeamed** (LilyPond default)
- **Cross-beat beaming**: Handled intelligently by LilyPond based on time signature

#### Implementation Notes

```rust
// Music-Text does NOT generate:
c'8[ d'8 e'8 f'8]  // Manual beam markup

// Instead generates:
c'8 d'8 e'8 f'8    // Let LilyPond decide beaming
```

The removal of `\autoBeamOff` allows LilyPond's default beaming to take effect.

## Template Configuration & Full Document Processing

### Mustache Template System

Music-Text uses **Mustache templates** to generate complete LilyPond documents. This approach provides:

1. **Separation of Concerns**: Musical logic separate from formatting
2. **Customization**: Easy template modification for different output styles
3. **Complete Documents**: LilyPond receives fully-formed, valid source code

### Template Processing Pipeline

```
Rhythm Items → Music-Text Renderer → Template Variables → Mustache Engine → Complete LilyPond
```

#### Template Variable Mapping

```rust
TemplateContext {
    version: "2.24.0",
    title: Option<String>,           // From directives
    composer: Option<String>,        // From directives
    time_signature: String,          // "\time 4/4"
    key_signature: Option<String>,   // "\key c \major"
    staves: String,                  // Rendered musical content
    lyrics: Option<String>           // Syllable assignments
}
```

### Multi-Stave Template Structure

For multi-stave documents (added 9/12/2025), the template generates either:

#### Single Staff (Legacy/Monophonic)
```lilypond
\score {
  \new Staff {
    \fixed c' {
      {{{staves}}}  // Single stave content
    }
  }
}
```

#### Multiple Staves (Polyphonic)
```lilypond
\score {
  \new StaffGroup <<
    \new Staff {
      \fixed c' {
        {{{stave_1}}}
      }
    }
    \new Staff {
      \fixed c' {
        {{{stave_2}}}
      }
    }
  >>
}
```

### Full Document Approach

**Key Principle**: LilyPond receives the **complete document**, not fragments.

#### Benefits

1. **Context Awareness**: LilyPond can optimize spacing, beaming, and layout across the entire piece
2. **Global Settings**: Headers, page layout, and style settings apply consistently
3. **Cross-Staff Features**: Slurs, ties, and dynamics can span multiple staves
4. **Error Recovery**: LilyPond handles edge cases and provides meaningful error messages

#### Document Completeness

Every generated LilyPond file includes:
- Version declaration (`\version "2.24.0"`)
- Language setting (`\language "english"`)
- Complete header block with metadata
- Full score structure with proper staff grouping
- All necessary layout and formatting directives

### Bar Checking Philosophy

**Design Principle**: Music-Text has historically chosen to **turn off LilyPond's bar checking**.

#### Rationale

1. **Musical Flexibility**: Allows composers to write phrases that don't conform to strict meter
2. **Experimental Notation**: Supports avant-garde and non-Western musical concepts
3. **User Freedom**: Avoids forcing users into Western classical constraints
4. **Simplified Workflow**: Eliminates meter-related compilation errors

#### Implementation

Current templates remove bar checking constraints:
```lilypond
\set Score.measureBarType = #""
\set Score.startRepeatBarType = #""
\set Score.endRepeatBarType = #""
```

This philosophy aligns with Music-Text's Sargam heritage, where rhythmic patterns may not align with Western barline conventions.

### Historical Template Reference

#### Doremi-Script Template (Legacy)

The original doremi-script system used this simpler template approach:

```mustache
#(ly:set-option 'midi-extension "mid")
\version "{{version}}"
\include "english.ly"
\header{
title = "{{{title}}}"
composer = "{{{composer}}}"
tagline = ""  % remove lilypond footer
}

\include "english.ly"

%{
    {{{doremi-source}}}
%}

{{{staves}}}
\layout {
  \context {
  \Score
  \remove "Bar_number_engraver"
  }
}

\midi {
  \context {
  \Score
  tempoWholesPerMinute = #(ly:make-moment 200 4)
  }
}
```

**Key Differences from Current Template**:
- **MIDI Support**: Explicit MIDI output generation
- **Simpler Structure**: Less sophisticated page layout
- **Source Comments**: Embedded original notation as comments (`{{{doremi-source}}}`)
- **Bar Numbers**: Explicitly removed via `Bar_number_engraver`
- **Direct Staves**: Simple `{{{staves}}}` insertion without complex staff grouping

#### Source Preservation Principle

**Design Decision**: **Always include the original music-text source as a LilyPond comment**.

This practice serves multiple purposes:
1. **Debugging**: Enables comparison between input and output during development
2. **Transparency**: Documents the transformation process for users
3. **Round-trip Reference**: Provides canonical source for potential reverse conversion
4. **Version Control**: Embeds the original notation within the generated file
5. **Documentation**: Self-documenting generated files

### Current Standard Template

```mustache
\version "{{version}}"
\language "english"

{{#source_comment}}
% Original music-text source:
{{#source_lines}}
% {{{.}}}
{{/source_lines}}
{{/source_comment}}

\header {
  {{#title}}title = "{{{title}}}"{{/title}}
  {{#composer}}composer = "{{{composer}}}"{{/composer}}
  tagline = ##f
}

\score {
  \new Staff {
    \fixed c' {
      \key c \major
      {{#time_signature}}{{{time_signature}}}{{/time_signature}}
      \set Score.measureBarType = #""
      \set Score.startRepeatBarType = #""
      \set Score.endRepeatBarType = #""
      {{{staves}}}
    }
  }

  {{#lyrics}}
  \addlyrics {
    {{{lyrics}}}
  }
  {{/lyrics}}
}
```

#### Template Variables for Source Preservation

```rust
TemplateContext {
    version: "2.24.0",
    source_comment: bool,                // Enable source embedding
    source_lines: Vec<String>,           // Original input lines
    title: Option<String>,
    composer: Option<String>,
    time_signature: String,
    staves: String,                      // Rendered musical content
    lyrics: Option<String>
}
```

## Error Handling

1. **Unknown Degrees**: Default to c'
2. **Invalid Fractions**: Default to quarter note (4)
3. **Missing Rhythm Items**: Skip stave or output empty
4. **Octave Out of Range**: Clamp to reasonable values

## Examples

### Simple Melody with Source Preservation

Input: `|1 2 3`

Output:
```lilypond
\version "2.24.0"
\language "english"

% Original music-text source:
% |1 2 3

\score {
  \new Staff {
    \fixed c' {
      \key c \major
      \time 4/4
      \set Score.measureBarType = #""
      \set Score.startRepeatBarType = #""
      \set Score.endRepeatBarType = #""
      | c'4 d'4 e'4
    }
  }
}
```

#### Source Comment Benefits

This embedded comment provides:
- **Input Traceability**: Clear record of what music-text input generated this output
- **Debugging Aid**: Easy comparison when troubleshooting rendering issues
- **Educational Value**: Shows the transformation from music-text to LilyPond
- **Archival Quality**: Self-contained files that preserve their origin

### Tied Notes

Input: `|1 - 2 - -`

Output:
```lilypond
| c'4~ c'4 d'4~ d'4~ d'4
```

### Multi-Stave Example (Added 9/12/2025)

Input:
```
|1 2 3 4

|5 6 7 1'
```

Output:
```lilypond
\version "2.24.0"
\language "english"

\score {
  \new StaffGroup <<
    \new Staff {
      \fixed c' {
        \key c \major
        \time 4/4
        | c'4 d'4 e'4 f'4
      }
    }
    \new Staff {
      \fixed c' {
        \key c \major
        \time 4/4
        | g'4 a'4 b'4 c''4
      }
    }
  >>
}
```

This demonstrates the system's evolution from its monophonic Sargam roots to supporting polyphonic compositions.

### Tuplet

Input: `|[123]`

Output:
```lilypond
| \tuplet 3/2 { c'8 d'8 e'8 }
```

## Performance Considerations

1. **Batch Processing**: Process all rhythm items in a single pass
2. **String Building**: Use efficient string concatenation
3. **Memory**: Reuse buffers where possible
4. **Caching**: Cache common conversions (degree→pitch, fraction→duration)

## Binary Output Generation (SVG/PNG/PDF)

### Current Implementation

Music-Text generates visual output by invoking the external **LilyPond engine** via command-line interface. The implementation uses **piping/stdin** to avoid writing temporary `.ly` files to disk.

#### LilyPond Command Invocation

```bash
lilypond --svg -dno-point-and-click --output /temp/temp_<uuid> -
```

**Command Breakdown**:
- `--svg`: Generate SVG output format
- `-dno-point-and-click`: Disable clickable elements for cleaner output
- `--output <path>`: Specify output directory and filename prefix
- `-`: Read LilyPond source from stdin (no temporary files)

#### Implementation Details

```rust
// From src/renderers/lilypond_generator.rs
let mut child = Command::new("lilypond")
    .args(&[
        "--svg",
        "-dno-point-and-click",
        "--output", &format!("{}/temp_{}", self.output_dir, temp_id),
        "-"  // Read from stdin
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Write LilyPond source directly to stdin
stdin.write_all(lilypond_source.as_bytes())?;
```

### Security Considerations

#### Current Risks

1. **Command Injection**: External process execution with user-derived content
2. **Resource Exhaustion**: LilyPond can consume significant CPU/memory
3. **File System Access**: LilyPond writes to temporary directories
4. **Process Management**: Spawned processes may hang or consume resources indefinitely

#### Mitigation Strategies

1. **Input Sanitization**: Validate music-text input before LilyPond compilation
2. **Resource Limits**: Implement timeouts and memory constraints
3. **Sandboxing**: Run LilyPond in isolated containers or chroot environments
4. **Process Monitoring**: Kill runaway LilyPond processes

### Scalability Issues

#### Performance Characteristics

- **LilyPond Startup**: Heavy initialization overhead (~1-2 seconds)
- **Memory Usage**: High memory footprint for complex scores
- **CPU Intensive**: Sophisticated layout algorithms
- **Process Overhead**: Spawning new processes for each request

#### Current Architecture Trade-offs

**LilyPond (Production Output)**:
- ✅ **High Quality**: Professional engraving standards
- ✅ **Full Feature Set**: Complete musical notation support
- ❌ **Heavy Weight**: Slow startup and high resource usage
- ❌ **Security Risk**: External process execution

**VexFlow (Live Preview)**:
- ✅ **Lightweight**: Fast JavaScript rendering in browser
- ✅ **Interactive**: Real-time updates as user types
- ✅ **Secure**: No server-side process execution
- ❌ **Limited Features**: Subset of musical notation
- ❌ **Quality**: Good but not professional engraving level

#### Hybrid Strategy

```
User Input → Parser → Rhythm Analysis
                                ↓
                    ┌─── VexFlow (Live Preview)
                    │
                    └─── LilyPond (Final Output)
```

**Design Rationale**:
- **Live Feedback**: VexFlow provides immediate visual feedback during composition
- **Professional Output**: LilyPond generates publication-quality final scores
- **Resource Optimization**: Heavy LilyPond processing only when explicitly requested

### Alternative Output Formats

**SVG Generation**:
```bash
lilypond --svg -dno-point-and-click --output <path> -
```

**PNG Generation**:
```bash
lilypond --png -dresolution=300 -dno-point-and-click --output <path> -
```

**PDF Generation**:
```bash
lilypond --pdf -dno-point-and-click --output <path> -
```

### Docker Deployment Strategy

#### Containerized Applications

**Docker Approach**: Package Music-Text with embedded LilyPond for consistent deployment.

```dockerfile
FROM debian:bullseye-slim

# Install LilyPond and dependencies
RUN apt-get update && apt-get install -y \
    lilypond \
    fonts-dejavu-core \
    fonts-freefont-ttf \
    && rm -rf /var/lib/apt/lists/*

# Copy Music-Text binary
COPY target/release/music-text /usr/local/bin/
COPY webapp/public /app/public

WORKDIR /app
EXPOSE 3000

CMD ["music-text", "--web"]
```

#### Benefits of Docker + Embedded LilyPond

1. **Dependency Management**: Eliminates "is LilyPond installed?" issues
2. **Version Consistency**: Locks specific LilyPond version for reproducible output
3. **Security Isolation**: Container boundaries limit LilyPond access
4. **Easy Deployment**: Single container with all dependencies
5. **Scalability**: Horizontal scaling with container orchestration

#### Docker Architecture Variants

**Micro-service Approach**:
```
┌─────────────────┐    ┌──────────────────┐
│   Music-Text    │───▶│   LilyPond       │
│   Web Service   │    │   Rendering      │
│   (Parser + UI) │    │   Service        │
└─────────────────┘    └──────────────────┘
```

**Monolithic Approach**:
```
┌─────────────────────────────────┐
│        Music-Text Container     │
│  ┌─────────────┐ ┌────────────┐ │
│  │   Parser    │ │  LilyPond  │ │
│  │   + Web UI  │ │  Embedded  │ │
│  └─────────────┘ └────────────┘ │
└─────────────────────────────────┘
```

#### Production Considerations

**Resource Limits**:
```dockerfile
# Memory and CPU constraints
RUN echo "ulimit -m 512000" >> /etc/profile
RUN echo "ulimit -t 30" >> /etc/profile
```

**Security Hardening**:
```dockerfile
# Non-root user for LilyPond execution
RUN useradd -m -s /bin/bash lilypond-user
USER lilypond-user
```

### Future Optimization Strategies

1. **LilyPond Pool**: Pre-spawned LilyPond processes to reduce startup overhead
2. **Caching**: Cache rendered output for identical input
3. **Progressive Rendering**: Render sections incrementally for large scores
4. **WebAssembly**: Potential future LilyPond compilation to WASM
5. **Native Rendering**: Pure Rust implementation of subset of LilyPond features
6. **Container Orchestration**: Kubernetes deployment with auto-scaling
7. **Edge Computing**: CDN-distributed rendering nodes

## Future Extensions

1. **Lyrics Integration**: Map syllables to notes
2. **Multiple Voices**: Support polyphonic notation
3. **Chord Support**: Render chord symbols and voicings
4. **Ornaments**: Grace notes, trills, mordents
5. **Dynamics**: Volume markings (p, f, mf, etc.)
6. **Articulations**: Staccato, accent, tenuto
7. **Key Signatures**: Support all major/minor keys
8. **Custom Templates**: User-defined LilyPond templates
9. **Transposition**: Automatic transposition support
10. **MIDI Output**: Generate MIDI along with visual score

## Testing Requirements

1. **Unit Tests**: Each conversion function tested independently
2. **Integration Tests**: Full pipeline from input to LilyPond
3. **Regression Tests**: Ensure changes don't break existing functionality
4. **Visual Tests**: Compile LilyPond and verify output appearance
5. **Edge Cases**: Empty input, single note, very long pieces

## Compliance

- LilyPond 2.24.0 compatibility
- English note names (`\language "english"`)
- UTF-8 encoding for all text
- Cross-platform path handling