# Architecture Vision: WYSIWYM Music Editor

## One-Liner

A keystroke-driven, text-first editor that round-trips to a parser and projects the parse back onto the text as live semantic overlays—so users type plain Sargam, but see a semantically accurate, 2D-aware score.

## Core Paradigm: WYSIWYM (What You See Is What You Mean)

This is **not** traditional WYSIWYG music notation software. Instead:

- **Text is the source of truth** - All notation exists as human-readable text
- **View is a projection of meaning** - Visual styling reflects semantic understanding
- **Keystroke-driven** - Real-time feedback without leaving text mode
- **2D-aware** - Understands musical structure (beats, slurs, octaves) in spatial context

## Architecture: LSP-Style Feedback Loop

Like a Language Server Protocol for music notation:

```
User Types → Editor → Parser → Semantic Tree → Spans → Visual Overlay
     ↑                                                           ↓
     ←―――――――――――― Live Feedback Loop ―――――――――――――――――――――――――
```

### The Flow

1. **User types**: `|S R G M|` (plain Sargam text)
2. **Parser analyzes**: Recognizes beat structure, barlines, notes
3. **Semantic tree**: `Beat { elements: [S, R, G, M], divisions: 4 }`
4. **Span generation**: Creates overlay instructions with semantic classes
5. **Editor renders**: Syntax highlighting + beat group indicators
6. **User sees**: Plain text with live semantic feedback

## Technical Implementation

### Core Components

- **`generate_syntax_spans()`**: The "language server" that returns semantic analysis
- **`DocumentNode`**: Unified semantic representation with position + classes
- **`generate_normalized_elements()`**: Single tree walk creating semantic annotations
- **CSS custom properties**: Dynamic styling (`--beat-loop-4`, `--show-divisions`)

### Data Flow

```rust
Input Text → Tokenizer → Parser → DocumentNode[] → {
    Editor: Spans for syntax highlighting
    Web: CharacterStyle[] for semantic CSS
}
```

### Key Innovation: Dual Output Pipeline

The same semantic analysis drives both:
1. **Editor highlighting** - CodeMirror spans for syntax coloring
2. **Visual rendering** - CSS classes for beat groups, slurs, octaves

## User Experience

### What Users Type
```
|S R G M| P D N S|
___     ___
```

### What Users See
- **Syntax highlighting**: Notes, barlines, beat group indicators
- **Semantic overlays**: Beat grouping visual cues, loop indicators
- **Real-time feedback**: Immediate parsing feedback without modal switches

### Contrast with Traditional Editors

| Traditional Music Software | Music-Text WYSIWYM |
|----------------------------|---------------------|
| GUI-first, mouse-driven | Text-first, keyboard-driven |
| Binary format source | Human-readable text source |
| WYSIWYG rendering | WYSIWYM semantic projection |
| Modal editing | Live feedback loop |

## Philosophy: Text as Musical Thinking

Music notation text should be:
- **Readable** - Humans can understand without software
- **Writable** - Fast text input, no GUI friction
- **Semantic** - Structure visible in the source
- **Projectable** - Rich visual feedback from simple text

The editor becomes a **thinking tool** for musicians, not just a notation input device.

## Implementation Status

This architectural vision is implemented through:
- ✅ Unified semantic pipeline (`tree_functions.rs`)
- ✅ Multi-notation system support (Sargam, Number, Western)
- ✅ CodeMirror integration with span overlays
- ✅ CSS-driven semantic styling
- ✅ Real-time parser feedback loop
- ✅ Beat group and slur detection
- ✅ 2D spatial awareness in parser

## Future Vision

- **Collaborative editing** - Multiple users editing same score
- **Version control** - Git-friendly text format
- **Plugin architecture** - Custom notation systems
- **Advanced semantics** - Harmonic analysis, voice leading
- **Export targets** - LilyPond, MusicXML, VexFlow

The goal is **VS Code for music** - where musicians think in text but see meaning.